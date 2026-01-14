use clap::{Args, Subcommand};
use serde::Serialize;

use homeboy::component::{self, Component, CreateSummary};

use super::CmdResult;

#[derive(Args)]
pub struct ComponentArgs {
    #[command(subcommand)]
    command: ComponentCommand,
}

#[derive(Subcommand)]
enum ComponentCommand {
    /// Create a new component configuration
    Create {
        /// JSON input spec for create/update (supports single or bulk)
        #[arg(long)]
        json: Option<String>,

        /// Skip items that already exist (JSON mode only)
        #[arg(long)]
        skip_existing: bool,

        /// Absolute path to local source directory (ID derived from directory name)
        #[arg(long)]
        local_path: Option<String>,
        /// Remote path relative to project basePath
        #[arg(long)]
        remote_path: Option<String>,
        /// Build artifact path relative to localPath
        #[arg(long)]
        build_artifact: Option<String>,
        /// Version targets in the form "file" or "file::pattern" (repeatable)
        #[arg(long = "version-target", value_name = "TARGET")]
        version_targets: Vec<String>,
        /// Build command to run in localPath
        #[arg(long)]
        build_command: Option<String>,
        /// Extract command to run after upload (e.g., "unzip -o {artifact} && rm {artifact}")
        #[arg(long)]
        extract_command: Option<String>,
    },
    /// Display component configuration
    Show {
        /// Component ID
        id: String,
    },
    /// Update component configuration fields
    #[command(visible_aliases = ["edit", "merge"])]
    Set {
        /// Component ID (optional if provided in JSON body)
        id: Option<String>,
        /// JSON object to merge into config (supports @file and - for stdin)
        #[arg(long, value_name = "JSON")]
        json: String,
    },
    /// Delete a component configuration
    Delete {
        /// Component ID
        id: String,
    },
    /// Rename a component (changes ID directly)
    Rename {
        /// Current component ID
        id: String,
        /// New component ID (should match repository directory name)
        new_id: String,
    },
    /// List all available components
    List,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentOutput {
    pub command: String,
    pub component_id: Option<String>,
    pub success: bool,
    pub updated_fields: Vec<String>,
    pub component: Option<Component>,
    pub components: Vec<Component>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub import: Option<CreateSummary>,
}

pub fn run(
    args: ComponentArgs,
    _global: &crate::commands::GlobalArgs,
) -> CmdResult<ComponentOutput> {
    match args.command {
        ComponentCommand::Create {
            json,
            skip_existing,
            local_path,
            remote_path,
            build_artifact,
            version_targets,
            build_command,
            extract_command,
        } => {
            if let Some(spec) = json {
                return create_json(&spec, skip_existing);
            }

            let result = component::create_from_cli(
                local_path,
                remote_path,
                build_artifact,
                version_targets,
                build_command,
                extract_command,
            )?;

            Ok((
                ComponentOutput {
                    command: "component.create".to_string(),
                    component_id: Some(result.id),
                    success: true,
                    updated_fields: vec![],
                    component: Some(result.component),
                    components: vec![],
                    import: None,
                },
                0,
            ))
        }
        ComponentCommand::Show { id } => show(&id),
        ComponentCommand::Set { id, json } => set(id.as_deref(), &json),
        ComponentCommand::Delete { id } => delete(&id),
        ComponentCommand::Rename { id, new_id } => rename(&id, &new_id),
        ComponentCommand::List => list(),
    }
}

fn create_json(spec: &str, skip_existing: bool) -> CmdResult<ComponentOutput> {
    let summary = component::create_from_json(spec, skip_existing)?;
    let exit_code = if summary.errors > 0 { 1 } else { 0 };

    Ok((
        ComponentOutput {
            command: "component.create".to_string(),
            component_id: None,
            success: summary.errors == 0,
            updated_fields: vec![],
            component: None,
            components: vec![],
            import: Some(summary),
        },
        exit_code,
    ))
}

fn show(id: &str) -> CmdResult<ComponentOutput> {
    let component = component::load(id)?;

    Ok((
        ComponentOutput {
            command: "component.show".to_string(),
            component_id: Some(id.to_string()),
            success: true,
            updated_fields: vec![],
            component: Some(component),
            components: vec![],
            import: None,
        },
        0,
    ))
}

fn set(id: Option<&str>, json: &str) -> CmdResult<ComponentOutput> {
    let result = component::merge_from_json(id, json)?;
    let component = component::load(&result.id)?;
    Ok((
        ComponentOutput {
            command: "component.set".to_string(),
            component_id: Some(result.id),
            success: true,
            updated_fields: result.updated_fields,
            component: Some(component),
            components: vec![],
            import: None,
        },
        0,
    ))
}

fn delete(id: &str) -> CmdResult<ComponentOutput> {
    component::delete_safe(id)?;

    Ok((
        ComponentOutput {
            command: "component.delete".to_string(),
            component_id: Some(id.to_string()),
            success: true,
            updated_fields: vec![],
            component: None,
            components: vec![],
            import: None,
        },
        0,
    ))
}

fn rename(id: &str, new_id: &str) -> CmdResult<ComponentOutput> {
    let result = component::rename(id, new_id)?;

    Ok((
        ComponentOutput {
            command: "component.rename".to_string(),
            component_id: Some(result.id.clone()),
            success: true,
            updated_fields: vec!["id".to_string()],
            component: Some(result.component),
            components: vec![],
            import: None,
        },
        0,
    ))
}

fn list() -> CmdResult<ComponentOutput> {
    let components = component::list()?;

    Ok((
        ComponentOutput {
            command: "component.list".to_string(),
            component_id: None,
            success: true,
            updated_fields: vec![],
            component: None,
            components,
            import: None,
        },
        0,
    ))
}
