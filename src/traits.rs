use serde::Serialize;
use std::fmt::Debug;

pub trait NetworkActorLifecycle {
    type Config;
    type Error;

    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn restart(&mut self) -> Result<(), Self::Error>;
}

// NOTE: implement trait Publisher on aeron_rs::Publication in bin
pub trait Publisher<Message> {
    type Error;

    fn publish(&self, msg: Message) -> Result<(), Self::Error>;
}

pub trait AeronMessage: Send + Sync + Debug + Clone {
    fn to_bytes(&self) -> Vec<u8>;
}
