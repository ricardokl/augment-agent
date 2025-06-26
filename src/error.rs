use nvim_oxi::api;
use std::sync::{MutexGuard, PoisonError};
use thiserror::Error; // Import necessary types

#[derive(Debug, Error)]
pub enum Error {
    #[error("Nvim API error: {0}")]
    Nvim(#[from] api::Error),

    #[error("Augment chat buffer not found")]
    NoAugmentBufferFound,

    #[error("Found Augment chat buffer is invalid")]
    InvalidAugmentBuffer,

    #[error("Plugin state mutex is poisoned: {0}")]
    StatePoisoned(#[from] PoisonError<MutexGuard<'static, crate::state::State>>),

    #[error("No path to edit")]
    NoPathToEdit,

    #[error("Buffer not found: {0}")]
    BufferNotFound(String),

    #[error("Invalid buffer: {0}")]
    InvalidBuffer(String),
}

pub type Result<T> = std::result::Result<T, Error>;
