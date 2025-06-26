use nvim_oxi::{
    Dictionary,
    api::{self, types},
};
use std::sync::atomic::Ordering;

use crate::{
    buffer::{self, find_buffer_by_path},
    code_extractor::extract_code_blocks,
    error::{Error, Result},
    state::{FIRST_ROUND_ATTACHED, INITIAL_INSTRUCTION, STATE},
};
pub fn chat(message_parts: Vec<String>) -> Result<()> {
    let state = STATE.lock()?;
    let mut message = message_parts.join(" ");

    let is_first_message = !FIRST_ROUND_ATTACHED.load(Ordering::SeqCst);

    if is_first_message {
        match buffer::attach() {
            Ok(_) => {
                FIRST_ROUND_ATTACHED.store(true, Ordering::SeqCst);
                api::notify(
                    "Augment chat buffer attached.",
                    types::LogLevel::Info,
                    &Dictionary::default(),
                )?;
            }
            Err(e) => {
                api::notify(
                    &format!("Error attaching buffer: {}", e),
                    types::LogLevel::Error,
                    &Dictionary::default(),
                )?;
                return Err(e);
            }
        }

        message = format!("{} {}", INITIAL_INSTRUCTION, message);
    }

    let escaped_message = shellescape(&message, true);
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
            &Dictionary::default(),
        )?;
        return Ok(());
    }

    let mut applied_any_changes = false;
    let parsed_content = extract_code_blocks(&current_content);

    for block in parsed_content {
        if block.mode.as_deref() == Some("EDIT") {
            if let Some(path) = block.path {
                let mut buffer = find_buffer_by_path(&path)?;
                let lines: Vec<_> = block.code.split('\n').collect();
                buffer.set_lines(.., true, lines)?;
                applied_any_changes = true;
            } else {
                return Err(Error::NoPathToEdit);
            }
        }
    }

    if applied_any_changes {
        state.current_turn_lines.clear();
        api::notify(
            "Code blocks processed and chat history cleared.",
            types::LogLevel::Info,
            &Dictionary::default(),
        )?;
    } else {
        api::notify(
            "No 'EDIT' code blocks with paths found to apply.",
            types::LogLevel::Info,
            &Dictionary::default(),
        )?;
    }

    Ok(())
}

pub fn clear_chat() -> Result<()> {
    api::command("Augment chat-new")?;

    let mut state = STATE.lock()?;

    state.current_turn_lines.clear();
    state.is_attached = false;
    FIRST_ROUND_ATTACHED.store(false, Ordering::SeqCst);

    Ok(())
}

fn shellescape(s: &str, special: bool) -> String {
    let mut escaped = String::with_capacity(s.len() + 2);
    escaped.push('\'');
    escaped.push_str(&s.replace('\'', "'\\''"));
    escaped.push('\'');

    if special {
        // In Vim's shellescape(), when {special} is true, it escapes characters
        // that are special in a shell command line.
        // For this use case, we are primarily concerned with `!` being replaced
        // by `\!` and `%` by `\%`. Other special characters might need handling if the use
        // case expands.
        escaped = escaped.replace("!", "\\!");
        escaped = escaped.replace("%", "\\%");
    }

    escaped
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shellescape_simple() {
        assert_eq!(shellescape("hello world", false), "'hello world'");
    }

    #[test]
    fn test_shellescape_with_quote() {
        assert_eq!(shellescape("it's a trap", false), "'it'\\''s a trap'");
    }

    #[test]
    fn test_shellescape_with_special_chars() {
        assert_eq!(shellescape("special!", true), "'special\\!'");
    }

    #[test]
    fn test_shellescape_with_special_chars_and_quote() {
        assert_eq!(shellescape("it's special!", true), "'it'\\''s special\\!'");
    }

    #[test]
    fn test_shellescape_empty_string() {
        assert_eq!(shellescape("", false), "''");
    }
}