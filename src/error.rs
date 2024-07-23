use aeron_rs::{publication::Publication, utils::errors::AeronError};
use std::fmt;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum NetworkCommunicationError {
    #[error("AeronError: {0}")]
    AeronInstanceError(#[from] AeronError),
    #[error("AeronPubliserError: {0}")]
    AeronPublisherError(#[from] PublisherError),
    #[error("Mutex poisoned: {0}")]
    PoisonError(#[from] PoisonError<MutexGuard<'static, Publication>>),
}

#[derive(Error, Debug, Clone)]
pub enum PublisherError {
    TODOAeronPublisherError,
}

impl fmt::Display for PublisherError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PublisherError::TODOAeronPublisherError => {
                todo!();
            }
            _ => {
                todo!();
            }
        }
    }
}

#[derive(Error, Debug)]
pub enum SubscriberError {
    TODOAeronSubscriberError,
}

impl fmt::Display for SubscriberError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SubscriberError::TODOAeronSubscriberError => {
                todo!();
            }
            _ => {
                todo!();
            }
        }
    }
}
