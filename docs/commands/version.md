# `homeboy version`

## Synopsis

```sh
homeboy version <COMMAND>
```

## Subcommands

### `show`

```sh
homeboy version show <component_id>
```

### `bump`

```sh
homeboy version bump <component_id> <patch|minor|major>
```

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). `homeboy version` returns a `VersionOutput` object as the `data` payload.

`VersionOutput`:

- `command`: `version.show` | `version.bump`
- `componentId`
- `versionFile`
- `versionPattern`
- `fullPath`
- `version` (for `show`)
- `oldVersion`, `newVersion` (for `bump`)

## Exit code

- `show`: `0` on success; errors if the version cannot be parsed.
- `bump`: `0` on success.

## Notes

- Components must have `version_file` configured.
- `version_pattern` is optional; when omitted, a default pattern is selected based on the configured `version_file` name.

## Related

- [build](build.md)
- [component](component.md)
- [git](git.md)
