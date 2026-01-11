# JSON output contract

Homeboy prints JSON to stdout for both success and error results.

## Top-level shape

Homeboy always prints a `homeboy_core::output::response::CliResponse<T>` object:

```json
{
  "success": true,
  "data": { "...": "..." }
}
```

or

```json
{
  "success": false,
  "error": {
    "code": "SOME_CODE",
    "message": "Human-readable message"
  }
}
```

## Where exit codes come from

- Each subcommand returns `(T, i32)` where `T` is a serializable payload type and `i32` is the intended process exit code.
- For successful commands, the process exit code is the returned `i32`, clamped to `0..=255`.
- For errors, the exit code is `1`.

## Success payload

On success, `data` is the command’s output struct (varies by command).

## Error payload

On error, `error.code` comes from `homeboy_core::Error::code()` and `error.message` is `Error::to_string()`.

## Command payload conventions

Many command output structs include a string field that identifies the action taken:

- Common fields: `command`
- Values often follow a dotted namespace (e.g. `project.show`, `server.key.generate`).

## Related

- Embedded docs outputs: [Docs command JSON](../commands/docs.md)
- Changelog output: [Changelog command JSON](../commands/changelog.md)
