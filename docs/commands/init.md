# `homeboy init`

## Synopsis

```sh
homeboy init [--plan | --apply] [--json] [--cwd <path>] [--scope <project|component|module>] \
  [--project-id <projectId>] [--component-id <componentId>] [--module-id <moduleId>]
```

`homeboy init` is a ceremony-elimination command intended to help users and coding agents initialize the current working directory as a Homeboy **project**, **component**, or **module** without making assumptions.

The command is designed to be called in **plan mode** first (read-only), then optionally executed with **apply mode**.

## Modes

- `--plan`: read-only; returns a deterministic plan of Homeboy commands required to initialize successfully.
- `--apply`: executes the plan; refuses to run when the output indicates ambiguity or missing inputs.

Notes:

- `--cwd` defaults to the current process working directory.
- When multiple scopes are plausible, `--scope` becomes required; Homeboy must not guess.

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). `homeboy init` returns an `InitOutput` object as the `data` payload.

### `InitOutput`

`InitOutput` is the canonical contract used by coding agents. Homeboy must never infer identifiers or choose among ambiguous matches.

```json
{
  "command": "init.plan",
  "version": "0.2.x",
  "cwd": "/abs/path",
  "scope": {
    "requested": null,
    "recommended": "component",
    "reason": "Resolved a single component candidate from cwd localPath matching.",
    "isAmbiguous": false,
    "choices": []
  },
  "resolution": {
    "project": {
      "match": "none",
      "resolvedId": null,
      "candidates": []
    },
    "component": {
      "match": "exact",
      "resolvedId": "homeboy-cli",
      "candidates": [
        {
          "id": "homeboy-cli",
          "localPath": "/abs/path",
          "match": "exact",
          "distance": 0
        }
      ]
    },
    "module": {
      "match": "unsupported",
      "resolvedId": null,
      "candidates": []
    }
  },
  "missingInputs": [],
  "checks": [
    {
      "name": "scope_selected",
      "status": "pass",
      "message": "Scope resolved to component"
    }
  ],
  "plan": [
    {
      "step": 1,
      "command": "homeboy component show homeboy-cli",
      "mode": "read",
      "reason": "Verify component exists and is readable.",
      "requires": []
    }
  ],
  "safeToApply": true
}
```

### Fields

- `command`: `init.plan` or `init.apply`
- `version`: Homeboy CLI version string (for agent compatibility)
- `cwd`: absolute path used for resolution
- `scope.requested`: optional user-provided scope (`project|component|module`)
- `scope.recommended`: `project|component|module|unknown`
- `scope.isAmbiguous`: `true` when Homeboy cannot choose safely; in that case `safeToApply` must be `false`
- `resolution.*.match`: one of:
  - `exact`: `cwd` matches an entity local path exactly
  - `ancestor`: `cwd` is inside an entity local path
  - `none`: no matches
  - `ambiguous`: multiple candidates; Homeboy must not pick
  - `unsupported`: Homeboy cannot resolve this entity type (e.g. no localPath concept)
- `resolution.*.resolvedId`: set only when match is `exact|ancestor` and not ambiguous
- `resolution.*.candidates`: ordered by smallest `distance` (closest match first)
- `missingInputs`: the authoritative list of required values needed to produce a safe plan or apply it
- `checks`: `pass|warn|fail|skip` results produced during init planning
- `plan`: ordered list of Homeboy commands to run (never non-Homeboy commands)
- `safeToApply`: `true` only when:
  - there is no ambiguity
  - `missingInputs` is empty
  - the plan can be executed deterministically

### Missing input keys (standardized)

Homeboy uses these keys as the only valid prompt contract for callers:

- `scope`
- `projectId`
- `projectType`
- `domain`
- `serverId`
- `projectLocalPath`
- `componentId`
- `componentLocalPath`
- `buildCommand`
- `versionFile`
- `versionPattern`
- `moduleId`
- `moduleSource`
- `moduleLocalPath`

## Exit code

- `0` on success.
- `1` on error.

## Related

- [project](project.md)
- [component](component.md)
- [module](module.md)
- [doctor](doctor.md)
- [JSON output contract](../json-output/json-output-contract.md)
