# workspacer

A CLI tool (`ws`) for managing related git worktrees in a dedicated workspace folder.
Uses [worktrunk](https://github.com/max-sixty/worktrunk) (`wt`) to create and manage worktrees across multiple repos.

## Prerequisites

- [worktrunk](https://github.com/max-sixty/worktrunk) must be installed and available as `wt` in your PATH.

## Installation

```sh
cargo install --path .
```

This installs the `ws` binary into your Cargo bin directory (usually `~/.cargo/bin/`).

During development you can also run it via:

```sh
cargo run -- <command>
```

## Shell integration

Add this to your `~/.zshrc` (or `~/.bashrc`) so that `ws switch` changes your shell's directory:

```sh
eval "$(ws shell-init)"
```

## Setup

Create a template that defines which repos belong together:

```sh
ws template add my-project --repo /path/to/repo-a --repo /path/to/repo-b
```

Optionally set a custom workspace directory (defaults to `~/workspaces`):

```sh
ws config --workspace-dir /path/to/workspaces
```

## Usage

```
ws new <name> [-t <template>]     # Create worktrees via `wt switch --create`
ws switch [name]                  # Switch to a workspace (TUI picker if name omitted)
ws list                           # List all workspaces
ws remove <name> [-t <template>]  # Remove worktrees via `wt remove` and clean up
```

If only one template exists, it is used automatically. Otherwise pass `-t <template>`.

### Managing templates

```
ws template list                          # List all templates
ws template add <name> --repo <path> ...  # Create or extend a template
ws template remove <name>                 # Delete a template
ws template remove <name> --repo <path>   # Remove specific repos from a template
ws template show <name>                   # Show repos in a template
```

### Configuration

```
ws config                         # Show current configuration
ws config --workspace-dir <path>  # Set the workspace directory
```

Config is stored at `~/.config/workspacer/config.json`.

| Key             | Default          | Description                                  |
|-----------------|------------------|----------------------------------------------|
| `workspace_dir` | `~/workspaces`   | Directory where workspaces live              |
| `templates`     | `{}`             | Named sets of repo paths for worktree creation |
