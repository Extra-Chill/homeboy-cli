# `homeboy component`

Manage standalone component configurations stored under `components/<id>.json`.

## Synopsis

```sh
homeboy component [OPTIONS] <COMMAND>
```


## Subcommands

### `create`

```sh
homeboy component create [OPTIONS] --local-path <path> --remote-path <path> --build-artifact <path>
```

The component ID is derived from the `--local-path` directory name (lowercased). For example, `--local-path /path/to/extrachill-api` creates a component with ID `extrachill-api`.

Options:

- `--json <spec>`: JSON input spec for create/update (supports single or bulk)
- `--skip-existing`: skip items that already exist (JSON mode only)
- `--local-path <path>`: absolute path to local source directory (required; ID derived from directory name; `~` is expanded)
- `--remote-path <path>`: remote path relative to project `basePath` (required)
- `--build-artifact <path>`: build artifact path relative to `localPath` (required)
- `--version-target <TARGET>`: version target in format `file` or `file::pattern` (repeatable)
- `--build-command <command>`: build command to run in `localPath`
- `--extract-command <command>`: command to run after upload (optional; supports `{artifact}` and `{targetDir}`)

### `show`

```sh
homeboy component show <id>
```

### `set`

```sh
homeboy component set <id> --json <JSON>
```

Updates a component by merging a JSON object into `components/<id>.json`.

Options:

- `--json <JSON>`: JSON object to merge into config (supports `@file` and `-` for stdin)

Notes:

- `set` no longer supports individual field flags; use `--json` and provide the fields you want to update.

### `delete`

```sh
homeboy component delete <id>
```

Deletion is safety-checked:

- If the component is referenced by one or more projects, the command errors and asks you to remove it from those projects first.

### `rename`

```sh
homeboy component rename <id> <new-id>
```

Renames a component by changing its ID directly and rewriting any project files that reference the old ID. Use this to migrate components to match their repository directory names.

Example:

```sh
# Rename to match repository directory name
homeboy component rename extra-chill-api extrachill-api
```

### `list`

```sh
homeboy component list
```

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

`homeboy component` returns a `ComponentOutput` object.

```json
{
  "action": "create|show|set|delete|rename|list|component.create",
  "componentId": "<id>|null",
  "success": true,
  "updatedFields": ["localPath", "remotePath"],
  "component": { },
  "components": [ ],
  "import": null
}
```

Notes:

- `action` is `component.create` only for JSON import mode (`homeboy component create --json ...`).
- Other subcommands use `create`, `show`, `set`, `delete`, `rename`, or `list`.


## Related

- [build](build.md)
- [deploy](deploy.md)
- [project](project.md)
- [JSON output contract](../json-output/json-output-contract.md)
