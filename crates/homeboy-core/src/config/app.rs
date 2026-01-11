use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_project_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_changelog_next_section_label: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_changelog_next_section_aliases: Option<Vec<String>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_modules: Option<HashMap<String, InstalledModuleConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct InstalledModuleConfig {
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub settings: HashMap<String, serde_json::Value>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
}
