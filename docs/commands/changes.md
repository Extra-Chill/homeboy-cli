# `homeboy changes`

## Synopsis

```sh
homeboy changes <componentId> [--since <tag>] [--git-diffs]
homeboy changes --cwd [--git-diffs]
homeboy changes --json <spec> [--git-diffs]
homeboy changes --project <projectId> [--git-diffs]
```

## Description

Show changes since the latest git tag for one component, multiple components (bulk JSON), all components attached to a project, or the current working directory.

This command reports:

- commits since the last tag (or a user-provided tag via `--since`)
- uncommitted changes in the working tree (including `uncommittedDiff`)
- optionally, a commit-range diff for commits since the baseline (via `--git-diffs`)

## Options

- `--cwd`: use current working directory (ad-hoc mode, no component registration required)
- `--since <tag>`: tag name to compare against (single-component mode)
- `--git-diffs`: include commit-range diff content in output
- `--json <spec>`: bulk mode input
  - `<spec>` supports `-` (stdin), `@file.json`, or an inline JSON string
- `--project <projectId>`: show changes for all components attached to a project

## JSON output

> Note: all command output is wrapped in the global JSON envelope described in the [JSON output contract](../json-output/json-output-contract.md). `homeboy changes` returns either a single `ChangesOutput` or a bulk `BulkChangesOutput` as `data`.

### Single-component output

```json
{
  "componentId": "<componentId>",
  "path": "<local path>",
  "success": true,
  "latestTag": "<tag>|null",
  "baselineSource": "tag|version_commit|last_n_commits",
  "baselineRef": "<ref>|null",
  "commits": [
    {
      "hash": "<sha>",
      "subject": "<subject>",
      "author": "<author>",
      "date": "<date>"
    }
  ],
  "uncommitted": {
    "hasChanges": true,
    "staged": ["..."],
    "unstaged": ["..."],
    "untracked": ["..."]
  },
  "uncommittedDiff": "<diff>",
  "diff": "<diff>"
}
```

Notes:

- `uncommittedDiff` is present when the working tree has changes.
- `diff` is included only when `--git-diffs` is used.
- Optional fields like `warning` / `error` may be omitted when unset.
}
```

### Bulk output (`--json` or `--project`)

```json
{
  "results": [ { "id": "<componentId>", "result": { ... }, "error": "<string>|null" } ],
  "summary": {
    "total": 2,
    "withCommits": 1,
    "withUncommitted": 1,
    "clean": 0,
    "failed": 0
  }
}
```

(Exact `commits[]` and `summary` fields are defined by the CLI output structs.)

## Exit code

- `0` when the command succeeds and `summary.failed == 0`.
- `1` in bulk/project modes when `summary.failed > 0`.

> Note: single-target modes (`<componentId>` and `--cwd`) always return exit code `0` on success, even when the underlying git operations report `success: false` in the output.

## Related

- [git](git.md)
- [version](version.md)
