use nvim_oxi::api::Buffer;
use once_cell::sync::Lazy;
use std::sync::Mutex;

/// Static plugin configuration.
#[derive(Debug)]
pub struct Config {
    pub defer_timeout: u64,
    pub initial_instruction: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            defer_timeout: 50,
            initial_instruction: "Always output full files, never just partial code. Never omit code, even if it is unchanged".to_string(),
        }
    }
}

/// Mutable plugin state.
#[derive(Debug, Default)]
pub struct State {
    /// The buffer attached to the Augment chat.
    pub aug_buf: Option<Buffer>,
    /// The lines of the current turn in the chat.
    pub current_turn_lines: Vec<String>,
    /// Whether we are currently attached to the buffer.
    pub is_attached: bool,
    /// The current round of chat conversation.
    pub chat_round: u32,
    /// The job ID of the currently running Augment process.
    pub current_job_id: Option<i32>,
}

pub static CONFIG: Lazy<Config> = Lazy::new(Config::default);
pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::default()));
