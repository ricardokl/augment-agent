use nvim_oxi::api;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Nvim API error: {0}")]
    Nvim(#[from] api::Error),

    #[error("Could not get a lock on the plugin state")]
    StateLock,
}

pub type Result<T> = std::result::Result<T, Error>;
