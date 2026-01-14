use crate::error::Error;
use crate::json;
use crate::local_files::{self, FileSystem};
use crate::slugify;
use crate::Result;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::path::PathBuf;

pub(crate) trait ConfigEntity: Serialize + DeserializeOwned {
    fn id(&self) -> &str;
    fn set_id(&mut self, id: String);
    fn config_path(id: &str) -> Result<PathBuf>;
    fn config_dir() -> Result<PathBuf>;
    fn not_found_error(id: String) -> Error;
    fn entity_type() -> &'static str;
}

pub(crate) fn load<T: ConfigEntity>(id: &str) -> Result<T> {
    let path = T::config_path(id)?;
    if !path.exists() {
        return Err(T::not_found_error(id.to_string()));
    }
    let content = local_files::local().read(&path)?;
    let mut entity: T = json::from_str(&content)?;
    entity.set_id(id.to_string());
    Ok(entity)
}

pub(crate) fn list<T: ConfigEntity>() -> Result<Vec<T>> {
    let dir = T::config_dir()?;
    let entries = local_files::local().list(&dir)?;

    let mut items: Vec<T> = entries
        .into_iter()
        .filter(|e| e.is_json() && !e.is_dir)
        .filter_map(|e| {
            let id = e.path.file_stem()?.to_string_lossy().to_string();
            let content = local_files::local().read(&e.path).ok()?;
            let mut entity: T = json::from_str(&content).ok()?;
            entity.set_id(id);
            Some(entity)
        })
        .collect();
    items.sort_by(|a, b| a.id().cmp(b.id()));
    Ok(items)
}

pub(crate) fn save<T: ConfigEntity>(entity: &T) -> Result<()> {
    slugify::validate_component_id(entity.id())?;

    let path = T::config_path(entity.id())?;
    local_files::ensure_app_dirs()?;
    let content = json::to_string_pretty(entity)?;
    local_files::local().write(&path, &content)?;
    Ok(())
}

pub(crate) fn delete<T: ConfigEntity>(id: &str) -> Result<()> {
    let path = T::config_path(id)?;
    if !path.exists() {
        return Err(T::not_found_error(id.to_string()));
    }
    local_files::local().delete(&path)?;
    Ok(())
}

pub(crate) fn exists<T: ConfigEntity>(id: &str) -> bool {
    T::config_path(id).map(|p| p.exists()).unwrap_or(false)
}

pub(crate) fn list_ids<T: ConfigEntity>() -> Result<Vec<String>> {
    let dir = T::config_dir()?;
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let entries = local_files::local().list(&dir)?;
    let mut ids: Vec<String> = entries
        .into_iter()
        .filter(|e| e.is_json() && !e.is_dir)
        .filter_map(|e| e.path.file_stem().map(|s| s.to_string_lossy().to_string()))
        .collect();
    ids.sort();
    Ok(ids)
}

// ============================================================================
// Batch Operations
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResult {
    pub created: u32,
    pub skipped: u32,
    pub errors: u32,
    pub items: Vec<BatchResultItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchResultItem {
    pub id: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl BatchResult {
    pub fn new() -> Self {
        Self {
            created: 0,
            skipped: 0,
            errors: 0,
            items: Vec::new(),
        }
    }

    pub fn record_created(&mut self, id: String) {
        self.created += 1;
        self.items.push(BatchResultItem {
            id,
            status: "created".to_string(),
            error: None,
        });
    }

    pub fn record_skipped(&mut self, id: String) {
        self.skipped += 1;
        self.items.push(BatchResultItem {
            id,
            status: "skipped".to_string(),
            error: None,
        });
    }

    pub fn record_error(&mut self, id: String, error: String) {
        self.errors += 1;
        self.items.push(BatchResultItem {
            id,
            status: "error".to_string(),
            error: Some(error),
        });
    }
}

impl Default for BatchResult {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Generic JSON Merge
// ============================================================================

pub(crate) fn merge_from_json<T: ConfigEntity>(
    id: Option<&str>,
    json_spec: &str,
) -> Result<json::MergeResult> {
    let raw = json::read_json_spec_to_string(json_spec)?;
    let mut parsed: serde_json::Value = json::from_str(&raw)?;

    let effective_id = id
        .map(String::from)
        .or_else(|| parsed.get("id").and_then(|v| v.as_str()).map(String::from))
        .ok_or_else(|| {
            Error::validation_invalid_argument(
                "id",
                &format!("Provide {} ID as argument or in JSON body", T::entity_type()),
                None,
                None,
            )
        })?;

    if let Some(obj) = parsed.as_object_mut() {
        obj.remove("id");
    }

    let mut entity = load::<T>(&effective_id)?;
    let result = json::merge_config(&mut entity, parsed)?;
    save(&entity)?;

    Ok(json::MergeResult {
        id: effective_id,
        updated_fields: result.updated_fields,
    })
}
