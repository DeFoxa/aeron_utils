use aeron_rs::{publication::Publication, utils::errors::AeronError};
use std::fmt;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkCommunicationError {
    #[error("AeronError: {0}")]
    AeronInstanceError(#[from] AeronError),
    #[error("AeronPubliserError: {0}")]
    AeronPublisherError(#[from] AeronPublisherError),
    // IPCError(IPCError),
    // TcpError(TcpError),
    #[error("Mutex poisoned: {0}")]
    PoisonError(#[from] PoisonError<MutexGuard<'static, Publication>>),
}

#[derive(Debug)]
pub enum IPCError {
    TODOIpcError,
}

#[derive(Debug)]
pub enum TcpError {
    TODOTCPErrors,
}

#[derive(Error, Debug)]
pub enum AeronPublisherError {
    TODOAeronPublisherError,
}

impl fmt::Display for AeronPublisherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AeronPublisherError::TODOAeronPublisherError => {
                todo!();
            }
            _ => {
                todo!();
            }
        }
    }
}
