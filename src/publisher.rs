use crate::{
    error::NetworkCommunicationError,
    test_config::*,
    traits::{Chunk, Publisher},
    utils::*,
};
use aeron_rs::concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer};
use aeron_rs::context::Context;
use aeron_rs::utils::errors::AeronError;
use aeron_rs::{aeron::Aeron, publication::Publication};
use eyre::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task::JoinHandle};

// TEMP: will change to generic message types

#[derive(Debug, Clone)]
pub enum DeserializedMessage {
    NormalizedBook(NormalizedBook),
    NormalizedTrade(NormalizedTrades),
}

impl Chunk for DeserializedMessage {
    fn chunk_data(&self) -> Vec<Vec<u8>> {
        todo!();
    }
}

#[derive(Debug, Clone)]
pub struct NormalizedTrades;

#[derive(Debug, Clone)]
pub struct NormalizedBook;

#[derive(Debug, Clone)]
pub struct CapnpMessage;

impl Chunk for CapnpMessage {
    fn chunk_data(&self) -> Vec<Vec<u8>> {
        todo!();
    }
}

#[derive(Debug)]
pub enum AeronMessage {
    CapnpMessage(CapnpMessage),
    //TODO: Add other transport/communication message types based on integrations
    Other(DeserializedMessage),
}

//

#[derive(Clone)]
pub struct AeronConfig {
    dir_prefix: String,
    channel: String,
    stream_id: i32,
    number_of_messages: Option<i64>,
    linger_timeout_ms: u64,
    context: Option<Context>,
}
//NOTE: default methods assume single stream, on single channel.
//TODO: Write aeron multi-stream method logic on second iteration of publisher. Current version
// only includes single Publication instance, i.e. single stream_id.
impl AeronConfig {
    pub fn default_udp() -> Result<Self> {
        //NOTE: Default -> Single stream handling
        Ok(Self {
            dir_prefix: String::new(),
            channel: String::from(DEFAULT_UDP_CHANNEL),
            stream_id: DEFAULT_UDP_STREAM_ID.parse()?,
            number_of_messages: None,
            linger_timeout_ms: 10000,
            context: None,
        })
    }
    pub fn default_ipc() -> Result<Self> {
        Ok(Self {
            dir_prefix: String::new(),
            channel: String::from(DEFAULT_IPC_CHANNEL),
            stream_id: DEFAULT_IPC_STREAM_ID.parse()?,
            number_of_messages: None,
            linger_timeout_ms: 10000,
            context: None,
        })
    }
    pub fn new(&self, channel: &str, stream_id: &str, linger_timeout_ms: u64) -> Result<Self> {
        Ok(Self {
            dir_prefix: String::new(),
            channel: String::from(channel),
            stream_id: stream_id.parse()?,
            number_of_messages: None,
            linger_timeout_ms,
            context: None,
        })
    }

    pub fn single_channel_multi_stream(
        &self,
        channel: &str,
        streams: Vec<&str>,
        linger_timeout_ms: u64,
    ) {
        // NOTE: See note above AeronPublicationManager
        // TODO: config file functionality should include single channel, multiple stream_ids, streams should
        // generate AeronConfig for each stream variation.
        unimplemented!();
    }

    pub fn with_static_number_of_messages(
        &self,
        channel: &str,
        stream_id: &str,
        number_of_messages: Option<i64>,
        linger_timeout_ms: u64,
    ) -> Result<Self> {
        Ok(Self {
            dir_prefix: String::new(),
            channel: String::from(channel),
            stream_id: stream_id.parse()?,
            number_of_messages,
            linger_timeout_ms,
            context: None,
        })
    }
    pub fn set_context(&mut self) -> Result<()> {
        let mut context = Context::new();
        if !self.dir_prefix.is_empty() {
            context.set_aeron_dir(self.dir_prefix.clone());
        }

        context.set_new_publication_handler(Box::new(on_new_publication_handler));
        context.set_error_handler(Box::new(error_handler));
        context.set_pre_touch_mapped_memory(true);

        self.context = Some(context);
        tracing::debug!("Aeron context created");
        Ok(())
    }
    pub fn context(&self) -> &Option<Context> {
        &self.context
    }
    pub fn get_or_initialize_context(&mut self) -> Result<Context> {
        if self.context.is_none() {
            self.set_context()?
        }

        Ok(self.context.to_owned().unwrap())
    }
}

#[derive(Clone)]
pub struct PublicationComponents {
    config: Arc<AeronConfig>,
    aeron_instance: Arc<Aeron>,
    context: Arc<Context>,
    pub publication: Arc<Mutex<Publication>>,
    publication_id: i64,
}

impl PublicationComponents {
    pub fn register_with_media_driver(mut config: AeronConfig) -> Result<Self> {
        let context = config.get_or_initialize_context()?;
        let mut aeron = Aeron::new(context.clone())?;
        let pub_id = aeron.add_publication(str_to_c(&config.channel)?, config.stream_id)?;

        let mut publication = aeron.find_publication(pub_id);

        while publication.is_err() {
            std::thread::yield_now();
            publication = aeron.find_publication(pub_id);
        }

        Ok(Self {
            config: Arc::new(config),
            aeron_instance: Arc::new(aeron),
            context: Arc::new(context.clone()),
            publication: publication?,
            publication_id: pub_id,
        })
    }
}

// NOTE: Current implementation with AeronPublicationManager handles cases where we have multiple streams
// (stream_ids) within the aeron instance, for which we need the option to publish different
// internal message streams to different aeron Publications instances. (future implementation)

//NOTE: local StreamId = registration_id on ClientConductor, return from add_publication = registration_id
type StreamId = i32;

pub struct AeronPublicationManager {
    publications: HashMap<StreamId, Arc<Mutex<Publication>>>,
}

impl AeronPublicationManager {
    pub fn new() -> Self {
        Self {
            publications: HashMap::new(),
        }
    }

    pub fn add_publication(&mut self, publication: Arc<Mutex<Publication>>) {
        let lock = publication.lock().unwrap();
        let id = lock.stream_id();
        drop(lock);
        self.publications.insert(id, publication);
    }

    fn get_publication(&self, stream_id: StreamId) -> Option<Arc<Mutex<Publication>>> {
        self.publications.get(&stream_id).cloned()
    }

    //TODO Revisit below method when other components completed:
    // will likely need to adjust with Vec<Arc<mutex<publication>>>

    fn extend_with_publications(&mut self, publications: Vec<Publication>) {
        publications.into_iter().for_each(|publication| {
            self.publications
                .insert(publication.stream_id(), Arc::new(Mutex::new(publication)));
        });
    }
}

pub struct AeronPublisher {
    receiver: mpsc::Receiver<AeronMessage>,
    components: Arc<Mutex<PublicationComponents>>,
    publisher: Arc<Mutex<Publication>>,
    // active_channels: Arc<Mutex<AeronPublicationManager>>,
}

impl AeronPublisher {
    fn new(
        receiver: mpsc::Receiver<AeronMessage>,
        components: Arc<Mutex<PublicationComponents>>,
    ) -> Result<Self> {
        let lock = components.lock().unwrap().publication.clone();

        Ok(AeronPublisher {
            receiver,
            components: Arc::clone(&components),
            publisher: lock,
        })
    }

    pub async fn handle_message(&mut self, msg: AeronMessage) -> Result<()> {
        tracing::debug!("Message received by aeron publisher");
        match msg {
            AeronMessage::CapnpMessage(msg) => {
                self.publisher
                    .lock()
                    .unwrap()
                    .publish(msg)
                    .map_err(|err| NetworkCommunicationError::AeronInstanceError(err));
            }
            AeronMessage::Other(data) => {
                todo!();
            }
        }
        Ok(())
    }
}

unsafe impl Send for PublicationComponents {}

async fn run_aeron_actor(mut actor: AeronPublisher) -> Result<()> {
    tokio::spawn(async move {
        while let Some(msg) = actor.receiver.recv().await {
            match actor.handle_message(msg).await {
                Ok(_) => tracing::debug!("Message sent to handle_message for publication"),
                Err(_) => tracing::error!("failed to handle_message"),
            }
        }
    })
    .await;
    Ok(())
}
pub struct AeronPublisherHandler {
    pub sender: mpsc::Sender<AeronMessage>,
}

impl AeronPublisherHandler {
    pub fn new() -> Result<(mpsc::Sender<AeronMessage>, JoinHandle<()>)> {
        let (sender, receiver) = mpsc::channel::<AeronMessage>(512);

        let config = AeronConfig::default_udp()?;
        let components = Arc::new(Mutex::new(
            PublicationComponents::register_with_media_driver(config)?,
        ));
        let /* mut */ manager = Arc::new(Mutex::new(AeronPublicationManager::new()));

        {
            let lock = components.lock().unwrap();

            manager
                .lock()
                .unwrap()
                .add_publication(lock.publication.clone());
        }

        tracing::debug!("aeron active_channels instantiated");

        let actor = AeronPublisher::new(
            receiver,
            Arc::clone(&components), /* , active_channels */
        )?;
        tracing::debug!("aeron actor instantiated");

        let handle = tokio::spawn(async move {
            run_aeron_actor(actor).await;
            tracing::debug!("aeron handle generated")
        });

        Ok((sender, handle))
    }
}

impl<DeserializedMessage: Chunk> Publisher<DeserializedMessage> for Publication {
    type Error = AeronError;

    fn publish(&self, msg: DeserializedMessage) -> Result<(), Self::Error> {
        let chunks = msg.chunk_data();
        for chunk in chunks {
            let buffer = AlignedBuffer::with_capacity(1024);
            let src_buffer = AtomicBuffer::from_aligned(&buffer);
            src_buffer.put_bytes(0, &chunk);

            self.offer(src_buffer);
        }

        /*
        NOTE: Commented method below:
            - offer_part: is an alternative Publication method for more precise control of buffer sending
                - Allows publishing portion of buffer to aeron stream, specifying sub-range of buffer to send
            offer_part usage: self.offer_part(src_buffer, 0, msg.data.len() as i32);

            - try_claim: zero copy, try to claim range in publication log, see aeron-rs for instantiation methods
                - usage requires setting idle_strategy, try_claim.err() -> idle strategy,

           try_claim usage:
                buffer_claim.buffer().put(buffer_claim.offset, msg)
                buffer_claim.commit

        */

        Ok(())
    }
}

//
// #[cfg(test)]
// mod tests {
//     use super::super::aeron_config::{
//         DEFAULT_IPC_CHANNEL, DEFAULT_UDP_CHANNEL, TEST_IPC_STREAM_ID, TEST_UDP_STREAM_ID,
//     };
//     use super::*;
//     use aeron_rs::concurrent::logbuffer::header::Header;
//     use aeron_rs::concurrent::strategies::{SleepingIdleStrategy, Strategy};
//     use aeron_rs::image::Image;
//     use aeron_rs::publication::Publication;
//     use aeron_rs::utils::types::Index;
//     use rand::distributions::Uniform;
//     use rand::{thread_rng, Rng};
//     use std::collections::HashSet;
//     use std::slice;
//     use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};
//     use std::sync::{Arc, Mutex};
//     use std::thread;
//
//     lazy_static! {
//         pub static ref SUBSCRIPTION_ID: AtomicI64 = AtomicI64::new(-1);
//     }
//
//     // NOTE: Requires associated subscriber with equivalent stream_id to AeronConfig generated
//     // stream_id, in this case default_udp()
//     // NOTE: Confirmation of publish must be established on receipt by aeron subscriber.
//
//     // TODO: Tests pass regardless of active subscriber. Consider test refactor to confirm active
//     // subscriber on aeron instance. Check aeron_rs::Aeron for method to confirm media driver
//     // state and include in assert.
//     #[test]
//     fn config_to_publish() -> Result<()> {
//         let msg = "test message";
//         let str_msg = format!("{}", msg);
//
//         let capnp_msg = CapnpMessage {
//             data: str_msg.into(),
//         };
//
//         let config = AeronConfig::default_udp()?;
//         let components = PublicationComponents::register_with_media_driver(config).unwrap();
//         let mut manager = AeronPublicationManager::new();
//
//         manager.add_publication(components.publication.clone());
//
//         let (place_holder_sender, place_holder_receiver) = mpsc::channel::<AeronMessage>(256);
//
//         let ap = AeronPublisher::new(place_holder_receiver, Arc::new(Mutex::new(components)))?;
//
//         let publisher = ap.publisher.lock().unwrap();
//         publisher.publish(capnp_msg).unwrap();
//
//         Ok(())
//     }
//
//     #[test]
//     fn aeron_udp_publisher() -> Result<()> {
//         let messages = generate_test_messages(10);
//         messages.iter().map(|x| x.to_string());
//
//         let config = AeronConfig::default_udp()?;
//
//         let mut context = Context::new();
//         setup_aeron_publisher_context(&mut context);
//
//         let mut aeron = Aeron::new(context).unwrap();
//
//         let pub_id = aeron
//             .add_publication(
//                 str_to_c(&config.channel).expect("test_aeron_ipc str_to_c conversion error"),
//                 config.stream_id,
//             )
//             .expect("unable to add publication to aeron");
//
//         let mut publication = aeron.find_publication(pub_id);
//
//         while publication.is_err() {
//             std::thread::yield_now();
//             publication = aeron.find_publication(pub_id);
//         }
//
//         let publication = publication.unwrap();
//
//         for &msg in &messages {
//             test_send_message(publication.clone(), msg.to_string());
//         }
//         Ok(())
//
//         // assert_eq!(receive_messages(&publication, messages.len()), messages);
//     }
//     #[test]
//     fn aeron_ipc_publisher() -> Result<()> {
//         let messages = generate_test_messages(10);
//         messages.iter().map(|x| x.to_string());
//
//         let config = AeronConfig::default_ipc()?;
//
//         let mut context = Context::new();
//         setup_aeron_publisher_context(&mut context);
//
//         let mut aeron = Aeron::new(context).unwrap();
//
//         let pub_id = aeron
//             .add_publication(
//                 str_to_c(&config.channel).expect("test_aeron_ipc str_to_c conversion error"),
//                 config.stream_id,
//             )
//             .expect("unable to add publication to aeron");
//
//         let mut publication = aeron.find_publication(pub_id);
//
//         while publication.is_err() {
//             std::thread::yield_now();
//             publication = aeron.find_publication(pub_id);
//         }
//
//         let publication = publication.unwrap();
//
//         for &msg in &messages {
//             test_send_message(publication.clone(), msg.to_string());
//         }
//         Ok(())
//
//         // assert_eq!(receive_messages(&publication, messages.len()), messages);
//     }
//     //NOTE: tmp
//     fn generate_test_messages(count: usize) -> Vec<i32> {
//         let mut rng = thread_rng();
//         let range = Uniform::new_inclusive(1, 1000);
//
//         (0..count).map(|_| rng.sample(&range)).collect()
//     }
//     //NOTE: tmp
//     fn test_send_message(publication: Arc<Mutex<Publication>>, msg: String) {
//         let str_msg = format!("{}", msg);
//         let c_str_msg = CString::new(str_msg).unwrap();
//         let buffer = AlignedBuffer::with_capacity(256);
//         let src_buffer = AtomicBuffer::from_aligned(&buffer);
//
//         src_buffer.put_bytes(0, c_str_msg.as_bytes());
//
//         publication
//             .lock()
//             .unwrap()
//             .offer_part(src_buffer, 0, c_str_msg.as_bytes().len() as i32);
//     }
//
//     fn available_image_handler(image: &Image) {
//         println!(
//             "Available image correlation_id={} session_id={} at position={} from {}",
//             image.correlation_id(),
//             image.session_id(),
//             image.position(),
//             image.source_identity().to_str().unwrap()
//         );
//     }
//
//     fn unavailable_image_handler(image: &Image) {
//         println!(
//             "Unavailable image correlation_id={} session_id={} at position={} from {}",
//             image.correlation_id(),
//             image.session_id(),
//             image.position(),
//             image.source_identity().to_str().unwrap()
//         );
//     }
//     fn on_new_fragment(buffer: &AtomicBuffer, offset: Index, length: Index, header: &Header) {
//         unsafe {
//             let slice_msg =
//                 slice::from_raw_parts_mut(buffer.buffer().offset(offset as isize), length as usize);
//             let msg = CString::new(slice_msg).unwrap();
//             println!(
//                 "Message to stream {} from session {} ({}@{}): <<{}>>",
//                 header.stream_id(),
//                 header.session_id(),
//                 length,
//                 offset,
//                 msg.to_str().unwrap()
//             );
//         }
//     }
//
//     pub fn setup_aeron_publisher_context(context: &mut Context) {
//         context.set_new_publication_handler(Box::new(on_new_publication_handler));
//         context.set_error_handler(Box::new(error_handler));
//         context.set_pre_touch_mapped_memory(true);
//     }
// }
