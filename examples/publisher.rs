#![allow(warnings)]
use crate::{
    error::NetworkCommunicationError,
    test_config::*,
    traits::{AeronMessage, Publisher},
};
use aeron_rs::{
    aeron::Aeron,
    concurrent::atomic_buffer::{AlignedBuffer, AtomicBuffer},
    context::Context,
    // publication::Publication,
    utils::errors::AeronError,
};
use eyre::Result;
use std::sync::{Arc, Mutex};
use tokio::{sync::mpsc, task::JoinHandle};

pub struct AeronPublisher<M: AeronMessage> {
    receiver: mpsc::Receiver<M>,
    components: Arc<Mutex<PublicationComponents>>,
    publisher: Arc<Mutex<Publication>>,
}

impl<M: AeronMessage> AeronPublisher<M> {
    pub fn new(
        receiver: mpsc::Receiver<M>,
        components: Arc<Mutex<PublicationComponents>>,
    ) -> Result<Self> {
        let lock = components.lock().unwrap().publication.clone();

        Ok(AeronPublisher {
            receiver,
            components: Arc::clone(&components),
            publisher: lock,
        })
    }

    pub async fn handle_message(&mut self, msg: M) -> Result<()> {
        tracing::debug!("Message received by aeron publisher");
        self.publisher
            .lock()
            .unwrap()
            .publish(msg)
            .map_err(|err| NetworkCommunicationError::AeronInstanceError(err));

        Ok(())
    }

    pub fn publisher(&self) -> Arc<Mutex<Publication>> {
        Arc::clone(&self.publisher)
    }
}

pub fn main() {
    todo!();
}
