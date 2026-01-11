# JSON output contract

Homeboy prints JSON to stdout for both success and error results.

## Top-level shape

Homeboy always prints a `homeboy_core::output::response::CliResponse<T>` object.

Success:

```json
{
  "success": true,
  "data": { "...": "..." }
}
```

Failure:

```json
{
  "success": false,
  "error": {
    "code": "SOME_CODE",
    "message": "Human-readable message"
  }
}
```

Notes:

- `data` is omitted on failure.
- `error` is omitted on success.

## Where exit codes come from

- Each subcommand returns `Result<(T, i32)>` where `T` is the success payload and `i32` is the intended process exit code.
- On success, the process exit code is the returned `i32`, clamped to `0..=255`.
- On error, the process exit code is `1`.

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
