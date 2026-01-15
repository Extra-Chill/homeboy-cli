# `homeboy release`

## Synopsis

```sh
homeboy release <COMMAND>
```

## Description

`homeboy release` plans release workflows based on the component-scoped `release` configuration.

## Subcommands

### `plan`

```sh
homeboy release plan <component_id>
```

Generates an ordered release plan without executing any steps.

Notes:

- Release config is read from the component (`components/<id>.json`).
- If no release config exists for the component, the command errors and suggests adding one via `homeboy component set`.
- Module actions are resolved from `component.modules`.

### `run`

```sh
homeboy release run <component_id>
```

Executes the release pipeline steps defined in the component `release` block.

Notes:

- Steps run in parallel when dependencies allow it.
- Any step depending on a failed/missing step is skipped.
- Release actions use module definitions configured in `component.modules`.

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "command": "release.plan",
  "plan": {
    "component_id": "<component_id>",
    "enabled": true,
    "steps": [
      {
        "id": "build",
        "type": "build",
        "label": "Build",
        "needs": [],
        "config": {},
        "status": "ready",
        "missing": []
      }
    ],
    "warnings": [],
    "hints": []
  }
}
```

```json
{
  "command": "release.run",
  "run": {
    "component_id": "<component_id>",
    "enabled": true,
    "result": {
      "status": "success",
      "warnings": [],
      "steps": [
        {
          "id": "build",
          "type": "build",
          "status": "success",
          "missing": [],
          "warnings": [],
          "hints": [],
          "data": {}
        }
      ]
    }
  }
}
```

## Related

- [component](component.md)
- [module](module.md)

