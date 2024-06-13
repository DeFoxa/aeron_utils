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
