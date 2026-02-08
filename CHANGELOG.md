# Changelog

All notable changes to dotsmith will be documented in this file.

## [0.1.0-alpha.1] - 2026-02-08

### Added

- **Phase 1: Foundation**
  - CLI skeleton with clap: `init`, `add`, `remove`, `list`, `status`
  - Manifest system for tracking tools (`manifest.toml`)
  - Module system with `include_str!` embedded Tier 1 data
  - Symlink-aware config detection (records user-facing paths, not targets)
  - Plugin manager detection (TPM, zinix-mgr, oh-my-zsh, lazy.nvim)
  - Tier 2 auto-detection for any tool
  - tmux Tier 1 module with 31 curated options
  - Path safety checks (symlinks must resolve within `$HOME`)
  - Atomic file writes with `0600` permissions

- **Phase 2: Snapshots and Management**
  - SQLite snapshot engine with SHA-256 content dedup and WAL mode
  - `snapshot` command with per-tool or all-tools support
  - `history` command with snapshot listing
  - `diff` command with colored unified output (via `similar` crate)
  - `rollback` command with automatic backup and `--dry-run`
  - `deploy` command for symlink creation with backup and `--dry-run`
  - `reload` command (tmux source-file, kitty SIGUSR1, i3/sway reload)
  - zsh Tier 1 module with 31 curated options
  - Shared `atomic_write` utility

- **Install System**
  - Makefile with `install`, `uninstall`, `test`, `check`, `clean` targets
  - Smart `install.sh` (prebuilt binary download with source build fallback)
  - GitHub Actions CI (clippy + tests on push/PR)
  - GitHub Actions release workflow (binary on tag push)
