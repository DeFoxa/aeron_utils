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

pub trait Publisher<M> {
    // type Message;
    type Error;

    fn publish(&self, msg: M) -> Result<(), Self::Error>;
}

pub trait AeronMessage: Send + Sync + Debug + Clone {
    fn to_bytes(&self) -> Vec<u8>;
}
// TODO: new publisher iteration for lib requires async/synch impls for generaized use, although
// wil deterimine if synch is necessary later

pub trait AsyncPublisher {
    fn publish(
        &self,
        data: &[u8],
    ) -> Pin<Box<dyn Future<Output = Result<(), PublishError>> + Send>>;
}

pub trait AsyncSubscriber {
    fn subscribe(&self) -> Pin<Box<dyn Future<Output = Result<Vec<u8>, SubscribeError>> + Send>>;
}

pub trait SyncPublisher {
    fn publish(&self, data: &[u8]) -> Result<(), PublishError>;
}

pub trait SyncSubscriber {
    fn subscribe(&self) -> Result<Vec<u8>, SubscribeError>;
}
