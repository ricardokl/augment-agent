use nvim_oxi::{
    Object,
    api::{self, types::AutocmdCallbackArgs},
};
use serde::Deserialize;

use crate::{error::Result, lua_utils};

#[derive(Debug, Deserialize)]
struct LspAttachData {
    client_id: u32,
}

#[derive(Debug, Deserialize)]
pub struct LspAttachArgs {
    pub data: LspAttachData,
}

pub fn on_lsp_attach(args: AutocmdCallbackArgs) -> Result<()> {
    let lsp_args = match LspAttachArgs::deserialize(Object::from(args.data)) {
        Ok(a) => a,
        Err(_) => return Ok(()), // Couldn't deserialize, so ignore.
    };

    let root_dir = match lua_utils::get_lsp_root_dir(lsp_args.data.client_id)? {
        Some(dir) => dir,
        None => return Ok(()),
    };

    let var_name = "augment_workspace_folders";
    let mut workspaces: Vec<String> = api::get_var(var_name).unwrap_or_default();

    if !workspaces.contains(&root_dir) {
        workspaces.push(root_dir);
        api::set_var(var_name, workspaces)?;
    }

    Ok(())
}
