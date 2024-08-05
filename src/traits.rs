use crate::error::{PublisherError, SubscriberError};
use eyre::Result;
use serde::Serialize;
use std::fmt::Debug;
use std::future::Future;
use std::pin::Pin;

pub trait NetworkActorLifecycle {
    type Config;
    type Error;

    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn restart(&mut self) -> Result<(), Self::Error>;
}

pub trait Publisher<M, const BUFFER_SIZE: i32> {
    type Error;

    fn publish(&self, msg: M) -> Result<(), Self::Error>;
    fn offer(&self, msg: M) -> Result<(), Self::Error>;
    fn offer_part(&self, msg: M) -> Result<(), Self::Error>;
    fn try_claim_and_commit(&self, msg: M) -> Result<(), Self::Error>; // NOTE: may change signature
                                                                       // later
}

pub trait AeronMessage: Send + Sync + Debug + Clone {
    fn to_bytes(&self) -> Vec<u8>;
}

//TODO: change naming convention
//
pub trait AsyncPublisher {
    fn publish(
        &self,
        data: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), PublisherError>> + Send>>;
}

pub trait AsyncSubscriber {
    fn subscribe(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, SubscriberError>> + Send>>;
}

pub trait SyncPublisher {
    fn publish(&self, data: &[u8]) -> Result<(), PublisherError>;
}

pub trait SyncSubscriber {
    fn subscribe(&self) -> Result<Vec<u8>, SubscriberError>;
}
