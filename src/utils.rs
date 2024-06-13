use aeron_rs::{
    client_conductor::ClientConductor,
    utils::{errors::AeronError, types::Moment},
};
use eyre::Result;
use std::ffi::CString;

type Index = i32;

pub const TERM_MIN_LENGTH: Index = 64 * 1024;
const DRIVER_TIMEOUT_MS: Moment = 10 * 1000;
const RESOURCE_LINGER_TIMEOUT_MS: Moment = 5 * 1000;
const INTER_SERVICE_TIMEOUT_NS: Moment = 5 * 1000 * 1000 * 1000;
const INTER_SERVICE_TIMEOUT_MS: Moment = INTER_SERVICE_TIMEOUT_NS / 1_000_000;
const PRE_TOUCH_MAPPED_MEMORY: bool = false;

pub fn new_local_client_conductor(
) -> Result<() /* NOTE: PlaceHolder, will return ClientConductor */, AeronError> {
    todo!();
    Ok(())
}

/* NOTE:
 Generalized implementation
 Individual variations may be necessary depending on context of usage
*/

pub fn str_to_c(val: &str) -> Result<CString> {
    Ok(CString::new(val)?)
}

pub fn error_handler(error: AeronError) {
    println!("Error: {:?}", error);
}

pub fn on_new_publication_handler(
    channel: CString,
    stream_id: i32,
    session_id: i32,
    correlation_id: i64,
) {
    println!(
        "Publication: {} {} {} {}",
        channel.to_str().unwrap(),
        stream_id,
        session_id,
        correlation_id
    );
}
