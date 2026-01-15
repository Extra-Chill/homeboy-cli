# `homeboy deploy`

## Synopsis

```sh
homeboy deploy <projectId> [<componentIds...>] [-c|--component <id>]... [--all] [--outdated] [--json '<spec>']
# If no component IDs are provided, you must use --all or --outdated.
```

## Arguments and flags

- `projectId`: project ID
- `<componentIds...>` (optional): component IDs to deploy (positional, trailing)

Options:

- `-c`, `--component`: component ID to deploy (can be repeated, alternative to positional)
- `--all`: deploy all configured components
- `--outdated`: deploy only outdated components
  - Determined from the first version target for each component.
- `--json`: JSON input spec for bulk operations (`{"component_ids": ["component-id", ...]}`)

Positional and flag component IDs can be mixed; both are merged into the deployment list.

If no component IDs are provided and neither `--all` nor `--outdated` is set, Homeboy returns an error.

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is `data`.

```json
{
  "command": "deploy.run",
  "project_id": "<projectId>",
  "all": false,
  "outdated": false,
  "results": [
    {
      "id": "<componentId>",
      "name": "<name>",
      "status": "deployed|failed|skipped",
      "deploy_reason": "explicitly_selected|all_selected|version_mismatch|unknown_local_version|unknown_remote_version",
      "local_version": "<v>|null",
      "remote_version": "<v>|null",
      "error": "<string>|null",
      "artifactPath": "<path>|null",
      "remote_path": "<path>|null",
      "build_command": "<cmd>|null",
      "build_exit_code": "<int>|null",
      "deploy_exit_code": "<int>|null"
    }
  ],
  "summary": { "succeeded": 0, "failed": 0, "skipped": 0 }
}
```

Notes:

- `deployReason` is omitted when not applicable.
- `artifactPath` is the component build artifact path as configured; it may be relative.

Note: `buildExitCode`/`deployExitCode` are numbers when present (not strings).

Exit code is `0` when `summary.failed == 0`, otherwise `1`.

## Exit code

- `0` when all selected component deploys succeed.
- `1` when any component deploy fails.

## Related

- [build](build.md)
- [component](component.md)
