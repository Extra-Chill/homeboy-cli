use regex::Regex;

/// Parse version from content using regex pattern.
/// Pattern must contain a capture group for the version string.
pub fn parse_version(content: &str, pattern: &str) -> Option<String> {
    let re = Regex::new(pattern).ok()?;
    re.captures(content)
        .and_then(|caps| caps.get(1))
        .map(|m| m.as_str().to_string())
}

pub fn parse_versions(content: &str, pattern: &str) -> Option<Vec<String>> {
    let re = Regex::new(pattern).ok()?;
    let mut versions = Vec::new();

    for caps in re.captures_iter(content) {
        if let Some(m) = caps.get(1) {
            versions.push(m.as_str().to_string());
        }
    }

    Some(versions)
}

pub fn replace_versions(
    content: &str,
    pattern: &str,
    new_version: &str,
) -> Option<(String, usize)> {
    let re = Regex::new(pattern).ok()?;
    let mut count = 0usize;

    let replaced = re
        .replace_all(content, |caps: &regex::Captures| {
            count += 1;
            let full_match = caps.get(0).map(|m| m.as_str()).unwrap_or("");
            let captured = caps.get(1).map(|m| m.as_str()).unwrap_or("");
            full_match.replacen(captured, new_version, 1)
        })
        .to_string();

    Some((replaced, count))
}

/// Get default version pattern based on file extension.
pub fn default_pattern_for_file(filename: &str) -> &'static str {
    if filename.ends_with(".toml") {
        r#"version\s*=\s*"(\d+\.\d+\.\d+)""#
    } else if filename.ends_with(".json") {
        r#""version"\s*:\s*"(\d+\.\d+\.\d+)""#
    } else if filename.ends_with(".php") {
        r"Version:\s*(\d+\.\d+\.\d+)"
    } else {
        r"(\d+\.\d+\.\d+)"
    }
}

/// Increment semver version.
/// bump_type: "patch", "minor", or "major"
pub fn increment_version(version: &str, bump_type: &str) -> Option<String> {
    let parts: Vec<&str> = version.split('.').collect();
    if parts.len() != 3 {
        return None;
    }

    let major: u32 = parts[0].parse().ok()?;
    let minor: u32 = parts[1].parse().ok()?;
    let patch: u32 = parts[2].parse().ok()?;

    let (new_major, new_minor, new_patch) = match bump_type {
        "patch" => (major, minor, patch + 1),
        "minor" => (major, minor + 1, 0),
        "major" => (major + 1, 0, 0),
        _ => return None,
    };

    Some(format!("{}.{}.{}", new_major, new_minor, new_patch))
}
