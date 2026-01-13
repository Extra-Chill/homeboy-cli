use clap::Args;

use crate::docs;

use super::CmdResult;

#[derive(Args)]
pub struct InitArgs {}

pub fn run_markdown(_args: InitArgs) -> CmdResult<String> {
    let topic = vec!["commands/homeboy-init".to_string()];
    let resolved = docs::resolve(&topic);

    if resolved.content.is_empty() {
        return Err(homeboy::Error::config_missing_key(
            "docs.commands/homeboy-init",
            None,
        ));
    }

    Ok((resolved.content, 0))
}
