Changelog
=========

All notable changes to this project will be documented in this file.


## [unreleased](https://github.com/devrc-hub/devrc/releases/tag/vX.X.X) - XXXX-XX-XX

### Changes
_Changes of existing functionality_

### New Features

### Bugfixes
_For any bug fixes_


### Security
_Security vulnerabilities improvements_

### Build/Testing/Packaging
_All changes related to github actions, packages_

### Other
_Other cases_

---

## [v0.5.0](https://github.com/devrc-hub/devrc/releases/tag/v0.5.0) - 2023-05-29

### Changes
Refactored the logic of loading devrc files.

### New Features

Added the ability to load devrc files from local and remote files.
Added the ability to load environment variables from remote files.
Added a mechanism for running subtasks.
Added execution plugins system.


---

## [v0.4.0](https://github.com/devrc-hub/devrc/releases/tag/vX.X.X) - 2022-09-14

### Changes
- Deno updated to v1.25.2

### New Features
- Added variables modifiers

---


## [v0.3.0](https://github.com/devrc-hub/devrc/releases/tag/v0.3.0) - 2021-03-15

### Changes
- Changed CI
- Added xtests
- Refactored interpreter option

### New Features

- Added deno execution runtime


## [v0.2.0](https://github.com/devrc-hub/devrc/releases/tag/v0.2.0) - 2021-03-03

### Changes
- Changed format for tasks listing

### New Features

- Added task parameters


## [v0.1.0](https://github.com/devrc-hub/devrc/releases/tag/v0.1.0) - 2021-02-23

### Changes

- Added MVP (minimum viable product) with variables, environment variables and code execution.
- Stabilized loading process

### New Features

- All tasks can be listed from command line
- Shebang support for commands
- Task dependencies
- Default tasks
- Template engine and variables interpolation are supported
- Environment variables customization
- Command line completion scripts
- devrc supports dotenv files
- Writing task commands in different languages
- Read Devrcfile contents from stdin
- Global and local defined variables and environment variables
- Override variables by command line option
