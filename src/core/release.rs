use serde::{Deserialize, Serialize};

use std::collections::HashMap;

use crate::component::{self, Component};
use crate::error::{Error, Result};
use crate::module::{self, ModuleManifest};
use crate::pipeline::{self, PipelineCapabilityResolver, PipelinePlanStep, PipelineStep};

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

struct ReleaseCapabilityResolver<'a> {
    module: Option<&'a ModuleManifest>,
}

impl<'a> ReleaseCapabilityResolver<'a> {
    fn new(module: Option<&'a ModuleManifest>) -> Self {
        Self { module }
    }
}

impl PipelineCapabilityResolver for ReleaseCapabilityResolver<'_> {
    fn is_supported(&self, step_type: &str) -> bool {
        is_core_step(step_type) || self.supports_module_action(step_type)
    }

    fn missing(&self, step_type: &str) -> Vec<String> {
        let action_id = format!("release.{}", step_type);
        vec![format!("Missing action '{}'", action_id)]
    }
}

impl ReleaseCapabilityResolver<'_> {
    fn supports_module_action(&self, step_type: &str) -> bool {
        let action_id = format!("release.{}", step_type);
        self.module
            .map(|module| module.actions.iter().any(|action| action.id == action_id))
            .unwrap_or(false)
    }
}

pub fn resolve_component_release(component: &Component) -> Option<ReleaseConfig> {
    component.release.clone()
}

pub fn plan(component_id: &str, module_id: Option<&str>) -> Result<ReleasePlan> {
    let component = component::load(component_id)?;
    let module = match module_id {
        Some(id) => {
            let suggestions = module::available_module_ids();
            Some(
                module::load_module(id)
                    .ok_or_else(|| Error::module_not_found(id.to_string(), suggestions))?,
            )
        }
        None => None,
    };
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
    let resolver = ReleaseCapabilityResolver::new(module.as_ref());
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
    let hints = build_plan_hints(component_id, &steps, module.as_ref());

    Ok(ReleasePlan {
        component_id: component_id.to_string(),
        enabled,
        steps,
        warnings: pipeline_plan.warnings,
        hints,
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
    module: Option<&ModuleManifest>,
) -> Vec<String> {
    let mut hints = Vec::new();
    if steps.is_empty() {
        hints.push("Release plan has no steps".to_string());
    }

    if steps
        .iter()
        .any(|step| matches!(step.status, ReleasePlanStatus::Missing))
    {
        match module {
            Some(module) => {
                hints.push(format!(
                    "Add module actions like 'release.<step_type>' in {}",
                    module.id
                ));
            }
            None => {
                hints.push("Provide --module to resolve module release actions".to_string());
            }
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
