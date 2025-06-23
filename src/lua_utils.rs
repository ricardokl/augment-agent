use crate::error::Result;
use nvim_oxi::{Buffer, api, mlua};

/// A generic helper to call a function in `vim.fn`.
fn call_fn<A, R>(func_name: &str, args: A) -> mlua::Result<R>
where
    A: mlua::prelude::LuaMultiValue<'static>,
    R: mlua::prelude::FromLuaMulti<'static>,
{
    mlua::with(|lua| {
        let vim_fn: mlua::Table = lua.globals().get("vim")?.get("fn")?;
        let func: mlua::Function = vim_fn.get(func_name)?;
        func.call(args)
    })
}

// Replaces `vim.fn.shellescape`.
pub fn shellescape(s: &str, is_special: bool) -> Result<String> {
    Ok(call_fn("shellescape", (s, is_special))?)
}

// Replaces `vim.fn.input`.
pub fn input(opts: &api::opts::InputOpts) -> Result<Option<String>> {
    Ok(call_fn("input", opts.clone())?)
}

// Replaces `vim.lsp.get_client_by_id` to get the root directory.
pub fn get_lsp_root_dir(client_id: u32) -> Result<Option<String>> {
    let result: mlua::Result<Option<String>> = mlua::with(|lua| {
        let get_client: mlua::Function = lua
            .globals()
            .get("vim")?
            .get("lsp")?
            .get("get_client_by_id")?;
        let client: mlua::Table = get_client.call(client_id)?;
        let config: mlua::Table = client.get("config")?;
        Ok(config.get("root_dir")?)
    });
    Ok(result?)
}

// Replaces `api::buf_detach` since it's not in the Rust API.
pub fn buf_detach(buffer: &Buffer) -> Result<()> {
    mlua::with(|lua| {
        let vim_api: mlua::Table = lua.globals().get("vim")?.get("api")?;
        let detach_fn: mlua::Function = vim_api.get("nvim_buf_detach")?;
        detach_fn.call((buffer.clone(),))?;
        Ok(())
    })?;
    Ok(())
}

// Uses `vim.defer_fn` as a replacement for a timer.
pub fn defer_fn<F>(callback: F, timeout: u64) -> Result<()>
where
    F: FnOnce(Vec<()>) -> nvim_oxi::Result<bool> + 'static,
{
    let callback = nvim_oxi::Function::from_fn_once(callback);
    call_fn::<_, ()>("defer_fn", (callback, timeout))?;
    Ok(())
}
