use clap::{Args, Subcommand, ValueEnum};
use serde::Serialize;

use homeboy::version::{
    bump_version, bump_version_cwd, read_version, read_version_cwd, set_version, VersionTargetInfo,
};

use super::CmdResult;

#[derive(Serialize)]
#[serde(untagged)]
pub enum VersionOutput {
    Show(VersionShowOutput),
    Bump(VersionBumpOutput),
    Set(VersionSetOutput),
}

#[derive(Args)]
pub struct VersionArgs {
    #[command(subcommand)]
    command: VersionCommand,
}

#[derive(Subcommand)]
enum VersionCommand {
    /// Show current version of a component
    Show {
        /// Use current working directory (ad-hoc mode with auto-detection)
        #[arg(long)]
        cwd: bool,

        /// Component ID
        component_id: Option<String>,
    },
    /// Bump version of a component and finalize changelog
    Bump {
        /// Use current working directory (ad-hoc mode with auto-detection)
        #[arg(long)]
        cwd: bool,

        /// Component ID
        component_id: Option<String>,

        /// Version bump type
        bump_type: BumpType,
    },
    /// Set version directly (without incrementing or changelog finalization)
    #[command(visible_aliases = ["edit", "merge"])]
    Set {
        /// Component ID
        component_id: Option<String>,

        /// New version (e.g., 1.2.3)
        new_version: String,
    },
}

#[derive(Clone, ValueEnum)]
enum BumpType {
    Patch,
    Minor,
    Major,
}

impl BumpType {
    fn as_str(&self) -> &'static str {
        match self {
            BumpType::Patch => "patch",
            BumpType::Minor => "minor",
            BumpType::Major => "major",
        }
    }
}

#[derive(Serialize)]

pub struct VersionShowOutput {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    pub version: String,
    targets: Vec<VersionTargetInfo>,
}

#[derive(Serialize)]

pub struct VersionBumpOutput {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    old_version: String,
    new_version: String,
    targets: Vec<VersionTargetInfo>,
    changelog_path: String,
    changelog_finalized: bool,
    changelog_changed: bool,
}

#[derive(Serialize)]

pub struct VersionSetOutput {
    command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    component_id: Option<String>,
    old_version: String,
    new_version: String,
    targets: Vec<VersionTargetInfo>,
}

pub fn run(args: VersionArgs, _global: &crate::commands::GlobalArgs) -> CmdResult<VersionOutput> {
    match args.command {
        VersionCommand::Show { cwd, component_id } => {
            // Priority: --cwd > component_id
            let (info, resolved_id) = if cwd {
                (read_version_cwd()?, None)
            } else {
                let info = read_version(component_id.as_deref())?;
                (info, component_id)
            };

            Ok((
                VersionOutput::Show(VersionShowOutput {
                    command: "version.show".to_string(),
                    component_id: resolved_id,
                    version: info.version,
                    targets: info.targets,
                }),
                0,
            ))
        }
        VersionCommand::Bump {
            cwd,
            component_id,
            bump_type,
        } => {
            // Priority: --cwd > component_id
            let (result, resolved_id) = if cwd {
                (bump_version_cwd(bump_type.as_str())?, None)
            } else {
                let result = bump_version(component_id.as_deref(), bump_type.as_str())?;
                (result, component_id)
            };

            Ok((
                VersionOutput::Bump(VersionBumpOutput {
                    command: "version.bump".to_string(),
                    component_id: resolved_id,
                    old_version: result.old_version,
                    new_version: result.new_version,
                    targets: result.targets,
                    changelog_path: result.changelog_path,
                    changelog_finalized: result.changelog_finalized,
                    changelog_changed: result.changelog_changed,
                }),
                0,
            ))
        }
        VersionCommand::Set {
            component_id,
            new_version,
        } => {
            // Core validates componentId
            let result = set_version(component_id.as_deref(), &new_version)?;

            Ok((
                VersionOutput::Set(VersionSetOutput {
                    command: "version.set".to_string(),
                    component_id,
                    old_version: result.old_version,
                    new_version: result.new_version,
                    targets: result.targets,
                }),
                0,
            ))
        }
    }
}

pub fn show_version_output(component_id: &str) -> homeboy::Result<(VersionShowOutput, i32)> {
    let info = read_version(Some(component_id))?;

    Ok((
        VersionShowOutput {
            command: "version.show".to_string(),
            component_id: Some(component_id.to_string()),
            version: info.version,
            targets: info.targets,
        },
        0,
    ))
}
