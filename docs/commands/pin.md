# `homeboy pin`

## Synopsis

```sh
homeboy pin <COMMAND>
```

## Subcommands

- `list <project_id> --type <file|log>`
- `add <project_id> <path> --type <file|log> [--label <label>] [--tail <lines>]`
- `remove <project_id> <path> --type <file|log>`

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). `homeboy pin` returns a `PinOutput` object as the `data` payload.

`PinOutput`:

- `command`: `pin.list` | `pin.add` | `pin.remove`
- `projectId`
- `type`: `file` | `log`
- `items`: present for `list`
- `added`: present for `add`
- `removed`: present for `remove`

List item (`items[]`):

- `path`
- `label`
- `displayName`
- `tailLines` (logs only)

Change object (`added`/`removed`):

- `path`
- `type`

## Related

- [file](file.md)
- [logs](logs.md)
