use clap::{Args, Subcommand};
use serde::Serialize;

use homeboy::git::{self, GitOutput};
use homeboy::BulkResult;

use crate::commands::version;

use super::CmdResult;

#[derive(Args)]
pub struct GitArgs {
    #[command(subcommand)]
    command: GitCommand,
}

#[derive(Subcommand)]
enum GitCommand {
    /// Show git status for a component
    Status {
        /// Use current working directory (ad-hoc mode)
        #[arg(long)]
        cwd: bool,

        /// JSON input spec for bulk operations.
        /// Use "-" for stdin, "@file.json" for file, or inline JSON string.
        #[arg(long)]
        json: Option<String>,

        /// Component ID (non-JSON mode)
        component_id: Option<String>,
    },
    /// Stage all changes and commit
    Commit {
        /// Use current working directory (ad-hoc mode)
        #[arg(long)]
        cwd: bool,

        /// JSON input spec for bulk operations.
        /// Use "-" for stdin, "@file.json" for file, or inline JSON string.
        #[arg(long)]
        json: Option<String>,

        /// Component ID (non-JSON mode)
        component_id: Option<String>,

        /// Commit message
        message: Option<String>,
    },
    /// Push local commits to remote
    Push {
        /// Use current working directory (ad-hoc mode)
        #[arg(long)]
        cwd: bool,

        /// JSON input spec for bulk operations.
        /// Use "-" for stdin, "@file.json" for file, or inline JSON string.
        #[arg(long)]
        json: Option<String>,

        /// Component ID (non-JSON mode)
        component_id: Option<String>,

        /// Push tags as well
        #[arg(long)]
        tags: bool,
    },
    /// Pull remote changes
    Pull {
        /// Use current working directory (ad-hoc mode)
        #[arg(long)]
        cwd: bool,

        /// JSON input spec for bulk operations.
        /// Use "-" for stdin, "@file.json" for file, or inline JSON string.
        #[arg(long)]
        json: Option<String>,

        /// Component ID (non-JSON mode)
        component_id: Option<String>,
    },
    /// Create a git tag
    Tag {
        /// Use current working directory (ad-hoc mode)
        #[arg(long)]
        cwd: bool,

        /// Component ID
        component_id: Option<String>,

        /// Tag name (e.g., v0.1.2)
        ///
        /// Required when using --cwd. Otherwise defaults to v<component version>.
        tag_name: Option<String>,

        /// Tag message (creates annotated tag)
        #[arg(short, long)]
        message: Option<String>,
    },
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum GitCommandOutput {
    Single(GitOutput),
    Bulk(BulkResult<GitOutput>),
}

pub fn run(args: GitArgs, _global: &crate::commands::GlobalArgs) -> CmdResult<GitCommandOutput> {
    match args.command {
        GitCommand::Status {
            cwd,
            json,
            component_id,
        } => {
            if let Some(spec) = json {
                let output = git::status_bulk(&spec)?;
                let exit_code = if output.summary.failed > 0 { 1 } else { 0 };
                return Ok((GitCommandOutput::Bulk(output), exit_code));
            }

            // --cwd or component_id (None = CWD)
            let target = if cwd { None } else { component_id.as_deref() };
            let output = git::status(target)?;
            let exit_code = output.exit_code;
            Ok((GitCommandOutput::Single(output), exit_code))
        }
        GitCommand::Commit {
            cwd,
            json,
            component_id,
            message,
        } => {
            if let Some(spec) = json {
                let output = git::commit_bulk(&spec)?;
                let exit_code = if output.summary.failed > 0 { 1 } else { 0 };
                return Ok((GitCommandOutput::Bulk(output), exit_code));
            }

            // --cwd or component_id (None = CWD), core validates message
            let target = if cwd { None } else { component_id.as_deref() };
            let output = git::commit(target, message.as_deref())?;
            let exit_code = output.exit_code;
            Ok((GitCommandOutput::Single(output), exit_code))
        }
        GitCommand::Push {
            cwd,
            json,
            component_id,
            tags,
        } => {
            if let Some(spec) = json {
                let output = git::push_bulk(&spec)?;
                let exit_code = if output.summary.failed > 0 { 1 } else { 0 };
                return Ok((GitCommandOutput::Bulk(output), exit_code));
            }

            // --cwd or component_id (None = CWD)
            let target = if cwd { None } else { component_id.as_deref() };
            let output = git::push(target, tags)?;
            let exit_code = output.exit_code;
            Ok((GitCommandOutput::Single(output), exit_code))
        }
        GitCommand::Pull {
            cwd,
            json,
            component_id,
        } => {
            if let Some(spec) = json {
                let output = git::pull_bulk(&spec)?;
                let exit_code = if output.summary.failed > 0 { 1 } else { 0 };
                return Ok((GitCommandOutput::Bulk(output), exit_code));
            }

            // --cwd or component_id (None = CWD)
            let target = if cwd { None } else { component_id.as_deref() };
            let output = git::pull(target)?;
            let exit_code = output.exit_code;
            Ok((GitCommandOutput::Single(output), exit_code))
        }
        GitCommand::Tag {
            cwd,
            component_id,
            tag_name,
            message,
        } => {
            if cwd {
                // CWD mode: core validates tag_name
                let output = git::tag(None, tag_name.as_deref(), message.as_deref())?;
                let exit_code = output.exit_code;
                return Ok((GitCommandOutput::Single(output), exit_code));
            }

            // Component mode: derive tag from version if not provided
            let final_tag = match tag_name {
                Some(name) => name,
                None => {
                    // Need component_id to look up version
                    let id = component_id.as_ref().ok_or_else(|| {
                        homeboy::Error::validation_invalid_argument(
                            "componentId",
                            "Missing componentId (required to derive tag from version)",
                            None,
                            None,
                        )
                    })?;
                    let (out, _) = version::show_version_output(id)?;
                    format!("v{}", out.version)
                }
            };

            let output = git::tag(component_id.as_deref(), Some(&final_tag), message.as_deref())?;
            let exit_code = output.exit_code;
            Ok((GitCommandOutput::Single(output), exit_code))
        }
    }
}
