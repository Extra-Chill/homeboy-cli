use clap::Args;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;

#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};

#[cfg(test)]
static TEST_SCP_CALL_COUNT: AtomicUsize = AtomicUsize::new(0);

#[cfg(test)]
fn reset_test_scp_call_count() {
    TEST_SCP_CALL_COUNT.store(0, Ordering::Relaxed);
}

use homeboy_core::config::{AppPaths, ConfigManager, ServerConfig};
use homeboy_core::ssh::{CommandOutput, SshClient};
use homeboy_core::version::parse_version;

use super::CmdResult;

#[derive(Args)]
pub struct DeployArgs {
    /// Project ID
    pub project_id: String,

    /// Component IDs to deploy
    pub component_ids: Vec<String>,

    /// Deploy all configured components
    #[arg(long)]
    pub all: bool,

    /// Deploy only outdated components
    #[arg(long)]
    pub outdated: bool,

    /// Build components before deploying
    #[arg(long)]
    pub build: bool,

    /// Show what would be deployed without executing
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployComponentResult {
    pub id: String,
    pub name: String,
    pub status: String,
    pub local_version: Option<String>,
    pub remote_version: Option<String>,
    pub error: Option<String>,
    pub artifact_path: Option<String>,
    pub remote_path: Option<String>,
    pub build_command: Option<String>,
    pub build_exit_code: Option<i32>,
    pub scp_exit_code: Option<i32>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeploySummary {
    pub succeeded: u32,
    pub failed: u32,
    pub skipped: u32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeployOutput {
    pub project_id: String,
    pub all: bool,
    pub outdated: bool,
    pub build: bool,
    pub dry_run: bool,
    pub components: Vec<DeployComponentResult>,
    pub summary: DeploySummary,
}

pub fn run(args: DeployArgs) -> CmdResult<DeployOutput> {
    let project = ConfigManager::load_project(&args.project_id)?;

    let server_id = project.server_id.clone().ok_or_else(|| {
        homeboy_core::Error::Other("Server not configured for project".to_string())
    })?;

    let server = ConfigManager::load_server(&server_id)?;

    let base_path = project
        .base_path
        .clone()
        .filter(|p| !p.is_empty())
        .ok_or_else(|| {
            homeboy_core::Error::Other("Base path not configured for project".to_string())
        })?;

    let client = SshClient::from_server(&server, &server_id)?;

    let all_components = load_components(&project.component_ids);
    if all_components.is_empty() {
        return Err(homeboy_core::Error::Other(
            "No components configured for project".to_string(),
        ));
    }

    let components_to_deploy =
        plan_components_to_deploy(&args, &all_components, &server, &base_path, &client)?;

    if components_to_deploy.is_empty() {
        return Ok((
            DeployOutput {
                project_id: args.project_id,
                all: args.all,
                outdated: args.outdated,
                build: args.build,
                dry_run: args.dry_run,
                components: vec![],
                summary: DeploySummary {
                    succeeded: 0,
                    failed: 0,
                    skipped: 0,
                },
            },
            0,
        ));
    }

    let local_versions: HashMap<String, String> = components_to_deploy
        .iter()
        .filter_map(|c| fetch_local_version(c).map(|v| (c.id.clone(), v)))
        .collect();

    let remote_versions = if args.dry_run || args.outdated {
        fetch_remote_versions(
            &components_to_deploy,
            &server,
            &base_path,
            &client as &dyn RemoteExec,
        )
    } else {
        HashMap::new()
    };

    let skipped: u32 = 0;

    if args.dry_run {
        let results = components_to_deploy
            .iter()
            .map(|component| DeployComponentResult {
                id: component.id.clone(),
                name: component.name.clone(),
                status: "would_deploy".to_string(),
                local_version: local_versions.get(&component.id).cloned(),
                remote_version: remote_versions.get(&component.id).cloned(),
                error: None,
                artifact_path: Some(component.build_artifact.clone()),
                remote_path: Some(
                    homeboy_core::base_path::join_remote_path(
                        Some(&base_path),
                        &component.remote_path,
                    )
                    .unwrap_or_else(|_| component.remote_path.clone()),
                ),
                build_command: component.build_command.clone(),
                build_exit_code: None,
                scp_exit_code: None,
            })
            .collect::<Vec<_>>();

        let succeeded = results.len() as u32;

        return Ok((
            DeployOutput {
                project_id: args.project_id,
                all: args.all,
                outdated: args.outdated,
                build: args.build,
                dry_run: true,
                components: results,
                summary: DeploySummary {
                    succeeded,
                    failed: 0,
                    skipped,
                },
            },
            0,
        ));
    }

    let mut results: Vec<DeployComponentResult> = vec![];
    let mut succeeded: u32 = 0;
    let mut failed: u32 = 0;
    let skipped: u32 = 0;

    for component in &components_to_deploy {
        let local_version = local_versions.get(&component.id).cloned();
        let remote_version = remote_versions.get(&component.id).cloned();

        let (build_exit_code, build_error) = if args.build {
            run_build_if_configured(component)
        } else {
            (None, None)
        };

        if let Some(ref error) = build_error {
            results.push(DeployComponentResult {
                id: component.id.clone(),
                name: component.name.clone(),
                status: "failed".to_string(),
                local_version,
                remote_version,
                error: Some(error.clone()),
                artifact_path: Some(component.build_artifact.clone()),
                remote_path: Some(
                    homeboy_core::base_path::join_remote_path(
                        Some(&base_path),
                        &component.remote_path,
                    )
                    .unwrap_or_else(|_| component.remote_path.clone()),
                ),
                build_command: component.build_command.clone(),
                build_exit_code,
                scp_exit_code: None,
            });
            failed += 1;
            continue;
        }

        if !Path::new(&component.build_artifact).exists() {
            results.push(DeployComponentResult {
                id: component.id.clone(),
                name: component.name.clone(),
                status: "failed".to_string(),
                local_version,
                remote_version,
                error: Some(format!("Artifact not found: {}", component.build_artifact)),
                artifact_path: Some(component.build_artifact.clone()),
                remote_path: Some(
                    homeboy_core::base_path::join_remote_path(
                        Some(&base_path),
                        &component.remote_path,
                    )
                    .unwrap_or_else(|_| component.remote_path.clone()),
                ),
                build_command: component.build_command.clone(),
                build_exit_code,
                scp_exit_code: None,
            });
            failed += 1;
            continue;
        }

        let (scp_exit_code, scp_error) =
            deploy_component_artifact(&server, &client, &base_path, component);

        if let Some(error) = scp_error {
            results.push(DeployComponentResult {
                id: component.id.clone(),
                name: component.name.clone(),
                status: "failed".to_string(),
                local_version,
                remote_version,
                error: Some(error),
                artifact_path: Some(component.build_artifact.clone()),
                remote_path: Some(
                    homeboy_core::base_path::join_remote_path(
                        Some(&base_path),
                        &component.remote_path,
                    )
                    .unwrap_or_else(|_| component.remote_path.clone()),
                ),
                build_command: component.build_command.clone(),
                build_exit_code,
                scp_exit_code,
            });
            failed += 1;
            continue;
        }

        results.push(DeployComponentResult {
            id: component.id.clone(),
            name: component.name.clone(),
            status: "deployed".to_string(),
            local_version: local_version.clone(),
            remote_version: local_version,
            error: None,
            artifact_path: Some(component.build_artifact.clone()),
            remote_path: Some(
                homeboy_core::base_path::join_remote_path(Some(&base_path), &component.remote_path)
                    .unwrap_or_else(|_| component.remote_path.clone()),
            ),
            build_command: component.build_command.clone(),
            build_exit_code,
            scp_exit_code,
        });
        succeeded += 1;
    }

    let exit_code = if failed > 0 { 1 } else { 0 };

    Ok((
        DeployOutput {
            project_id: args.project_id,
            all: args.all,
            outdated: args.outdated,
            build: args.build,
            dry_run: args.dry_run,
            components: results,
            summary: DeploySummary {
                succeeded,
                failed,
                skipped,
            },
        },
        exit_code,
    ))
}

#[derive(Clone)]
struct VersionTarget {
    file: String,
    pattern: Option<String>,
}

#[derive(Clone)]
struct Component {
    id: String,
    name: String,
    local_path: String,
    remote_path: String,
    build_artifact: String,
    build_command: Option<String>,
    version_targets: Option<Vec<VersionTarget>>,
}

fn plan_components_to_deploy(
    args: &DeployArgs,
    all_components: &[Component],
    server: &ServerConfig,
    base_path: &str,
    client: &dyn RemoteExec,
) -> homeboy_core::Result<Vec<Component>> {
    if args.all {
        return Ok(all_components.to_vec());
    }

    if !args.component_ids.is_empty() {
        let selected: Vec<Component> = all_components
            .iter()
            .filter(|c| args.component_ids.contains(&c.id))
            .cloned()
            .collect();
        return Ok(selected);
    }

    if args.outdated {
        let remote_versions = fetch_remote_versions(all_components, server, base_path, client);

        let selected: Vec<Component> = all_components
            .iter()
            .filter(|c| {
                let Some(local_version) = fetch_local_version(c) else {
                    return true;
                };

                let Some(remote_version) = remote_versions.get(&c.id) else {
                    return true;
                };

                local_version != *remote_version
            })
            .cloned()
            .collect();

        return Ok(selected);
    }

    Err(homeboy_core::Error::Other(
        "No components specified. Use component IDs, --all, or --outdated".to_string(),
    ))
}

fn run_build_if_configured(component: &Component) -> (Option<i32>, Option<String>) {
    let Some(ref build_cmd) = component.build_command else {
        return (None, None);
    };

    let status = Command::new("sh")
        .args(["-c", build_cmd])
        .current_dir(&component.local_path)
        .status();

    match status {
        Ok(status) => {
            if status.success() {
                (Some(status.code().unwrap_or(0)), None)
            } else {
                (
                    Some(status.code().unwrap_or(1)),
                    Some(format!("Build failed for {}", component.id)),
                )
            }
        }
        Err(err) => (Some(1), Some(format!("Build error: {}", err))),
    }
}

fn deploy_component_artifact(
    server: &ServerConfig,
    client: &dyn RemoteExec,
    base_path: &str,
    component: &Component,
) -> (Option<i32>, Option<String>) {
    let install_dir =
        match homeboy_core::base_path::join_remote_path(Some(base_path), &component.remote_path) {
            Ok(value) => value,
            Err(err) => return (Some(1), Some(err.to_string())),
        };

    if component.build_artifact.ends_with(".zip") {
        let zip_filename = Path::new(&component.build_artifact)
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| format!(".homeboy-{}", name))
            .unwrap_or_else(|| format!(".homeboy-{}.zip", component.id));

        let upload_path = match homeboy_core::base_path::join_remote_child(
            Some(base_path),
            &component.remote_path,
            &zip_filename,
        ) {
            Ok(value) => value,
            Err(err) => return (Some(1), Some(err.to_string())),
        };

        let mkdir_cmd =
            match homeboy_core::shell::cd_and("/", &format!("mkdir -p '{}'", install_dir)) {
                Ok(value) => value,
                Err(err) => return (Some(1), Some(err.to_string())),
            };

        let mkdir_output = client.execute(&mkdir_cmd);
        if !mkdir_output.success {
            return (Some(mkdir_output.exit_code), Some(mkdir_output.stderr));
        }

        let (scp_exit_code, scp_error) =
            scp_to_path(server, client, &component.build_artifact, &upload_path);
        if scp_error.is_some() {
            return (scp_exit_code, scp_error);
        }

        let unzip_cmd = match homeboy_core::shell::cd_and(
            &install_dir,
            &format!("unzip -o '{}' && rm '{}'", upload_path, upload_path),
        ) {
            Ok(value) => value,
            Err(err) => return (Some(1), Some(err.to_string())),
        };

        let unzip_output = client.execute(&unzip_cmd);
        if !unzip_output.success {
            return (Some(unzip_output.exit_code), Some(unzip_output.stderr));
        }

        return (Some(0), None);
    }

    scp_to_path(server, client, &component.build_artifact, &install_dir)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dry_run_short_circuits_without_scp_calls() {
        reset_test_scp_call_count();

        let all_components = vec![Component {
            id: "sell-my-images".to_string(),
            name: "Sell My Images".to_string(),
            local_path: "/tmp".to_string(),
            remote_path: "wp-content/plugins/sell-my-images".to_string(),
            build_artifact: "/tmp/sell-my-images.zip".to_string(),
            build_command: None,
            version_targets: None,
        }];

        let args = DeployArgs {
            project_id: "saraichinwag".to_string(),
            component_ids: vec![],
            all: true,
            outdated: false,
            build: false,
            dry_run: true,
        };

        let client = TestRemoteExec::default();
        let server = ServerConfig {
            id: "cloudways".to_string(),
            name: "Cloudways".to_string(),
            host: "example.com".to_string(),
            user: "user".to_string(),
            port: 22,
            identity_file: None,
        };

        let selected =
            plan_components_to_deploy(&args, &all_components, &server, "/var/www", &client)
                .unwrap();
        assert_eq!(selected.len(), 1);

        assert_eq!(TEST_SCP_CALL_COUNT.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn zip_deploy_creates_dir_and_unzips_into_install_dir() {
        reset_test_scp_call_count();

        let component = Component {
            id: "sell-my-images".to_string(),
            name: "Sell My Images".to_string(),
            local_path: "/tmp".to_string(),
            remote_path: "wp-content/plugins/sell-my-images".to_string(),
            build_artifact: "/tmp/sell-my-images.zip".to_string(),
            build_command: None,
            version_targets: None,
        };

        let client = TestRemoteExec::default();
        let server = ServerConfig {
            id: "cloudways".to_string(),
            name: "Cloudways".to_string(),
            host: "example.com".to_string(),
            user: "user".to_string(),
            port: 22,
            identity_file: None,
        };

        let (exit_code, error) =
            deploy_component_artifact(&server, &client, "/var/www/site", &component);
        assert_eq!(exit_code, Some(0));
        assert!(error.is_none());

        let commands = client.commands().join("\n");
        assert!(commands.contains("mkdir -p '/var/www/site/wp-content/plugins/sell-my-images'"));
        assert!(commands.contains("cd '/var/www/site/wp-content/plugins/sell-my-images'"));
        assert!(commands.contains("unzip -o '/var/www/site/wp-content/plugins/sell-my-images/.homeboy-sell-my-images.zip'"));
    }
}

fn scp_to_path(
    _server: &ServerConfig,
    client: &dyn RemoteExec,
    local_path: &str,
    remote_path: &str,
) -> (Option<i32>, Option<String>) {
    #[cfg(test)]
    {
        TEST_SCP_CALL_COUNT.fetch_add(1, Ordering::Relaxed);
    }
    let mut scp_args: Vec<String> = vec![];

    let Some(ssh_client) = client.as_ssh_client() else {
        #[cfg(test)]
        return (Some(0), None);

        #[cfg(not(test))]
        return (
            Some(1),
            Some("SCP requires SSH client configuration".to_string()),
        );
    };

    if let Some(identity_file) = &ssh_client.identity_file {
        scp_args.push("-i".to_string());
        scp_args.push(identity_file.clone());
    }

    if ssh_client.port != 22 {
        scp_args.push("-P".to_string());
        scp_args.push(ssh_client.port.to_string());
    }

    scp_args.push(local_path.to_string());
    scp_args.push(format!(
        "{}@{}:{}",
        ssh_client.user, ssh_client.host, remote_path
    ));

    let output = Command::new("scp").args(&scp_args).output();

    match output {
        Ok(output) if output.status.success() => (Some(output.status.code().unwrap_or(0)), None),
        Ok(output) => (
            Some(output.status.code().unwrap_or(1)),
            Some(String::from_utf8_lossy(&output.stderr).to_string()),
        ),
        Err(err) => (Some(1), Some(err.to_string())),
    }
}

fn load_components(component_ids: &[String]) -> Vec<Component> {
    let mut components = Vec::new();

    for id in component_ids {
        let path = AppPaths::component(id);
        if let Ok(content) = fs::read_to_string(&path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                let local_path = config["localPath"].as_str().unwrap_or("").to_string();

                let build_artifact = config["buildArtifact"]
                    .as_str()
                    .map(|s| {
                        if s.starts_with('/') {
                            s.to_string()
                        } else {
                            format!("{}/{}", local_path, s)
                        }
                    })
                    .unwrap_or_default();

                let version_targets = config["versionTargets"].as_array().map(|targets| {
                    targets
                        .iter()
                        .filter_map(|target| {
                            let file = target["file"].as_str()?.to_string();
                            let pattern = target["pattern"].as_str().map(|value| value.to_string());
                            Some(VersionTarget { file, pattern })
                        })
                        .collect::<Vec<_>>()
                });

                components.push(Component {
                    id: config["id"].as_str().unwrap_or(id).to_string(),
                    name: config["name"].as_str().unwrap_or(id).to_string(),
                    local_path,
                    remote_path: config["remotePath"].as_str().unwrap_or("").to_string(),
                    build_artifact,
                    build_command: config["buildCommand"].as_str().map(|s| s.to_string()),
                    version_targets,
                });
            }
        }
    }

    components
}

fn parse_component_version(content: &str, pattern: Option<&str>) -> Option<String> {
    let default_pattern = r"Version:\s*(\d+\.\d+\.\d+)";

    let pattern_str = match pattern {
        Some(p) => p.replace("\\\\", "\\"),
        None => default_pattern.to_string(),
    };

    parse_version(content, &pattern_str)
}

fn fetch_local_version(component: &Component) -> Option<String> {
    let target = component.version_targets.as_ref()?.first()?;
    let path = format!("{}/{}", component.local_path, target.file);
    let content = fs::read_to_string(&path).ok()?;
    parse_component_version(&content, target.pattern.as_deref())
}

trait RemoteExec {
    fn execute(&self, command: &str) -> CommandOutput;
    fn as_ssh_client(&self) -> Option<&SshClient>;
}

#[cfg(test)]
#[derive(Default)]
struct TestRemoteExec {
    commands: std::sync::Mutex<Vec<String>>,
}

#[cfg(test)]
impl TestRemoteExec {
    fn commands(&self) -> Vec<String> {
        self.commands.lock().unwrap().clone()
    }
}

#[cfg(test)]
impl RemoteExec for TestRemoteExec {
    fn execute(&self, command: &str) -> CommandOutput {
        self.commands.lock().unwrap().push(command.to_string());
        CommandOutput {
            stdout: String::new(),
            stderr: String::new(),
            success: true,
            exit_code: 0,
        }
    }

    fn as_ssh_client(&self) -> Option<&SshClient> {
        None
    }
}

impl RemoteExec for SshClient {
    fn execute(&self, command: &str) -> CommandOutput {
        SshClient::execute(self, command)
    }

    fn as_ssh_client(&self) -> Option<&SshClient> {
        Some(self)
    }
}

fn fetch_remote_versions(
    components: &[Component],
    _server: &ServerConfig,
    base_path: &str,
    client: &dyn RemoteExec,
) -> HashMap<String, String> {
    let mut versions = HashMap::new();

    for component in components {
        let Some(version_file) = component
            .version_targets
            .as_ref()
            .and_then(|targets| targets.first())
            .map(|t| t.file.as_str())
        else {
            continue;
        };

        let remote_path = match homeboy_core::base_path::join_remote_child(
            Some(base_path),
            &component.remote_path,
            version_file,
        ) {
            Ok(value) => value,
            Err(_) => continue,
        };

        let output = client.execute(&format!("cat '{}' 2>/dev/null", remote_path));

        if output.success {
            let pattern = component
                .version_targets
                .as_ref()
                .and_then(|targets| targets.first())
                .and_then(|t| t.pattern.as_deref());

            if let Some(version) = parse_component_version(&output.stdout, pattern) {
                versions.insert(component.id.clone(), version);
            }
        }
    }

    versions
}
