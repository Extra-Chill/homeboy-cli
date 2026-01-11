# `homeboy changelog`

## Synopsis

```sh
homeboy changelog
```

## Description

Returns the embedded documentation content for the `changelog` topic.

This command expects the embedded docs key `changelog` to exist (from `docs/changelog.md`).

## JSON output (success)

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "topic_label": "changelog",
  "content": "<markdown content>"
}
```

## Errors

If embedded docs do not contain `changelog`, the command returns an error message:

- `No changelog found (expected embedded docs topic 'changelog')`

## Related

- [Docs command](docs.md)
- [Embedded docs topic resolution](../embedded-docs/embedded-docs-topic-resolution.md)
- [Changelog content](../changelog.md)
