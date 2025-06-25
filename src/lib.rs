use nvim_oxi::{
    self as nvim,
    api::{self, opts, types},
};

mod buffer;
mod chat;
mod code_extractor;
mod commands;
mod error;
mod state;

#[nvim::plugin]
fn my_augment() -> nvim::Result<()> {
    let flush_opts = opts::CreateCommandOpts::default();
    let chat_opts = opts::CreateCommandOpts::builder()
        .nargs(types::CommandNArgs::Any)
        .build();
    let clear_opts = opts::CreateCommandOpts::default();

    api::create_user_command("MyAugmentApply", commands::flush_turn_command, &flush_opts)?;
    api::create_user_command(
        "MyAugmentChat",
        commands::my_augment_chat_command,
        &chat_opts,
    )?;
    api::create_user_command(
        "MyAugmentChatClear",
        commands::my_augment_chat_clear_command,
        &clear_opts,
    )?;

    Ok(())
}
