# Changelog

All notable changes to Homeboy CLI are documented in this file.

## 0.1.1

### Breaking Changes
- **Config Rename**: `local_cli` renamed to `local_environment` in project configuration JSON files (matches desktop app 0.7.0).

### Improvements
- **Deploy Command**: Improved deployment workflow.
- **Module Command**: Enhanced CLI module execution with better variable substitution.
- **PM2 Command**: Improved PM2 command handling for Node.js projects.
- **WP Command**: Improved WP-CLI command handling for WordPress projects.

## 0.1.0

Initial release.
- Project, server, and component management
- Remote SSH operations (wp, pm2, ssh, db, file, logs)
- Deploy and pin commands
- CLI module execution
- Shared configuration with desktop app
