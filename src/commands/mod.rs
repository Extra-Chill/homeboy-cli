use clap::Args;

pub type CmdResult<T> = homeboy::Result<(T, i32)>;

pub(crate) struct GlobalArgs {}

/// Shared arguments for dynamic set commands.
///
/// Allows arbitrary `--key value` pairs that map directly to JSON keys.
/// Flag names become JSON keys with no case conversion.
#[derive(Args, Default, Debug)]
pub struct DynamicSetArgs {
    /// Entity ID (optional if provided in JSON body)
    pub id: Option<String>,

    /// JSON spec (positional, supports @file and - for stdin)
    pub spec: Option<String>,

    /// Explicit JSON spec (takes precedence over positional)
    #[arg(long, value_name = "JSON")]
    pub json: Option<String>,

    /// Additional key=value flags (e.g., --remote-path /var/www)
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub extra: Vec<String>,
}

impl DynamicSetArgs {
    /// Get the JSON spec from either --json or positional argument
    pub fn json_spec(&self) -> Option<&str> {
        self.json.as_deref().or(self.spec.as_deref())
    }
}

pub mod api;
pub mod auth;
pub mod build;
pub mod changelog;
pub mod changes;
pub mod cli;
pub mod component;
pub mod context;
pub mod db;
pub mod deploy;
pub mod docs;
pub mod file;
pub mod git;
pub mod init;
pub mod logs;
pub mod module;
pub mod project;
pub mod server;
pub mod ssh;
pub mod upgrade;
pub mod version;

pub(crate) fn run_markdown(
    command: crate::Commands,
    _global: &GlobalArgs,
) -> homeboy::Result<(String, i32)> {
    match command {
        crate::Commands::Docs(args) => docs::run_markdown(args),
        crate::Commands::Init(args) => init::run_markdown(args),
        crate::Commands::Changelog(args) => changelog::run_markdown(args),
        _ => Err(homeboy::Error::validation_invalid_argument(
            "output_mode",
            "Command does not support markdown output",
            None,
            None,
        )),
    }
}

pub(crate) fn run_json(
    command: crate::Commands,
    global: &GlobalArgs,
) -> (homeboy::Result<serde_json::Value>, i32) {
    match command {
        crate::Commands::Init(_) => {
            let err = homeboy::Error::validation_invalid_argument(
                "output_mode",
                "Init command uses markdown output mode",
                None,
                None,
            );
            crate::output::map_cmd_result_to_json::<serde_json::Value>(Err(err))
        }
        crate::Commands::Project(args) => {
            crate::output::map_cmd_result_to_json(project::run(args, global))
        }
        crate::Commands::Ssh(args) => crate::output::map_cmd_result_to_json(ssh::run(args, global)),
        crate::Commands::Server(args) => {
            crate::output::map_cmd_result_to_json(server::run(args, global))
        }
        crate::Commands::Db(args) => crate::output::map_cmd_result_to_json(db::run(args, global)),
        crate::Commands::File(args) => {
            crate::output::map_cmd_result_to_json(file::run(args, global))
        }
        crate::Commands::Logs(args) => {
            crate::output::map_cmd_result_to_json(logs::run(args, global))
        }
        crate::Commands::Deploy(args) => {
            crate::output::map_cmd_result_to_json(deploy::run(args, global))
        }
        crate::Commands::Component(args) => {
            crate::output::map_cmd_result_to_json(component::run(args, global))
        }
        crate::Commands::Context(args) => {
            crate::output::map_cmd_result_to_json(context::run(args, global))
        }
        crate::Commands::Module(args) => {
            crate::output::map_cmd_result_to_json(module::run(args, global))
        }
        crate::Commands::Docs(args) => {
            crate::output::map_cmd_result_to_json(docs::run(args, global))
        }
        crate::Commands::Changelog(args) => {
            crate::output::map_cmd_result_to_json(changelog::run(args, global))
        }
        crate::Commands::Git(args) => crate::output::map_cmd_result_to_json(git::run(args, global)),
        crate::Commands::Version(args) => {
            crate::output::map_cmd_result_to_json(version::run(args, global))
        }
        crate::Commands::Build(args) => {
            crate::output::map_cmd_result_to_json(build::run(args, global))
        }
        crate::Commands::Changes(args) => {
            crate::output::map_cmd_result_to_json(changes::run(args, global))
        }
        crate::Commands::Auth(args) => {
            crate::output::map_cmd_result_to_json(auth::run(args, global))
        }
        crate::Commands::Api(args) => crate::output::map_cmd_result_to_json(api::run(args, global)),
        crate::Commands::Upgrade(args) | crate::Commands::Update(args) => {
            crate::output::map_cmd_result_to_json(upgrade::run(args, global))
        }
        crate::Commands::List => {
            let err = homeboy::Error::validation_invalid_argument(
                "output_mode",
                "List command uses raw output mode",
                None,
                None,
            );
            crate::output::map_cmd_result_to_json::<serde_json::Value>(Err(err))
        }
    }
}
