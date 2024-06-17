use crate::{
    error::NetworkCommunicationError, publisher::AeronConfig,
    system_management::analytics::AeronAnalytics, system_management::state::AeronState,
    test_config::*, traits::NetworkActorLifecycle,
};
use aeron_rs::{
    aeron::Aeron,
    client_conductor::ClientConductor,
    cnc_file_descriptor,
    concurrent::{
        broadcast::{
            broadcast_receiver::BroadcastReceiver, copy_broadcast_receiver::CopyBroadcastReceiver,
        },
        ring_buffer::ManyToOneRingBuffer,
    },
    context::Context,
    counter::Counter,
    driver_proxy::DriverProxy,
    utils::{errors::AeronError, misc::unix_time_ms},
};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::sync::{
    atomic::{AtomicU16, AtomicU64, Ordering},
    Arc, Mutex,
};

//TODO write methods to update stored metrics, write methods to generate/calculate metrics from Counter values,  write methods to update state from
// metrics, write methods to manage driver from state, lifecycle after metrics/monitoring
// TODO: Consider concurrency for counters, metrics, analytics. initial thoughts are spawning
// isolated threads (possibly 4) running metrics calculations on a single isolated thread, analytic pushes to
// grafana/prom on anoter thread, and pub/sub/counter increments and updates split across two. this
// should limit critical sections
// TODO: Consider SoA refactor for cache optimized component metrics.

/*

 NOTE: Counter monitoring methods inherited from aeron_rs::atomic_counter::AtomicCounter
 (atomic_counter field on Counter)

*/

pub type RegistrationId = i64;
type Index = i32;

pub struct AeronCounterManager {
    conductor: Arc<Mutex<ClientConductor>>,
    counters: HashMap<RegistrationId, Counter>,
    monitors: HashMap<RegistrationId, AeronMonitor>,
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
            monitors: HashMap::new(),
            analytics: HashMap::new(),
        })
    }

    // pub fn default_add_counter(
    //
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
        self.monitors.insert(registration_id, monitor.into());
    }

    pub fn release_counter(&mut self, registration_id: RegistrationId) -> Result<(), AeronError> {
        self.conductor
            .lock()
            .unwrap()
            .release_counter(registration_id)?;

        Ok(())
    }
}

struct AeronMonitor {
    id: RegistrationId,
    counter: Counter,
    component_type: AeronComponentTypes,
    state: AeronState,
    metrics: AeronMetrics,
}

impl AeronMonitor {
    fn new(id: RegistrationId, counter: Counter, component_type: AeronComponentTypes) -> Self {
        Self {
            id,
            counter,
            component_type: component_type.clone(),
            state: AeronState::default(),
            metrics: AeronComponentTypes::generate_metrics(&component_type, id),
        }
    }
}

#[derive(Debug, Clone)]
enum AeronComponentTypes {
    Publisher,
    Subscriber,
    Driver,
}

impl AeronComponentTypes {
    pub fn generate_metrics(&self, id: RegistrationId) -> AeronMetrics {
        match self {
            AeronComponentTypes::Publisher => AeronMetrics::Publisher(PublisherMetrics::new(id)),
            AeronComponentTypes::Subscriber => AeronMetrics::Subscriber(SubscriberMetrics::new(id)),
            AeronComponentTypes::Driver => AeronMetrics::Driver(DriverMetrics::new()),
        }
    }
}

pub enum AeronMetrics {
    Publisher(PublisherMetrics),
    Subscriber(SubscriberMetrics),
    Driver(DriverMetrics),
}

// NOTE: Some metrics will require req/res, or message acknowledgement, between publisher and subscriber

#[derive(Debug)]
pub struct PublisherMetrics {
    pub id: RegistrationId,
    pub published_index: AtomicU64,
    pub throughput: AtomicU64,
    pub error_rate: AtomicU64,
    pub drop_rate: AtomicU64,
    pub queue_length: AtomicU64,
    pub buffer_utilization: AtomicU64,
    pub reconnection_attempts: AtomicU64,
    pub cpu_usage: AtomicU64,
    pub memory_usage: AtomicU64,
    pub jitter: AtomicU64,
    pub serialization_time: Option<AtomicU64>,
    pub network_bandwidth: AtomicU64,
    pub network_packet_loss: AtomicU64,
    pub network_rtt: AtomicU64,
    pub message_size_distribution: Vec<AtomicU64>,
}

impl PublisherMetrics {
    pub fn new(id: RegistrationId) -> Self {
        Self {
            id,
            published_index: AtomicU64::new(0),
            throughput: AtomicU64::new(0),
            error_rate: AtomicU64::new(0),
            drop_rate: AtomicU64::new(0),
            queue_length: AtomicU64::new(0),
            buffer_utilization: AtomicU64::new(0),
            reconnection_attempts: AtomicU64::new(0),
            cpu_usage: AtomicU64::new(0),
            memory_usage: AtomicU64::new(0),
            jitter: AtomicU64::new(0),
            serialization_time: Some(AtomicU64::new(0)),
            network_bandwidth: AtomicU64::new(0),
            network_packet_loss: AtomicU64::new(0),
            network_rtt: AtomicU64::new(0),
            message_size_distribution: (0..100).map(|_| AtomicU64::new(0)).collect(),
        }
    }
    pub fn increment_published_message_index(&self) {
        self.published_index.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_err(&self) {
        self.error_rate.fetch_add(1, Ordering::Relaxed);
    }

    pub fn record_drop(&self) {
        self.drop_rate.fetch_add(1, Ordering::Relaxed);
    }
}

#[derive(Debug)]
pub struct SubscriberMetrics {
    pub id: RegistrationId,
    pub consumed_index: AtomicU64,
    pub throughput: AtomicU64,
    pub latency: AtomicU64, // time pub -> consumed
    pub error_rate: AtomicU64,
    pub drop_rate: AtomicU64,
    pub queue_length: AtomicU64,
    pub processed_message_count: AtomicU64,
    pub avg_processing_time: Option<AtomicU64>,
    pub buffer_utilization: AtomicU64,
    pub reconnection_attempts: AtomicU64,
    pub consumer_msg_lag: AtomicU64, // processed message lag producer -> consumer, i.e. num_messages not time
    pub cpu_usage: AtomicU64,
    pub memory_usage: AtomicU64,
    pub jitter: AtomicU64, // successive msg latency: prev_msg_latency - current_msg_latency
    pub deserialization_time: Option<AtomicU64>,
    pub network_bandwidth: AtomicU64,
    pub network_packet_loss: AtomicU64,
    pub network_rtt: AtomicU64,
    pub message_size_distribution: Vec<AtomicU64>,
}

impl SubscriberMetrics {
    pub fn new(id: RegistrationId) -> Self {
        Self {
            id,
            consumed_index: AtomicU64::new(0),
            throughput: AtomicU64::new(0),
            latency: AtomicU64::new(0),
            error_rate: AtomicU64::new(0),
            drop_rate: AtomicU64::new(0),
            queue_length: AtomicU64::new(0),
            processed_message_count: AtomicU64::new(0),
            avg_processing_time: Some(AtomicU64::new(0)),
            buffer_utilization: AtomicU64::new(0),
            reconnection_attempts: AtomicU64::new(0),
            consumer_msg_lag: AtomicU64::new(0),
            cpu_usage: AtomicU64::new(0),
            memory_usage: AtomicU64::new(0),
            jitter: AtomicU64::new(0),
            deserialization_time: Some(AtomicU64::new(0)),
            network_bandwidth: AtomicU64::new(0),
            network_packet_loss: AtomicU64::new(0),
            network_rtt: AtomicU64::new(0),
            message_size_distribution: (0..100).map(|_| AtomicU64::new(0)).collect(),
        }
    }
    pub fn update_consumer_lag(&self, publisher_index: u64, subscriber_index: u64) {
        let lag = publisher_index.saturating_sub(subscriber_index);
        self.consumer_msg_lag.store(lag, Ordering::SeqCst);
    }
}

pub struct DriverMetrics {
    pub active_channels: AtomicU16,
    pub active_streams: AtomicU16,
    pub active_publisher_count: AtomicU16,
    pub active_subscriber_count: AtomicU16,
    pub active_counter_count: AtomicU16,
    pub inactive_stream_count: AtomicU16,
    pub total_message_count: AtomicU64,
    pub current_throughput: AtomicU64,
    pub average_throughput: AtomicU64,
    pub total_error_count: AtomicU64,
    pub total_drop_count: AtomicU64,
    pub total_cpu_usage: AtomicU64,
    pub total_memory_usage: AtomicU64,
    // NOTE: revisit value of latency field inclusion later
}

impl DriverMetrics {
    pub fn new() -> Self {
        Self {
            active_channels: AtomicU16::new(0),
            active_streams: AtomicU16::new(0),
            active_publisher_count: AtomicU16::new(0),
            active_subscriber_count: AtomicU16::new(0),
            active_counter_count: AtomicU16::new(0),
            inactive_stream_count: AtomicU16::new(0),
            total_message_count: AtomicU64::new(0),
            current_throughput: AtomicU64::new(0),
            average_throughput: AtomicU64::new(0),
            total_error_count: AtomicU64::new(0),
            total_drop_count: AtomicU64::new(0),
            total_cpu_usage: AtomicU64::new(0),
            total_memory_usage: AtomicU64::new(0),
        }
    }

    pub fn increment_active_channels(&self) {
        self.active_channels.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_active_channels(&self) {
        self.active_channels.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_active_channels(&self, update_val: u16) {
        self.active_channels.store(update_val, Ordering::Relaxed);
    }

    pub fn increment_active_streams(&self) {
        self.active_streams.fetch_add(1, Ordering::Relaxed);
    }
    pub fn decrement_active_streams(&self) {
        self.active_streams.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_active_streams(&self, update_val: u16) {
        self.active_streams.store(update_val, Ordering::Relaxed);
    }

    pub fn increment_publisher_count(&self) {
        self.active_publisher_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_publisher_count(&self) {
        self.active_publisher_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_publisher_count(&self, update_val: u16) {
        self.active_publisher_count
            .store(update_val, Ordering::Relaxed);
    }

    pub fn increment_subscriber_count(&self) {
        self.active_subscriber_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_subscriber_count(&self) {
        self.active_subscriber_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_subscriber_count(&self, update_val: u16) {
        self.active_subscriber_count
            .store(update_val, Ordering::Relaxed);
    }

    pub fn increment_counter_count(&self) {
        self.active_counter_count.fetch_add(1, Ordering::Relaxed);
    }
    pub fn decrement_counter_count(&self) {
        self.active_counter_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_counter_count(&self, update_val: u16) {
        self.inactive_stream_count
            .store(update_val, Ordering::Relaxed);
    }

    pub fn increment_inactive_stream_count(&self) {
        self.inactive_stream_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn decrement_inactive_stream_count(&self) {
        self.inactive_stream_count.fetch_sub(1, Ordering::Relaxed);
    }

    pub fn update_inactive_stream_count(&self, update_val: u16) {
        self.inactive_stream_count
            .store(update_val, Ordering::Relaxed);
    }

    pub fn increment_message_count(&self) {
        self.total_message_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_message_count(&self, update_val: u64) {
        self.total_message_count
            .store(update_val, Ordering::Relaxed);
    }

    pub fn batch_update_message_count(&self, batch_length: u64) -> Result<(), std::io::Error> {
        self.total_message_count
            .fetch_update(Ordering::SeqCst, Ordering::SeqCst, |x| {
                Some(x + batch_length)
            })
            .map_err(|_| {
                std::io::Error::new(ErrorKind::Other, "Failed to batch update message count")
            })?;
        Ok(())
    }

    pub fn update_current_throughput(&self, update_val: u64) {
        self.current_throughput.store(update_val, Ordering::SeqCst)
    }

    pub fn update_average_current_throughput(&self, update_val: u64) {
        self.average_throughput.store(update_val, Ordering::SeqCst)
    }

    pub fn increment_error_count(&self) {
        self.total_error_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn update_error_count(&self, update_val: u64) {
        self.total_error_count.store(update_val, Ordering::Relaxed);
    }

    pub fn update_cpu_usage(&self, update_val: u64) {
        self.total_cpu_usage.store(update_val, Ordering::SeqCst);
    }
    pub fn update_memory_usage(&self, update_val: u64) {
        self.total_memory_usage.store(update_val, Ordering::SeqCst);
    }
}

#[cfg(test)]
mod test {
    use super::{AeronCounterManager, AeronMonitor};
    use aeron_rs::context::Context;
    use eyre::Result;

    #[test]
    fn aeron_monitor_conductor() -> Result<()> {
        //TODO: verify locally generated ClientConductor behaves as expected re: driver

        // let mut context = Context::new();
        // let counter = AeronCounterManager::new(&context)?;
        // let aeron_monitor = AeronMonitor::new(counter.clone());
        //
        // counter.conductor.lock().unwrap().ensure_open()?;
        //
        Ok(())
    }
}
