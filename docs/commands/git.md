# `homeboy git`

## Synopsis

```sh
homeboy git <COMMAND>
```

## Subcommands

- `status <component_id>`
- `commit <component_id> <message>`
- `push <component_id> [--tags]`
- `pull <component_id>`
- `tag <component_id> <tag_name> [-m <message>]`

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "componentId": "<id>",
  "path": "<local path>",
  "action": "status|commit|push|pull|tag",
  "success": true,
  "exitCode": 0,
  "stdout": "<stdout>",
  "stderr": "<stderr>"
}
```

Notes:

- `commit` returns a successful result with `stdout` set to `Nothing to commit, working tree clean` when there are no changes.

## Exit code

Exit code matches the underlying `git` command.

## Related

- [version](version.md)
