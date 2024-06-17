pub trait NetworkActorLifecycle {
    type Config;
    type Error;

    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error>;
    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
    fn restart(&mut self) -> Result<(), Self::Error>;
}

pub trait Publisher<Message> {
    // type Message;
    type Error;

    fn publish(&self, msg: Message) -> Result<(), Self::Error>;
}

// TMP

pub trait Chunk {
    fn chunk_data(&self) -> Vec<Vec<u8>>;
}
