use nvim_oxi::api::{self, types::CommandArgs};

use crate::{chat, error::Result};

pub fn flush_turn_command(_: CommandArgs) -> Result<()> {
    chat::apply_changes()
}

pub fn my_augment_chat_command(args: CommandArgs) -> Result<()> {
    let fargs = args.fargs;
    if fargs.is_empty() {
        let input: Option<String> = api::call_function("input", ("Augment Chat: ",))?;
        if let Some(input) = input {
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
