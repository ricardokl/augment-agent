use nvim_oxi::api::{self, opts, types};

use crate::{
    buffer,
    error::{Error, Result},
    lua_utils,
    state::{CONFIG, STATE},
};

pub fn chat(message_parts: Vec<String>) -> Result<()> {
    let mut state = STATE.lock().map_err(|_| Error::StateLock)?;
    let mut message = message_parts.join(" ");

    if state.chat_round == 0 {
        message = format!("{} {}", &CONFIG.initial_instruction, message);
    }

    state.chat_round += 1;
    state.current_turn_lines.clear();
    state.is_attached = false;

    let escaped_message = lua_utils::shellescape(&message, true)?;
    let cmd = format!("Augment chat {}", escaped_message);

    api::command(&cmd)?;

    let timeout = CONFIG.defer_timeout;
    lua_utils::defer_fn(
        move |_| {
            buffer::attach_if_needed()?;
            Ok(false) // don't repeat
        },
        timeout,
    )?;

    Ok(())
}

pub fn flush_current_turn() -> Result<()> {
    let mut state = STATE.lock().map_err(|_| Error::StateLock)?;

    let buf = match &state.aug_buf {
        Some(b) if b.is_valid() => b,
        _ => {
            api::notify(
                "Not attached to a valid buffer",
                types::LogLevel::Warn,
                &opts::NotifyOpts::default(),
            )?;
            return Ok(());
        }
    };

    state.current_turn_lines = buf.get_lines(.., false)?.map(|s| s.to_string()).collect();

    if state.current_turn_lines.join("\n").is_empty() {
        return Ok(());
    }

    // NOTE: This command now simply captures the buffer's content.
    // The user manually runs this when they see the AI is done.

    Ok(())
}

pub fn clear_chat() -> Result<()> {
    api::command("Augment chat-new")?;

    let mut state = STATE.lock().map_err(|_| Error::StateLock)?;

    if let Some(buf) = state.aug_buf.take() {
        if buf.is_valid() {
            let _ = lua_utils::buf_detach(&buf);
        }
    }

    state.chat_round = 0;
    state.current_turn_lines.clear();
    state.is_attached = false;

    Ok(())
}
