# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.2](https://github.com/seakayone/workspacer/compare/v0.2.1...v0.2.2) - 2026-04-11

### Other

- add actions:write permission for workflow_dispatch

## [0.2.1](https://github.com/seakayone/workspacer/compare/v0.2.0...v0.2.1) - 2026-04-11

### Other

- remove manual release instructions
- support workflow_dispatch for release builds
- trigger release workflow on release-plz tag creation

## [0.2.0](https://github.com/seakayone/workspacer/compare/v0.1.0...v0.2.0) - 2026-04-11

### Added

- *(build)* detect dirty working tree in version string
- add dynamic versioning with git commit metadata

### Other

- *(deps)* update github-actions dependencies ([#6](https://github.com/seakayone/workspacer/pull/6))
- *(deps)* update cargo non-major dependencies ([#5](https://github.com/seakayone/workspacer/pull/5))
- *(renovate)* set mode to full for complete dependency updates
- *(renovate)* add GitHub Actions workflow for automated dependency updates
- *(renovate)* change schedule from "every monday" to "on monday"
- add Renovate configuration for dependency updates
- *(release-plz)* clarify binary tool release config
- remove workspace_dir helper function
- set up release-plz for automated versioning and releases
- reorder imports in main.rs
- *(build)* move cargo_version into build_version
- simplify nested conditionals with guard clauses
