# `homeboy project`

## Synopsis

```sh
homeboy project <COMMAND>
```

## Subcommands

### `show`

```sh
homeboy project show [project_id]
```

- `project_id` (optional): if omitted, uses the active project.

JSON output:

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "command": "project.show",
  "projectId": "<id>",
  "project": { }
}
```

`project` is the serialized `ProjectConfiguration`.

### `switch`

```sh
homeboy project switch <project_id>
```

JSON output:

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "command": "project.switch",
  "projectId": "<id>",
  "project": { }
}
```

## Related

- [projects](projects.md)
- [JSON output contract](../json-output/json-output-contract.md)
