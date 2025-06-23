use nvim_oxi::{
    self as nvim,
    api::{self, opts, types},
};

mod autocmds;
mod buffer;
mod chat;
mod commands;
mod error;
mod lua_utils;
mod state;

#[nvim::plugin]
fn my_augment() -> nvim::Result<()> {
    let flush_opts = opts::CreateCommandOpts::default();
    let chat_opts = opts::CreateCommandOpts::builder()
        .nargs(types::CommandNArgs::Any)
        .build();
    let clear_opts = opts::CreateCommandOpts::default();

    api::create_user_command("FlushTurn", commands::flush_turn_command, &flush_opts)?;
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

    let augroup = api::create_augroup(
        "MyAugment",
        &opts::CreateAugroupOpts::builder().clear(true).build(),
    )?;

    let lsp_attach_opts = opts::CreateAutocmdOpts::builder()
        .group(augroup)
        .callback(autocmds::on_lsp_attach)
        .build();

    api::create_autocmd(["LspAttach"], &lsp_attach_opts)?;

    Ok(())
}
