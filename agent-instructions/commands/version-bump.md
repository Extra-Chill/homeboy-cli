---
name: version-bump
description: Bump component version and update changelog via Homeboy.
version: 0.1.0
allowed-tools: Bash(homeboy *)
---

# Version bump

Use Homeboy to update the version and changelog together. Do not manually edit changelog files.

## Workflow

1. `homeboy component show <component_id>`
2. `homeboy version show <component_id>`
3. Review changes since last version tag:

```sh
homeboy changes <component_id>
```

4. Based on the changes output, decide bump interval: `patch|minor|major`
5. Add changelog entries:

```sh
homeboy changelog add --json '{"component_id":"<component_id>","messages":["<change 1>","<change 2>"]}'
# (Alternative non-JSON mode)
# homeboy changelog add <component_id> "<change 1>"
```

6. Bump version and finalize changelog:

```sh
homeboy version bump <component_id> <patch|minor|major>
```

7. `homeboy build <component_id>`
8. `homeboy git commit <component_id> "Bump version to X.Y.Z"`
9. `homeboy git push <component_id>`

## Notes

- Ask the user if you should also use `homeboy git tag` and `homeboy git push <component_id> --tags` 
