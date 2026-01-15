use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::component::{self, Component};
use crate::error::{Error, Result};
use crate::module::{self, ModuleManifest};
use crate::pipeline::{
    self, PipelineCapabilityResolver, PipelinePlanStep, PipelineRunResult, PipelineRunStatus,
    PipelineStep, PipelineStepExecutor, PipelineStepResult,
};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]

pub struct ReleaseConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<ReleaseStep>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub settings: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ReleaseStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, serde_json::Value>,
}

impl From<ReleaseStep> for PipelineStep {
    fn from(step: ReleaseStep) -> Self {
        PipelineStep {
            id: step.id,
            step_type: step.step_type,
            label: step.label,
            needs: step.needs,
            config: step.config,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ReleasePlan {
    pub component_id: String,
    pub enabled: bool,
    pub steps: Vec<ReleasePlanStep>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ReleaseRun {
    pub component_id: String,
    pub enabled: bool,
    pub result: PipelineRunResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]

pub struct ReleasePlanStep {
    pub id: String,
    #[serde(rename = "type")]
    pub step_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub needs: Vec<String>,
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub config: HashMap<String, serde_json::Value>,
    pub status: ReleasePlanStatus,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub missing: Vec<String>,
}

impl From<PipelinePlanStep> for ReleasePlanStep {
    fn from(step: PipelinePlanStep) -> Self {
        let status = match step.status {
            pipeline::PipelineStepStatus::Ready => ReleasePlanStatus::Ready,
            pipeline::PipelineStepStatus::Missing => ReleasePlanStatus::Missing,
            pipeline::PipelineStepStatus::Disabled => ReleasePlanStatus::Disabled,
        };

        Self {
            id: step.id,
            step_type: step.step_type,
            label: step.label,
            needs: step.needs,
            config: step.config,
            status,
            missing: step.missing,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]

pub enum ReleasePlanStatus {
    Ready,
    Missing,
    Disabled,
}

struct ReleaseCapabilityResolver {
    modules: Vec<ModuleManifest>,
}

impl ReleaseCapabilityResolver {
    fn new(modules: Vec<ModuleManifest>) -> Self {
        Self { modules }
    }
}

impl PipelineCapabilityResolver for ReleaseCapabilityResolver {
    fn is_supported(&self, step_type: &str) -> bool {
        is_core_step(step_type) || self.supports_module_action(step_type)
    }

    fn missing(&self, step_type: &str) -> Vec<String> {
        let action_id = format!("release.{}", step_type);
        vec![format!("Missing action '{}'", action_id)]
    }
}

impl ReleaseCapabilityResolver {
    fn supports_module_action(&self, step_type: &str) -> bool {
        let action_id = format!("release.{}", step_type);
        self.modules
            .iter()
            .any(|module| module.actions.iter().any(|action| action.id == action_id))
    }
}

struct ReleaseStepExecutor {
    component_id: String,
    modules: Vec<ModuleManifest>,
}

impl ReleaseStepExecutor {
    fn new(component_id: String, modules: Vec<ModuleManifest>) -> Self {
        Self {
            component_id,
            modules,
        }
    }

    fn step_result(
        &self,
        step: &PipelineStep,
        status: PipelineRunStatus,
        data: Option<serde_json::Value>,
        error: Option<String>,
        hints: Vec<crate::error::Hint>,
    ) -> PipelineStepResult {
        PipelineStepResult {
            id: step.id.clone(),
            step_type: step.step_type.clone(),
            status,
            missing: Vec::new(),
            warnings: Vec::new(),
            hints,
            data,
            error,
        }
    }

    fn execute_core_step(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        match step.step_type.as_str() {
            "build" => self.run_build(step),
            "changes" => self.run_changes(step),
            "version" => self.run_version(step),
            "git.tag" => self.run_git_tag(step),
            "git.push" => self.run_git_push(step),
            _ => Err(Error::validation_invalid_argument(
                "release.steps",
                format!("Unsupported core step '{}'", step.step_type),
                None,
                None,
            )),
        }
    }

    fn run_build(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let (output, exit_code) = crate::build::run(&self.component_id)?;
        let data = serde_json::to_value(output)
            .map_err(|e| Error::internal_json(e.to_string(), Some("build output".to_string())))?;
        let status = if exit_code == 0 {
            PipelineRunStatus::Success
        } else {
            PipelineRunStatus::Failed
        };
        Ok(self.step_result(step, status, Some(data), None, Vec::new()))
    }

    fn run_changes(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let include_diff = step
            .config
            .get("includeDiff")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let output = crate::git::changes(Some(&self.component_id), None, include_diff)?;
        let data = serde_json::to_value(output)
            .map_err(|e| Error::internal_json(e.to_string(), Some("changes output".to_string())))?;
        Ok(self.step_result(
            step,
            PipelineRunStatus::Success,
            Some(data),
            None,
            Vec::new(),
        ))
    }

    fn run_version(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let bump_type = step
            .config
            .get("bump")
            .and_then(|v| v.as_str())
            .unwrap_or("patch");
        let result = crate::version::bump_version(Some(&self.component_id), bump_type)?;
        let data = serde_json::to_value(result)
            .map_err(|e| Error::internal_json(e.to_string(), Some("version output".to_string())))?;
        Ok(self.step_result(
            step,
            PipelineRunStatus::Success,
            Some(data),
            None,
            Vec::new(),
        ))
    }

    fn run_git_tag(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let tag = step
            .config
            .get("name")
            .and_then(|v| v.as_str())
            .map(|v| v.to_string())
            .or_else(|| {
                step.config
                    .get("versionTag")
                    .and_then(|v| v.as_str())
                    .map(|v| v.to_string())
            });
        let message = step.config.get("message").and_then(|v| v.as_str());

        let tag_name = match tag {
            Some(name) => name,
            None => {
                let info = crate::version::read_version(Some(&self.component_id))?;
                info.version
            }
        };

        let output = crate::git::tag(Some(&self.component_id), Some(&tag_name), message)?;
        let data = serde_json::to_value(output)
            .map_err(|e| Error::internal_json(e.to_string(), Some("git tag output".to_string())))?;
        Ok(self.step_result(
            step,
            PipelineRunStatus::Success,
            Some(data),
            None,
            Vec::new(),
        ))
    }

    fn run_git_push(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let tags = step
            .config
            .get("tags")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        let output = crate::git::push(Some(&self.component_id), tags)?;
        let data = serde_json::to_value(output).map_err(|e| {
            Error::internal_json(e.to_string(), Some("git push output".to_string()))
        })?;
        Ok(self.step_result(
            step,
            PipelineRunStatus::Success,
            Some(data),
            None,
            Vec::new(),
        ))
    }

    fn run_module_action(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        let action_id = format!("release.{}", step.step_type);
        let module = resolve_module_action(&self.modules, &action_id)?;
        let payload = if step.config.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&step.config).map_err(|e| {
                Error::internal_json(e.to_string(), Some("release step config".to_string()))
            })?)
        };

        let response = module::run_action(&module.id, &action_id, None, payload.as_deref())?;
        let data = serde_json::to_value(response).map_err(|e| {
            Error::internal_json(e.to_string(), Some("module action output".to_string()))
        })?;
        Ok(self.step_result(
            step,
            PipelineRunStatus::Success,
            Some(data),
            None,
            Vec::new(),
        ))
    }
}

impl PipelineStepExecutor for ReleaseStepExecutor {
    fn execute_step(&self, step: &PipelineStep) -> Result<PipelineStepResult> {
        if is_core_step(&step.step_type) {
            return self.execute_core_step(step);
        }

        self.run_module_action(step)
    }
}

fn resolve_modules(component: &Component, module_id: Option<&str>) -> Result<Vec<ModuleManifest>> {
    if module_id.is_some() {
        return Err(Error::validation_invalid_argument(
            "module",
            "Module selection is configured via component.modules; --module is not supported",
            None,
            None,
        ));
    }

    let mut modules = Vec::new();
    if let Some(configured) = component.modules.as_ref() {
        let mut module_ids: Vec<String> = configured.keys().cloned().collect();
        module_ids.sort();
        let suggestions = module::available_module_ids();
        for module_id in module_ids {
            let manifest = module::load_module(&module_id).ok_or_else(|| {
                Error::module_not_found(module_id.to_string(), suggestions.clone())
            })?;
            modules.push(manifest);
        }
    }

    Ok(modules)
}

fn resolve_module_action(modules: &[ModuleManifest], action_id: &str) -> Result<ModuleManifest> {
    let matches: Vec<ModuleManifest> = modules
        .iter()
        .filter(|module| module.actions.iter().any(|action| action.id == action_id))
        .cloned()
        .collect();

    if matches.is_empty() {
        return Err(Error::validation_invalid_argument(
            "release.steps",
            format!("Missing module action '{}'", action_id),
            None,
            None,
        ));
    }

    if matches.len() > 1 {
        let modules: Vec<String> = matches.iter().map(|module| module.id.clone()).collect();
        return Err(Error::validation_invalid_argument(
            "release.steps",
            format!(
                "Multiple modules provide action '{}' ({})",
                action_id,
                modules.join(", ")
            ),
            None,
            None,
        ));
    }

    Ok(matches[0].clone())
}

pub fn resolve_component_release(component: &Component) -> Option<ReleaseConfig> {
    component.release.clone()
}

pub fn plan(component_id: &str, module_id: Option<&str>) -> Result<ReleasePlan> {
    let component = component::load(component_id)?;
    let modules = resolve_modules(&component, module_id)?;
    let resolver = ReleaseCapabilityResolver::new(modules.clone());
    let release = resolve_component_release(&component).ok_or_else(|| {
        Error::validation_invalid_argument(
            "release",
            "Release configuration is missing",
            Some(component_id.to_string()),
            None,
        )
        .with_hint(format!(
            "Use 'homeboy component set {} --json' to add a release block",
            component_id
        ))
        .with_hint("See 'homeboy docs commands/release' for examples")
    })?;

    let enabled = release.enabled.unwrap_or(true);
    let pipeline_steps: Vec<PipelineStep> = release
        .steps
        .iter()
        .cloned()
        .map(PipelineStep::from)
        .collect();
    let pipeline_plan = pipeline::plan(&pipeline_steps, &resolver, enabled, "release.steps")?;
    let steps: Vec<ReleasePlanStep> = pipeline_plan
        .steps
        .into_iter()
        .map(ReleasePlanStep::from)
        .collect();
    let hints = build_plan_hints(component_id, &steps, &modules);

    Ok(ReleasePlan {
        component_id: component_id.to_string(),
        enabled,
        steps,
        warnings: pipeline_plan.warnings,
        hints,
    })
}

pub fn run(component_id: &str, module_id: Option<&str>) -> Result<ReleaseRun> {
    let component = component::load(component_id)?;
    let modules = resolve_modules(&component, module_id)?;
    let resolver = ReleaseCapabilityResolver::new(modules.clone());
    let release = resolve_component_release(&component).ok_or_else(|| {
        Error::validation_invalid_argument(
            "release",
            "Release configuration is missing",
            Some(component_id.to_string()),
            None,
        )
        .with_hint(format!(
            "Use 'homeboy component set {} --json' to add a release block",
            component_id
        ))
        .with_hint("See 'homeboy docs commands/release' for examples")
    })?;

    let enabled = release.enabled.unwrap_or(true);
    let executor = ReleaseStepExecutor::new(component_id.to_string(), modules.clone());

    let pipeline_steps: Vec<PipelineStep> =
        release.steps.into_iter().map(PipelineStep::from).collect();

    let run_result = pipeline::run(
        &pipeline_steps,
        std::sync::Arc::new(executor),
        std::sync::Arc::new(resolver),
        enabled,
        "release.steps",
    )?;

    Ok(ReleaseRun {
        component_id: component_id.to_string(),
        enabled,
        result: run_result,
    })
}

fn is_core_step(step_type: &str) -> bool {
    matches!(
        step_type,
        "build" | "changelog" | "version" | "git.tag" | "git.push" | "changes"
    )
}

fn build_plan_hints(
    component_id: &str,
    steps: &[ReleasePlanStep],
    modules: &[ModuleManifest],
) -> Vec<String> {
    let mut hints = Vec::new();
    if steps.is_empty() {
        hints.push("Release plan has no steps".to_string());
    }

    if steps
        .iter()
        .any(|step| matches!(step.status, ReleasePlanStatus::Missing))
    {
        if modules.is_empty() {
            hints.push("Configure component modules to resolve release actions".to_string());
        } else {
            let module_names: Vec<String> =
                modules.iter().map(|module| module.id.clone()).collect();
            hints.push(format!(
                "Release actions are resolved from modules: {}",
                module_names.join(", ")
            ));
        }
    }

    if !hints.is_empty() {
        hints.push(format!(
            "Update release config with: homeboy component set {} --json",
            component_id
        ));
    }

    hints
}
