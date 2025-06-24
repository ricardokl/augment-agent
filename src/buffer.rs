use crate::{
    error::{Error, Result},
    state::{FIRST_ROUND_ATTACHED, STATE},
};
use nvim_oxi::api::{self, Buffer, opts};
use std::sync::atomic::Ordering;

use nvim_oxi::api::types::OnLinesArgs;

pub fn find_augment_buffer() -> Result<Buffer> {
    for buf in api::list_bufs() {
        if buf.get_name()?.ends_with("AugmentChatHistory") {
            if !buf.is_valid() {
                return Err(Error::InvalidAugmentBuffer);
            }
            return Ok(buf);
        }
    }
    Err(Error::NoAugmentBufferFound)
}

pub fn attach() -> Result<()> {
    let mut state = STATE.lock()?; // This remains '?' as attach() returns our custom Result<()>

    if state.is_attached {
        if let Some(buf) = &state.aug_buf {
            if buf.is_valid() {
                return Ok(());
            }
        }
        state.is_attached = false;
        state.aug_buf = None;
    }

    let buf = find_augment_buffer()?;

    let on_lines_callback = |(_, buffer, _, firstline, _, new_lastline, _, _, _): OnLinesArgs| {
        // Use match to handle PoisonError, as this callback returns nvim_oxi::Result<bool>
        let mut state = match STATE.lock() {
            Ok(s) => s,
            Err(e) => {
                // Log the error or handle it as appropriate for a callback
                // Since we can't propagate our custom Error, return Ok(false) to signal failure.
                api::nvim_err_writeln(&format!(
                    "Error locking state in on_lines_callback: {:?}",
                    e
                ))
                .ok(); // Ignore error writing to stderr
                return Ok(false);
            }
        };

        let new_lines_iter = match buffer.get_lines(firstline..new_lastline, false) {
            Ok(iter) => iter,
            Err(e) => {
                api::nvim_err_writeln(&format!(
                    "Error getting lines in on_lines_callback: {:?}",
                    e
                ))
                .ok();
                return Ok(false);
            }
        };

        for line_str in new_lines_iter {
            state.current_turn_lines.push(line_str.to_string());
        }

        Ok(false)
    };

    let on_detach_callback = |_| {
        // Use match to handle PoisonError, as this callback returns nvim_oxi::Result<bool>
        if let Ok(mut state) = STATE.lock() {
            state.aug_buf = None;
            state.is_attached = false;
        } else {
            // Log the error if the lock fails, similar to on_lines_callback
            api::nvim_err_writeln("Error locking state in on_detach_callback").ok();
        }
        FIRST_ROUND_ATTACHED.store(false, Ordering::SeqCst);
        Ok(false)
    };

    let attach_opts = opts::BufAttachOpts::builder()
        .on_lines(on_lines_callback)
        .on_detach(on_detach_callback)
        .build();

    buf.attach(false, &attach_opts)?;

    state.aug_buf = Some(buf.clone());
    state.is_attached = true;

    Ok(())
}
