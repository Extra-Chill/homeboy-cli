# `homeboy docs`

## Synopsis

```sh
homeboy docs [<topic>...]
```

## Description

Returns embedded documentation content for a topic.

- Topic arguments are treated as a free-form trailing list.
- The resolved key must exist in embedded docs; otherwise the command errors.

Topic resolution is documented in: [Embedded docs topic resolution](../embedded-docs/embedded-docs-topic-resolution.md).

## Arguments

- `<topic>...` (optional): documentation topic (examples in CLI help: `deploy`, `project set`).

## JSON output (success)

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). The object below is the `data` payload.

```json
{
  "topic": "<original topic as a single space-joined string>",
  "topic_label": "<same as topic, or 'index' when omitted>",
  "content": "<markdown content>",
  "available_topics": "<newline-separated list of embedded keys>"
}
```

### Fields

- `topic`: raw user input joined by spaces.
- `topic_label`: label returned by the resolver (`index` when no topic args are provided).
- `content`: embedded markdown content.
- `available_topics`: newline-separated list of available embedded keys.

## Errors

If resolved content is empty, the command returns an error message:

- `No documentation found for '<topic>' (available: <available_topics>)`

`<available_topics>` is a newline-separated list.

## Related

- [Changelog command](changelog.md)
- [JSON output contract](../json-output/json-output-contract.md)
