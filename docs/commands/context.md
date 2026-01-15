# `homeboy context`

## Synopsis

```sh
homeboy context
homeboy context --discover [--depth <n>]
```

## Description

Prints a JSON payload describing the current working directory context:

- current directory (`cwd`)
- detected git root (if any)
- whether the directory matches any configured component local paths

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is `data`.

```json
{
  "command": "context.show",
  "cwd": "/absolute/path",
  "git_root": "/absolute/git/root",
  "managed": true,
  "matched_components": ["component_id"],
  "suggestion": null
}
```

Payload shape:

```json
{ "command": "context.show", "cwd": "...", "git_root": "...", "managed": true, "matched_components": [], "suggestion": null }
```

### Fields

- `command` (string): `context.show` (or `context.discover` when using `--discover`)
- `cwd` (string): current working directory
- `gitRoot` (string|null): `git rev-parse --show-toplevel` when available
- `managed` (bool): `true` when `matchedComponents` is non-empty
- `matchedComponents` (string[]): component IDs whose `localPath` matches `cwd` (exact match or ancestor)
- `suggestion` (string|null): guidance when `managed` is `false`

## Repository discovery (`--discover`)

When `--discover` is used, Homeboy scans subdirectories (default depth: `2`) and returns a list of git repositories plus whether they are managed (match a configured component).

JSON payload (as `data`) is a `DiscoverOutput`:

- `command`: `context.discover`
- `basePath`: base directory used for discovery
- `depth`: max depth
- `repos`: array of `{ path, name, isManaged, matchedComponent }`

## Related

- [init](init.md)
- [component](component.md)
- [project](project.md)
