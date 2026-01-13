use homeboy::{Error, Result};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use std::io::Read;

/// Read JSON from a spec string: "-" for stdin, "@path" for file, or inline JSON.
pub fn read_json_spec_to_string(spec: &str) -> Result<String> {
    use std::io::IsTerminal;

    if spec.trim() == "-" {
        let mut buf = String::new();
        let mut stdin = std::io::stdin();
        if stdin.is_terminal() {
            return Err(Error::validation_invalid_argument(
                "json",
                "Cannot read JSON from stdin when stdin is a TTY",
                None,
                None,
            ));
        }
        stdin
            .read_to_string(&mut buf)
            .map_err(|e| Error::internal_io(e.to_string(), Some("read stdin".to_string())))?;
        return Ok(buf);
    }

    if let Some(path) = spec.strip_prefix('@') {
        if path.trim().is_empty() {
            return Err(Error::validation_invalid_argument(
                "json",
                "Invalid JSON spec '@' (missing file path)",
                None,
                None,
            ));
        }

        return std::fs::read_to_string(path).map_err(|e| {
            Error::internal_io(
                e.to_string(),
                Some(format!("read json file spec '{}'", path)),
            )
        });
    }

    Ok(spec.to_string())
}

/// Wrapper for JSON payloads with an `op` field and `data` field.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OpPayload<T> {
    pub op: String,
    pub data: T,
}

/// Load typed data from a JSON spec with operation validation.
pub fn load_op_data<T: DeserializeOwned>(spec: &str, expected_op: &str) -> Result<T> {
    let raw = read_json_spec_to_string(spec)?;

    let payload: OpPayload<T> = serde_json::from_str(&raw)
        .map_err(|e| Error::validation_invalid_json(e, Some("parse op payload".to_string())))?;

    if payload.op != expected_op {
        return Err(Error::validation_invalid_argument(
            "op",
            format!("Unexpected op '{}'", payload.op),
            Some(expected_op.to_string()),
            Some(vec![expected_op.to_string()]),
        ));
    }

    Ok(payload.data)
}

/// Simple bulk input with just component IDs.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BulkIdsInput {
    pub component_ids: Vec<String>,
}

/// Parse JSON spec into a BulkIdsInput.
pub fn parse_bulk_ids(json_spec: &str) -> Result<BulkIdsInput> {
    let raw = read_json_spec_to_string(json_spec)?;
    serde_json::from_str(&raw)
        .map_err(|e| Error::validation_invalid_json(e, Some("parse bulk IDs input".to_string())))
}
