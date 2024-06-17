use crate::system_management::monitoring::RegistrationId;
use std::{
    io::ErrorKind,
    sync::atomic::{AtomicU16, AtomicU64, Ordering},
};

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
