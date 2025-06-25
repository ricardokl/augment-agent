use nvim_oxi::api;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error; // Import necessary types

#[derive(Debug, Error)]
pub enum Error {
    #[error("Nvim API error: {0}")]
    Nvim(#[from] api::Error),

    #[error("Could not get a lock on the plugin state")]
    StateLock, // This variant is now superseded by StatePoisoned, but I'll keep it for now as a distinct semantic error if needed.

    #[error("Augment chat buffer not found")]
    NoAugmentBufferFound,

    #[error("Found Augment chat buffer is invalid")]
    InvalidAugmentBuffer,

    #[error("Plugin state mutex is poisoned: {0}")] // New error variant for mutex poisoning
    StatePoisoned(#[from] PoisonError<MutexGuard<'static, crate::state::State>>), // Use crate::state::State for the type

    #[error("No path to edit")]
    NoPathToEdit,

    #[error("Buffer not found: {0}")]
    BufferNotFound(String),

    #[error("Invalid buffer: {0}")]
    InvalidBuffer(String),
}

pub type Result<T> = std::result::Result<T, Error>;
