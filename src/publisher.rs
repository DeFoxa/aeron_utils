use crate::{
    error::NetworkCommunicationError,
    test_config::*,
    traits::{AeronMessage, Publisher},
    utils::*,
};

use aeron_rs::{
    aeron::Aeron,
    concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer},
    context::Context,
    publication::Publication,
    utils::errors::AeronError,
};

use eyre::Result;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task::JoinHandle};

/*

 TODO: lib redesign:
        - rip out the async actor model/methods
        - generalize and remove previous bin design's opinionated components from publisher
        - review code and think through implications of above
*/

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
unsafe impl Send for PublicationComponents {}
