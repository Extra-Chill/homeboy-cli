use clap::{Args, Subcommand};
use serde::Serialize;
use std::collections::HashMap;

use homeboy::auth;

use crate::tty::{prompt, prompt_password};
use super::{CmdResult, GlobalArgs};

#[derive(Args)]
pub struct AuthArgs {
    #[command(subcommand)]
    command: AuthCommand,
}

#[derive(Subcommand)]
enum AuthCommand {
    /// Authenticate with a project's API
    Login {
        /// Project ID
        #[arg(long)]
        project: String,

        /// Username or email
        #[arg(long)]
        identifier: Option<String>,

        /// Password (or read from stdin)
        #[arg(long)]
        password: Option<String>,
    },

    /// Clear stored authentication for a project
    Logout {
        /// Project ID
        #[arg(long)]
        project: String,
    },

    /// Show authentication status for a project
    Status {
        /// Project ID
        #[arg(long)]
        project: String,
    },
}

#[derive(Serialize)]
#[serde(tag = "command", rename_all = "camelCase")]
pub enum AuthOutput {
    Login { project_id: String, success: bool },
    Logout { project_id: String },
    Status { project_id: String, authenticated: bool },
}

pub fn run(args: AuthArgs, _global: &GlobalArgs) -> CmdResult<AuthOutput> {
    match args.command {
        AuthCommand::Login {
            project,
            identifier,
            password,
        } => run_login(&project, identifier, password),
        AuthCommand::Logout { project } => run_logout(&project),
        AuthCommand::Status { project } => run_status(&project),
    }
}

fn run_login(
    project_id: &str,
    identifier: Option<String>,
    password: Option<String>,
) -> CmdResult<AuthOutput> {
    // CLI handles prompting - core just receives credentials
    let identifier = match identifier {
        Some(id) => id,
        None => prompt("Username/Email: ")?,
    };

    let password = match password {
        Some(pw) => pw,
        None => prompt_password("Password: ")?,
    };

    // Build credentials map
    let mut credentials = HashMap::new();
    credentials.insert("identifier".to_string(), identifier);
    credentials.insert("password".to_string(), password);

    // Call core auth module
    let result = auth::login(project_id, credentials)?;

    Ok((
        AuthOutput::Login {
            project_id: result.project_id,
            success: result.success,
        },
        0,
    ))
}

fn run_logout(project_id: &str) -> CmdResult<AuthOutput> {
    auth::logout(project_id)?;

    Ok((
        AuthOutput::Logout {
            project_id: project_id.to_string(),
        },
        0,
    ))
}

fn run_status(project_id: &str) -> CmdResult<AuthOutput> {
    let status = auth::status(project_id)?;

    Ok((
        AuthOutput::Status {
            project_id: status.project_id,
            authenticated: status.authenticated,
        },
        0,
    ))
}
