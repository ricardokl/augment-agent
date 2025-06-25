use nvim_oxi::api::Buffer;
use once_cell::sync::Lazy;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

/// Initial instruction prepended to the first chat message.
pub const INITIAL_INSTRUCTION: &str =
    "Always output full files, never just partial code. Never omit code, even if it is unchanged";

/// Mutable plugin state.
#[derive(Debug, Default)]
pub struct State {
    /// The buffer attached to the Augment chat.
    pub aug_buf: Option<Buffer>,
    /// The lines of the current turn in the chat.
    pub current_turn_lines: Vec<String>,
    /// Whether we are currently attached to the buffer.
    pub is_attached: bool,
}

// Removed: pub static CONFIG: Lazy<Config> = Lazy::new(Config::default);
pub static STATE: Lazy<Mutex<State>> = Lazy::new(|| Mutex::new(State::default()));
pub static FIRST_ROUND_ATTACHED: Lazy<AtomicBool> = Lazy::new(|| AtomicBool::new(false));
