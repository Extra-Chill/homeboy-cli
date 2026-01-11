# `homeboy doctor`

## Synopsis

```sh
homeboy doctor [OPTIONS]
```

## Description

Scan Homeboy configuration files and report issues (errors, warnings, info). This command always prints a JSON report to stdout.

## Options

- `--scope <SCOPE>`: Scope of configuration to scan (default: `all`)
  - Allowed values: `all`, `app`, `projects`, `servers`, `components`, `modules`
- `--file <PATH>`: Scan a specific JSON file path instead of a scope
- `--fail-on <LEVEL>`: Exit non-zero when issues at this severity exist (default: `error`)
  - Allowed values: `error`, `warning`

## JSON output

Homeboy wraps command output in the global JSON envelope described in [JSON output contract](../json-output/json-output-contract.md).

On success, `data` is a `DoctorReport` value:

```json
{
  "success": true,
  "data": {
    "command": "doctor.scan",
    "summary": {
      "filesScanned": 3,
      "issues": {
        "error": 1,
        "warning": 2,
        "info": 0
      }
    },
    "issues": [
      {
        "severity": "error",
        "code": "PROJECT.MISSING_SERVER",
        "message": "Project references unknown server id 'prod'",
        "file": "/path/to/projects/my-project.json",
        "pointer": "/config/serverId"
      }
    ]
  }
}
```

Notes:

- `severity` is lowercase: `error`, `warning`, `info`.
- `pointer` and `details` are optional and may be omitted.

## Exit codes

- `0`: no errors (and no warnings when `--fail-on warning` is used)
- `1`: errors found, or warnings found with `--fail-on warning`
