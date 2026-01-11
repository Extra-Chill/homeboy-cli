use super::ConfigManager;
use crate::Result;

const DEFAULT_CLI_PATH: &str = "wp";
const DEFAULT_DATABASE_HOST: &str = "127.0.0.1";
const DEFAULT_LOCAL_DB_PORT: u16 = 33306;

#[derive(Debug, Clone)]
pub struct EffectiveDbSettings {
    pub cli_path: String,
    pub host: String,
    pub local_port: u16,
}

pub fn resolve_db_settings() -> Result<EffectiveDbSettings> {
    let app = ConfigManager::load_app_config()?;

    let cli_path = app
        .default_cli_path
        .unwrap_or_else(|| DEFAULT_CLI_PATH.to_string());

    let host = app
        .default_database_host
        .unwrap_or_else(|| DEFAULT_DATABASE_HOST.to_string());

    let local_port = app.default_local_db_port.unwrap_or(DEFAULT_LOCAL_DB_PORT);

    Ok(EffectiveDbSettings {
        cli_path,
        host,
        local_port,
    })
}

pub fn resolve_cli_path() -> Result<String> {
    let app = ConfigManager::load_app_config()?;
    Ok(app
        .default_cli_path
        .unwrap_or_else(|| DEFAULT_CLI_PATH.to_string()))
}

pub fn resolve_database_host() -> Result<String> {
    let app = ConfigManager::load_app_config()?;
    Ok(app
        .default_database_host
        .unwrap_or_else(|| DEFAULT_DATABASE_HOST.to_string()))
}

pub fn resolve_local_db_port() -> Result<u16> {
    let app = ConfigManager::load_app_config()?;
    Ok(app.default_local_db_port.unwrap_or(DEFAULT_LOCAL_DB_PORT))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolve_db_settings_returns_defaults_when_no_config() {
        let settings = resolve_db_settings().unwrap();
        assert_eq!(settings.cli_path, "wp");
        assert_eq!(settings.host, "127.0.0.1");
        assert_eq!(settings.local_port, 33306);
    }
}
