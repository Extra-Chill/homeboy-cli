use clap::{Args, ValueEnum};
use serde::Serialize;

use homeboy_core::doctor::{Doctor, DoctorScope, FailOn};

#[derive(Args)]
pub struct DoctorArgs {
    /// Scope of configuration to scan
    #[arg(long, value_enum, default_value_t = ScopeArg::All)]
    pub scope: ScopeArg,

    /// Scan a specific JSON file path instead of a scope
    #[arg(long)]
    pub file: Option<String>,

    /// Fail with non-zero exit if warnings are found
    #[arg(long, value_enum, default_value_t = FailOnArg::Error)]
    pub fail_on: FailOnArg,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum ScopeArg {
    All,
    App,
    Projects,
    Servers,
    Components,
    Modules,
}

#[derive(Clone, Copy, ValueEnum)]
pub enum FailOnArg {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DoctorOutput {
    #[serde(flatten)]
    pub report: homeboy_core::doctor::DoctorReport,
}

pub fn run(args: DoctorArgs) -> homeboy_core::Result<(DoctorOutput, i32)> {
    let scan_result = if let Some(path) = args.file.as_deref() {
        Doctor::scan_file(std::path::Path::new(path))?
    } else {
        Doctor::scan(scope_to_core(args.scope))?
    };

    let exit_code = Doctor::exit_code(&scan_result, fail_to_core(args.fail_on));

    Ok((
        DoctorOutput {
            report: scan_result.report,
        },
        exit_code,
    ))
}

fn scope_to_core(scope: ScopeArg) -> DoctorScope {
    match scope {
        ScopeArg::All => DoctorScope::All,
        ScopeArg::App => DoctorScope::App,
        ScopeArg::Projects => DoctorScope::Projects,
        ScopeArg::Servers => DoctorScope::Servers,
        ScopeArg::Components => DoctorScope::Components,
        ScopeArg::Modules => DoctorScope::Modules,
    }
}

fn fail_to_core(fail_on: FailOnArg) -> FailOn {
    match fail_on {
        FailOnArg::Error => FailOn::Error,
        FailOnArg::Warning => FailOn::Warning,
    }
}
