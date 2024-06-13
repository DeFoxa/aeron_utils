pub const DEFAULT_UDP_CHANNEL: &str = "aeron:udp?endpoint=localhost:40123";
pub const DEFAULT_IPC_CHANNEL: &str = "aeron:ipc";
pub const DEFAULT_UDP_STREAM_ID: &str = "1001";
pub const DEFAULT_IPC_STREAM_ID: &str = "1002";
pub const TEST_UDP_STREAM_ID: &str = "5001";
pub const TEST_IPC_STREAM_ID: &str = "5002";
pub const DEFAULT_NUMBER_OF_WARM_UP_MESSAGES: &str = "100000";
pub const DEFAULT_NUMBER_OF_MESSAGES: &str = "10000000";
pub const DEFAULT_MESSAGE_LENGTH: &str = "32";
pub const DEFAULT_LINGER_TIMEOUT_MS: &str = "0";
pub const DEFAULT_FRAGMENT_COUNT_LIMIT: &str = "10";
pub const DEFAULT_RANDOM_MESSAGE_LENGTH: bool = false;
pub const DEFAULT_PUBLICATION_RATE_PROGRESS: bool = false;

//Monitoring

pub const DEFAULT_KEY_BUFFER: &[u8] = &[0; 16];
pub const METADATA_ENTRY_SIZE: usize = 404;
pub const DEFAULT_NUM_COUNTERS: usize = 200;

#[repr(i32)]
pub enum CounterTypeId {
    NetworkMessages = 100,
    ErrorCount = 101,
    SessionActivity = 102,
}
