use super::monitoring::{PublisherMetrics, RegistrationId, SubscriberMetrics};
use std::collections::HashMap;
/*

NOTE: analytics module builds api to pass metrics from monitors (?), for each counter/publisher and media driver, that can then be passed to grafana, prometheus, etc

*/

#[derive(Clone)]
pub struct AeronAnalytics {
    place_holder: String,
}

impl AeronAnalytics {
    fn new() -> Self {
        Self {
            place_holder: String::new(),
        }
    }
}

//TODO: Finish PubSubMetrics/methods, Figure out ClusterMetric representation, AeronMonitor metric
// evaluation and calculation methods, state transition logic relative to metrics

#[derive(Debug)]
pub enum MetricCluster {
    Publisher(PublisherMetrics),
    Subscriber(SubscriberMetrics),
    Composite(CompositeCluster), // user/config generated String identifier for cluster type
}

#[derive(Debug)]
pub enum CompositeCluster {
    PublisherCluster(PublisherMetrics),
    SubscriberCluster(SubscriberMetrics),
}

pub struct ClusterMetrics {
    identifier: String,
    metrics: HashMap<RegistrationId, MetricCluster>,
}
