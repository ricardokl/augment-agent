use nvim_oxi::api::{self, Buffer, opts, types::AutocmdCallbackArgs};

use crate::{
    error::{Error, Result},
    state::STATE,
};

pub fn find_augment_buffer() -> Result<Option<Buffer>> {
    for buf in api::list_bufs() {
        if buf.is_valid() && buf.get_name()?.ends_with("AugmentChatHistory") {
            return Ok(Some(buf));
        }
    }
    Ok(None)
}

pub fn attach_if_needed() -> Result<bool> {
    let mut state = STATE.lock().map_err(|_| Error::StateLock)?;

    if state.is_attached {
        if let Some(buf) = &state.aug_buf {
            if buf.is_valid() {
                return Ok(true);
            }
        }
        state.is_attached = false;
        state.aug_buf = None;
    }

    let buf = match find_augment_buffer()? {
        Some(b) => b,
        None => {
            state.is_attached = false;
            return Ok(false);
        }
    };

    let lines_iter = buf.get_lines(.., false)?;
    state.current_turn_lines = lines_iter.map(|s| s.to_string()).collect();

    let on_lines_callback = |args: AutocmdCallbackArgs| {
        let mut state = match STATE.lock() {
            Ok(s) => s,
            Err(_) => return Ok(false),
        };

        if !state.is_attached {
            return Ok(false);
        }

        let new_lines_iter = match args
            .buffer
            .get_lines(args.firstline..args.new_lastline, false)
        {
            Ok(iter) => iter,
            Err(_) => return Ok(false),
        };

        for (i, line_str) in new_lines_iter.enumerate() {
            let line_num = args.firstline as usize + i;
            let rust_str = line_str.to_string();
            if line_num < state.current_turn_lines.len() {
                state.current_turn_lines[line_num] = rust_str;
            } else {
                state.current_turn_lines.push(rust_str);
            }
        }

        Ok(false)
    };

    let on_detach_callback = |_| {
        if let Ok(mut state) = STATE.lock() {
            state.aug_buf = None;
            state.is_attached = false;
        }
        Ok(false)
    };

    let attach_opts = opts::BufAttachOpts::builder()
        .on_lines(on_lines_callback)
        .on_detach(on_detach_callback)
        .build();

    buf.attach(false, &attach_opts)?;

    state.aug_buf = Some(buf.clone());
    state.is_attached = true;

    Ok(true)
}
