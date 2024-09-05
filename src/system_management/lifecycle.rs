use crate::{
    error::*,
    publisher::{AeronConfig /* AeronPublicationManager */},
    system_management::monitoring::AeronCounterManager,
    test_config::{CounterTypeId, DEFAULT_KEY_BUFFER},
    traits::NetworkActorLifecycle,
};
use aeron_rs::{client_conductor::ClientConductor, utils::errors::AeronError};

// TODO: Refactor
pub struct MediaDriverManager {
    conductor: ClientConductor,
    // publisher_manager: AeronPublicationManager,
    counter_manager: AeronCounterManager,
}

/*
 Counter methods for monitoring/analytics
 counter: type_id = local integer associated with counter category, see
 aeron_config::CounterTypeId
 counter: label = human readable label defining counter type
*/

impl MediaDriverManager {
    fn defualt_add_counter(
        &mut self,
        type_id: CounterTypeId,
        counter_label: &str,
    ) -> Result<(), AeronError> {
        self.conductor
            .add_counter(type_id as i32, DEFAULT_KEY_BUFFER, counter_label);
        Ok(())
    }

    fn verify_active_driver(&self) -> Result<(), AeronError> {
        self.conductor.verify_driver_is_active()
    }

    fn release_publication_from_driver(&mut self, id: i32) -> Result<(), AeronError> {
        self.conductor.release_publication(id.into())
    }
}
//NOTE: Manage Publisher lifecycle components from MediaDriverManager(ClientConductor)
impl NetworkActorLifecycle for MediaDriverManager {
    type Config = AeronConfig;
    type Error = NetworkCommunicationError;

    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        todo!();
    }
    fn start(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
    fn stop(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
    fn restart(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
}
