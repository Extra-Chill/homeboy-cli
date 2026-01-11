use serde::{Deserialize, Serialize};

use super::{Record, SetName, SlugIdentifiable};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VersionTarget {
    pub file: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChangelogTarget {
    pub file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentConfiguration {
    pub name: String,
    pub local_path: String,
    pub remote_path: String,
    pub build_artifact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version_targets: Option<Vec<VersionTarget>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_targets: Option<Vec<ChangelogTarget>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_next_section_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub changelog_next_section_aliases: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub build_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_network: Option<bool>,
}

impl SlugIdentifiable for ComponentConfiguration {
    fn name(&self) -> &str {
        &self.name
    }
}

impl SetName for ComponentConfiguration {
    fn set_name(&mut self, name: String) {
        self.name = name;
    }
}

impl ComponentConfiguration {
    pub fn new(
        name: String,
        local_path: String,
        remote_path: String,
        build_artifact: String,
    ) -> Self {
        Self {
            name,
            local_path,
            remote_path,
            build_artifact,
            version_targets: None,
            changelog_targets: None,
            changelog_next_section_label: None,
            changelog_next_section_aliases: None,
            build_command: None,
            is_network: None,
        }
    }
}

pub type ComponentRecord = Record<ComponentConfiguration>;
