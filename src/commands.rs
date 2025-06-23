use nvim_oxi::api::{opts::InputOpts, types::CommandArgs};

use crate::{chat, error::Result, lua_utils};

pub fn flush_turn_command(_: CommandArgs) -> Result<()> {
    chat::flush_current_turn()
}

pub fn my_augment_chat_command(args: CommandArgs) -> Result<()> {
    let fargs = args.fargs;
    if fargs.is_empty() {
        let input_opts = InputOpts::builder().prompt("Augment Chat: ").build();
        if let Some(input) = lua_utils::input(&input_opts)? {
            if !input.is_empty() {
                chat::chat(vec![input])?;
            }
        }
    } else {
        chat::chat(fargs)?;
    }
    Ok(())
}

pub fn my_augment_chat_clear_command(_: CommandArgs) -> Result<()> {
    chat::clear_chat()
}
