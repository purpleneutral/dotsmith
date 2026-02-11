# dotsmith

[![CI](https://github.com/purpleneutral/dotsmith/actions/workflows/ci.yml/badge.svg)](https://github.com/purpleneutral/dotsmith/actions/workflows/ci.yml)
[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.1.0--alpha.8-orange.svg)](CHANGELOG.md)

The dotfile workbench -- explore, manage, and master your configs.

dotsmith is a CLI + TUI tool that does what no other dotfile manager does: it shows you what config options exist, which you're using, which you're missing, and teaches you what each one does -- all while handling snapshots, rollback, plugins, and deployment.

## Features

- **Explore** config options with descriptions, examples, and documentation URLs
- **Track changes** with snapshots, diffs, and one-command rollback
- **Interactive TUI** -- dashboard, option explorer, diff viewer, history browser
- **Plugin management** for zsh and tmux -- no framework needed
- **Profiles** -- save and restore named configuration sets
- **Remote deploy** tracked configs to any machine via SSH
- **Git repo sync** for version-controlled dotfile backups
- **7 Tier 1 tools** with 270+ curated options: tmux, zsh, git, kitty, neovim, alacritty, awesomewm
- **Tier 2 auto-detection** for any tool with a config file

## Install

```sh
curl -sSf https://raw.githubusercontent.com/purpleneutral/dotsmith/main/install.sh | sh
```

Or with cargo: `cargo install --git https://github.com/purpleneutral/dotsmith.git`

See [Getting Started](docs/getting-started.md) for all install methods, shell completions, and man page setup.

## Quick Start

```sh
dotsmith add tmux      # start tracking a tool (auto-initializes on first use)
dotsmith add zsh
dotsmith                # launch the TUI dashboard (press 'a' to add more tools)
dotsmith explore tmux   # explore config options
```

## Documentation

| Guide | Description |
|-------|-------------|
| [Getting Started](docs/getting-started.md) | Installation, first run, how it works |
| [Command Reference](docs/commands.md) | All CLI commands with flags and examples |
| [TUI Guide](docs/tui.md) | Dashboard, explorer, diff, history, plugins |
| [Plugin Management](docs/plugins.md) | Adding, managing, and exploring plugins |
| [Snapshots & History](docs/snapshots-and-history.md) | Snapshots, diffs, rollback, file watching |
| [Deploy & Profiles](docs/deploy-and-profiles.md) | Local/remote deploy, profiles, repo sync |
| [Configuration](docs/configuration.md) | config.toml, manifest.toml, environment variables |
| [Supported Tools](docs/supported-tools.md) | Tier 1/2 tools, option counts, validation |
| [Contributing](docs/contributing.md) | Development setup, code guidelines, architecture |

## Project Status

**Current:** v0.1.0-alpha.8 -- See [CHANGELOG.md](CHANGELOG.md) for detailed release notes.

## Support

If you find dotsmith useful, consider buying me a coffee:

[![Buy Me A Coffee](https://img.shields.io/badge/Buy%20Me%20A%20Coffee-support-yellow?style=flat&logo=buy-me-a-coffee)](https://buymeacoffee.com/uniqueuserg)

## License

Copyright (c) 2026 purpleneutral

This program is free software: you can redistribute it and/or modify it under
the terms of the GNU General Public License as published by the Free Software
Foundation, version 3.

This software is provided **as-is, without warranty of any kind**. dotsmith
creates backups before write operations (rollback, deploy, profile load), but
you are responsible for verifying changes with `--dry-run` before applying them.
The authors are not liable for any data loss or damage. See [LICENSE](LICENSE)
for the full terms.
