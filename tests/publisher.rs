#[allow(warnings)]
extern crate aeron_client;

use aeron_client::{
    publisher::{AeronConfig, AeronPublisher, AeronPublisherHandler, PublicationComponents},
    test_config::*,
    traits::AeronMessage,
};
use std::sync::{Arc, Mutex};
use tokio::sync::mpsc;

use aeron_rs::{
    aeron::Aeron,
    concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer},
    context::Context,
    publication::Publication,
    utils::errors::AeronError,
};

/* Below should be moved to bin
#[derive(Debug, Clone)]
pub enum DeserializedMessage {
    NormalizedBook(NormalizedBook),
    NormalizedTrade(NormalizedTrades),
}
#[derive(Debug, Clone)]
pub struct NormalizedTrades;

#[derive(Debug, Clone)]
pub struct NormalizedBook;

#[derive(Debug, Clone)]
pub struct CapnpMessage;

impl AeronMessage for CapnpMessage {
    fn to_bytes(&self) -> Vec<u8> {
        // tmp
        todo!();
    }
}

impl AeronMessage for DeserializedMessage {
    fn to_bytes(&self) -> Vec<u8> {
        todo!();
    }
}
*/

#[derive(Clone, Debug)]
pub struct TestMessage(String);

impl AeronMessage for TestMessage {
    fn to_bytes(&self) -> Vec<u8> {
        todo!();
    }
}

#[test]
fn config_to_publish() {
    let test_msg = "Test Message".to_string();
    let msg = TestMessage(test_msg);
    let (sender, receiver) = mpsc::channel::<TestMessage>(512);
    let config = AeronConfig::default_udp().unwrap();

    let components = PublicationComponents::register_with_media_driver(config).unwrap();

    let mut publisher = AeronPublisher::new(receiver, Arc::new(Mutex::new(components))).unwrap();
    // let test = publisher.publisher().lock().unwrap(); //.publish(msg);
}
