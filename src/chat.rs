use nvim_oxi::api::{self, opts, types};
use regex::Regex;
use std::sync::atomic::Ordering;

use crate::{
    buffer,
    error::{Error, Result},
    lua_utils,
    state::{FIRST_ROUND_ATTACHED, INITIAL_INSTRUCTION, STATE},
};

pub fn chat(message_parts: Vec<String>) -> Result<()> {
    // Seamless conversion from PoisonError to Error::StatePoisoned
    let mut state = STATE.lock()?;
    let mut message = message_parts.join(" ");

    let is_first_message = !FIRST_ROUND_ATTACHED.load(Ordering::SeqCst);

    if is_first_message {
        match buffer::attach() {
            Ok(_) => {
                FIRST_ROUND_ATTACHED.store(true, Ordering::SeqCst);
                api::notify(
                    "Augment chat buffer attached.",
                    types::LogLevel::Info,
                    &opts::NotifyOpts::default(),
                )?;
            }
            Err(e) => {
                api::notify(
                    &format!("Error attaching buffer: {}", e),
                    types::LogLevel::Error,
                    &opts::NotifyOpts::default(),
                )?;
                return Err(e);
            }
        }

        message = format!("{} {}", INITIAL_INSTRUCTION, message);
    }

    let escaped_message = lua_utils::shellescape(&message, true)?;
    let cmd = format!("Augment chat {}", escaped_message);

    api::command(&cmd)?;

    Ok(())
}

pub fn apply_changes() -> Result<()> {
    let mut state = STATE.lock()?;

    let current_content = state.current_turn_lines.join("\n");
    if current_content.is_empty() {
        api::notify(
            "No content in the chat buffer to apply.",
            types::LogLevel::Info,
            &opts::NotifyOpts::default(),
        )?;
        return Ok(());
    }

    // Corrected regex: changed [\"'] to ["'] and [^\"'] to [^"']
    let code_block_regex = Regex::new(r"```(?P<lang>\w+)(?:\s+path=["'](?P<path>[^"']+)["'])?(?:\s+mode=["'](?P<mode>[^"']+)["'])?\n(?P<content>[\s\S]*?)\n```")
        .map_err(|e| Error::Nvim(api::Error::Other(format!("Regex error: {}", e))))?;


    let mut applied_any_changes = false;

    for cap in code_block_regex.captures_iter(&current_content) {
        let path = cap.name("path").map(|m| m.as_str());
        let mode = cap.name("mode").map(|m| m.as_str());
        let content = cap.name("content").map(|m| m.as_str()).unwrap_or("");

        if let (Some(file_path), Some(operation_mode)) = (path, mode) {
            if operation_mode == "EDIT" {
                let target_buf = api::list_bufs()
                    .into_iter()
                    .find(|b| b.is_valid() && b.get_name().map_or(false, |name| name.ends_with(file_path)));

                if let Some(buf) = target_buf {
                    buf.set_lines(.., true, content.lines().map(|s| s.to_string()).collect())?;
                    api::notify(
                        &format!("Applied changes to: {}", file_path),
                        types::LogLevel::Info,
                        &opts::NotifyOpts::default(),
                    )?;
                    applied_any_changes = true;
                } else {
                    api::notify(
                        &format!("Buffer not found for path: {}", file_path),
                        types::LogLevel::Warn,
                        &opts::NotifyOpts::default(),
                    )?;
                }
            } else {
                api::notify(
                    &format!("Unsupported mode '{}' for path: {}", operation_mode, file_path),
                    types::LogLevel::Warn,
                    &opts::NotifyOpts::default(),
                )?;
            }
        }
    }

    if applied_any_changes {
        state.current_turn_lines.clear();
        api::notify(
            "Code blocks processed and chat history cleared.",
            types::LogLevel::Info,
            &opts::NotifyOpts::default(),
        )?;
    } else {
        api::notify(
            "No 'EDIT' code blocks with paths found to apply.",
            types::LogLevel::Info,
            &opts::NotifyOpts::default(),
        )?;
    }

    Ok(())
}

pub fn clear_chat() -> Result<()> {
    api::command("Augment chat-new")?;

    // Seamless conversion from PoisonError to Error::StatePoisoned
    let mut state = STATE.lock()?;

    if let Some(buf) = state.aug_buf.take() {
        if buf.is_valid() {
            let _ = lua_utils::buf_detach(&buf);
        }
    }

    state.current_turn_lines.clear();
    state.is_attached = false;
    FIRST_ROUND_ATTACHED.store(false, Ordering::SeqCst);

    Ok(())
}
