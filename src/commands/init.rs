use std::collections::HashSet;

use clap::Args;
use serde::Serialize;

use homeboy::component::{self, Component};
use homeboy::context::{self, ContextOutput};
use homeboy::module::{is_module_compatible, is_module_linked, is_module_ready, load_all_modules};
use homeboy::project::{self, Project};
use homeboy::server::{self, Server};

use super::CmdResult;

#[derive(Args)]
pub struct InitArgs {
    /// Show all components, modules, projects, and servers
    #[arg(long, short = 'a')]
    pub all: bool,
}

#[derive(Debug, Serialize)]
pub struct InitOutput {
    pub command: &'static str,
    pub context: ContextOutput,
    pub next_steps: Vec<String>,
    pub servers: Vec<Server>,
    pub projects: Vec<ProjectListItem>,
    pub components: Vec<Component>,
    pub modules: Vec<ModuleEntry>,
}

#[derive(Debug, Serialize)]
pub struct ProjectListItem {
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<String>,
}

impl From<Project> for ProjectListItem {
    fn from(p: Project) -> Self {
        Self {
            id: p.id,
            domain: p.domain,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ModuleEntry {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub runtime: String,
    pub compatible: bool,
    pub ready: bool,
    pub linked: bool,
}

pub fn run_json(args: InitArgs) -> CmdResult<InitOutput> {
    // Get context for current directory
    let (context_output, _) = context::run(None)?;

    // Collect relevant component IDs from context
    let relevant_ids: HashSet<String> = context_output
        .matched_components
        .iter()
        .chain(context_output.contained_components.iter())
        .cloned()
        .collect();

    // Load all data sources
    let all_components = component::list().unwrap_or_default();
    let all_projects = project::list().unwrap_or_default();
    let all_servers = server::list().unwrap_or_default();
    let all_modules = load_all_modules();

    // Determine if we should show focused output
    let show_all = args.all || relevant_ids.is_empty();

    // Filter components
    let components: Vec<Component> = if show_all {
        all_components
    } else {
        all_components
            .into_iter()
            .filter(|c| relevant_ids.contains(&c.id))
            .collect()
    };

    // Get module IDs linked to matched components
    let linked_module_ids: HashSet<String> = components
        .iter()
        .filter_map(|c| c.modules.as_ref())
        .flat_map(|m| m.keys().cloned())
        .collect();

    // Filter modules: linked modules + platform modules (runtime.is_none())
    let modules: Vec<ModuleEntry> = all_modules
        .iter()
        .filter(|m| show_all || linked_module_ids.contains(&m.id) || m.runtime.is_none())
        .map(|m| ModuleEntry {
            id: m.id.clone(),
            name: m.name.clone(),
            version: m.version.clone(),
            description: m
                .description
                .as_ref()
                .and_then(|d| d.lines().next())
                .unwrap_or("")
                .to_string(),
            runtime: if m.runtime.is_some() {
                "executable"
            } else {
                "platform"
            }
            .to_string(),
            compatible: is_module_compatible(m, None),
            ready: is_module_ready(m),
            linked: is_module_linked(&m.id),
        })
        .collect();

    // Filter projects: those containing relevant components
    let filtered_projects: Vec<Project> = if show_all {
        all_projects
    } else {
        all_projects
            .into_iter()
            .filter(|p| p.component_ids.iter().any(|id| relevant_ids.contains(id)))
            .collect()
    };

    // Get server IDs from filtered projects
    let relevant_server_ids: HashSet<String> = filtered_projects
        .iter()
        .filter_map(|p| p.server_id.clone())
        .collect();

    // Convert projects to list items
    let projects: Vec<ProjectListItem> = filtered_projects
        .into_iter()
        .map(ProjectListItem::from)
        .collect();

    // Filter servers
    let servers: Vec<Server> = if show_all {
        all_servers
    } else {
        all_servers
            .into_iter()
            .filter(|s| relevant_server_ids.contains(&s.id))
            .collect()
    };

    let mut next_steps = vec![
        "Read CLAUDE.md and README.md for repo-specific guidance.".to_string(),
        "Run `homeboy docs documentation/index` for Homeboy documentation.".to_string(),
        "Run `homeboy docs commands/commands-index` to browse available commands.".to_string(),
    ];

    if context_output.managed {
        next_steps.push("Run `homeboy context` to inspect local config state.".to_string());
        if !components.is_empty() {
            next_steps
                .push("Run `homeboy component show <id>` to inspect a component.".to_string());
        }
    } else if !context_output.contained_components.is_empty() {
        next_steps.push("Run `homeboy component show <id>` for a contained component.".to_string());
    } else {
        next_steps.push(
            "Create a project with `homeboy project create <name> <domain> --server <server_id> --module <module_id>`.".to_string(),
        );
        next_steps.push(
            "Create a component with `homeboy component create <name> --local-path . --remote-path <path> --project <project_id>`.".to_string(),
        );
    }

    if let Some(suggestion) = context_output.suggestion.as_ref() {
        next_steps.push(format!("Suggestion: {}", suggestion));
    }

    Ok((
        InitOutput {
            command: "init",
            context: context_output,
            next_steps,
            servers,
            projects,
            components,
            modules,
        },
        0,
    ))
}
