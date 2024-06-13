use crate::transport::{
    network::{
        aeron::{
            analytics::AeronAnalytics,
            publisher::AeronConfig,
            test_config::{
                CounterTypeId, DEFAULT_KEY_BUFFER, DEFAULT_NUM_COUNTERS, METADATA_ENTRY_SIZE,
            },
        },
        error::NetworkCommunicationError,
    },
    publisher_traits::NetworkActorLifecycle,
};

use aeron_rs::{
    aeron::Aeron,
    client_conductor::ClientConductor,
    cnc_file_descriptor,
    concurrent::{
        atomic_buffer::AtomicBuffer,
        broadcast::{
            broadcast_receiver::BroadcastReceiver, copy_broadcast_receiver::CopyBroadcastReceiver,
        },
        counters::CountersManager,
        ring_buffer::ManyToOneRingBuffer,
    },
    context::Context,
    counter::Counter,
    driver_proxy::DriverProxy,
    utils::{errors::AeronError, misc::unix_time_ms},
};
use std::collections::HashMap;
use std::sync::{
    atomic::{AtomicU64, Ordering},
    Arc, Mutex,
};

/*

 NOTE: monitoring module handles interaction between Counters, streams and automating lifecycle relative to stream/publisher state (derived from counters)

 NOTE: Counter monitoring methods inherited from aeron_rs::atomic_counter::AtomicCounter
 (atomic_counter field on Counter) -> all non-lifecycle management should run through
 AtomicCounter methods, see aeron_rs src code.


*/

//TODO: Build analytics integrations around above considerations and AtomicCounter methods

type RegistrationId = i64;
type Index = i32;

pub struct AeronCounterManager {
    conductor: Arc<Mutex<ClientConductor>>,
    // monitor: AeronMonitor,
    counters: HashMap<RegistrationId, AeronMonitor>,
    analytics: HashMap<RegistrationId, AeronAnalytics>,
}

impl AeronCounterManager {
    pub fn new(context: &Context /* , monitor: AeronMonitor */) -> Result<Self, AeronError> {
        let cnc_buf = Aeron::map_cnc_file(&context)?;

        let local_driver_proxy = Arc::new(DriverProxy::new(Arc::new(ManyToOneRingBuffer::new(
            cnc_file_descriptor::create_to_driver_buffer(&cnc_buf),
        )?)));

        let local_conductor = ClientConductor::new(
            unix_time_ms,
            local_driver_proxy.clone(),
            Arc::new(Mutex::new(CopyBroadcastReceiver::new(Arc::new(
                Mutex::new(BroadcastReceiver::new(
                    cnc_file_descriptor::create_to_clients_buffer(&cnc_buf),
                )?),
            )))),
            cnc_file_descriptor::create_counter_metadata_buffer(&cnc_buf),
            cnc_file_descriptor::create_counter_values_buffer(&cnc_buf),
            context.new_publication_handler(),
            context.new_exclusive_publication_handler(),
            context.new_subscription_handler(),
            context.error_handler(),
            context.available_counter_handler(),
            context.unavailable_counter_handler(),
            context.close_client_handler(),
            context.media_driver_timeout(),
            context.resource_linger_timeout(),
            cnc_file_descriptor::client_liveness_timeout(&cnc_buf) as u64,
            context.pre_touch_mapped_memory(),
        );

        Ok(Self {
            conductor: local_conductor,
            counters: HashMap::new(),
            analytics: HashMap::new(),
        })
    }

    // pub fn default_add_counter(
    //     &mut self,
    //     type_id: CounterTypeId,
    //     label: &str,
    // ) -> Result<(), AeronError> {
    //     let registration_id = self.conductor.lock().unwrap().add_counter(
    //         type_id as i32,
    //         DEFAULT_KEY_BUFFER,
    //         label,
    //     )?;
    //
    //     let counter = self
    //         .conductor
    //         .lock()
    //         .unwrap()
    //         .find_counter(registration_id)?;
    //
    //     self.counters.insert(registration_id, counter);
    //     Ok(())
    // }

    pub fn insert_monitor(&mut self, registration_id: RegistrationId, monitor: AeronMonitor) {
        self.counters.insert(registration_id, monitor.into());
    }

    pub fn release_counter(&mut self, registration_id: RegistrationId) -> Result<(), AeronError> {
        self.conductor
            .lock()
            .unwrap()
            .release_counter(registration_id)?;

        Ok(())
    }
}

pub enum AeronComponentType {
    Generic,
    Publisher,
    Subscriber,
    Counter,
}

pub enum AeronState {
    Healthy(HealthyState),
    Unhealthy(UnhealthyStates),
    Uninitialized,
    Inactive(InactiveState),
    Error(ErrorState),
}

impl AeronState {
    pub fn default() -> Self {
        AeronState::Uninitialized
    }
}

pub enum ActiveState {
    Healthy(HealthyState),
    Unhealthy(UnhealthyStates),
}

pub enum HealthyState {
    Active,
    Connecting,
    Reconnecting,
    Acknowledged,
    FlowControl,
}

pub enum UnhealthyStates {
    SlowConsumer,
    HighLatency,
    BufferOverflow,
    DroppedMessages,
    ErrorRateThreshold,
    Degraded,
}

pub enum InactiveState {
    Idle,
    Paused,
    Draining,
    TaskComplete,
    Unresponsive,
    AwaitingResources,
    //todo
}

pub enum ErrorState {
    Disconnected,
    Timeout,
    ConfigError,
    TranportError,
    ProtocolError,
    AuthError,
    PermissionDenied,
    ResourcesExhausted,
}

/*

impl NetworkActorLifecycle for Counter {
    type Config = AeronConfig;
    type Error = NetworkCommunicationError;

    fn initialize(&mut self, config: Self::Config) -> Result<(), Self::Error> {
        todo!();
    }
    fn start(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
    fn stop(&mut self) -> Result<(), Self::Error> {
        if !self.is_closed() {
            self.close();
        }
        Ok(())
    }
    fn restart(&mut self) -> Result<(), Self::Error> {
        todo!();
    }
}
*/
//NOTE: Temp Architecture for monitoring/analytics. may change later

// Counter lifecycle/state monitoring
// #[derive(Clone)]
struct AeronMonitor {
    // manager: Box<AeronCounterManager>,
    id: RegistrationId,
    counter: Counter,
    component_type: AeronComponentType,
    state: AeronState,
    metrics: AeronMetrics,
}

impl AeronMonitor {
    fn new(id: RegistrationId, counter: Counter, component_type: AeronComponentType) -> Self {
        Self {
            id,
            counter,
            component_type,
            state: AeronState::default(),
            metrics: AeronMetrics::new(),
        }
    }
}

// NOTE: Some metrics will require req/res, or message acknowledgement, between publisher and subscriber
// NOTE: Maybe this should go in analytics with
pub struct AeronMetrics {
    pub publisher_id: RegistrationId,
    pub published_message_count: AtomicU64,
    pub throughput: AtomicU64,
    pub latency: AtomicU64,
    pub error_rate: AtomicU64,
    pub drop_rate: AtomicU64,
    pub queue_length: AtomicU64,
    // Subscriber only
    pub processed_message_count: Option<AtomicU64>,
    // Subscriber only
    pub processing_time: Option<AtomicU64>,
    pub buffer_utilization: AtomicU64,
    pub reconnection_attempts: AtomicU64,
    pub cpu_usage: AtomicU64, // ?
    pub memory_usage: AtomicU64,
    pub jitter: AtomicU64,
    // Publisher
    pub serialization_time: Option<AtomicU64>,
    // Subscriber only
    pub deserialization_time: Option<AtomicU64>,
    pub network_bandwidth: AtomicU64,
    pub network_packet_loss: AtomicU64,
    pub network_rtt: AtomicU64,
    pub message_size_distribution: Vec<AtomicU64>,
}

impl AeronMetrics {
    pub fn new() -> Self {
        todo!();
    }
    pub fn increment_published_message_count(&self) {
        self.published_message_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_err(&self) {
        self.error_rate.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_drop(&self) {
        self.drop_rate.fetch_add(1, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod test {
    use super::{AeronCounterManager, AeronMonitor};
    use aeron_rs::context::Context;
    use eyre::Result;

    #[test]
    fn aeron_monitor_conductor() -> Result<()> {
        let mut context = Context::new();
        // todo!();
        // let counter = AeronCounterManager::new(&context)?;
        // let aeron_monitor = AeronMonitor::new(counter.clone());
        //
        // counter.conductor.lock().unwrap().ensure_open()?;
        //
        Ok(())
    }
}
