# Contributing

Thanks for your interest in contributing to dotsmith.

## Getting Started

1. Fork and clone the repo
2. Install Rust 1.85+ via [rustup](https://rustup.rs)
3. Run `make check` to verify everything builds and tests pass

## Development Commands

```sh
cargo build              # debug build
cargo build --release    # optimized build
cargo test               # run all tests (~450)
cargo clippy             # lint
make check               # clippy + tests together
make install             # build + install binary + man page
```

## Project Structure

```
src/
  main.rs                # CLI entry point, command dispatch
  lib.rs                 # Library root
  cli/                   # One file per CLI command
    mod.rs               # Command definitions (clap structs)
    add.rs, remove.rs, list.rs, status.rs, init.rs
    snapshot.rs, history.rs, diff.rs, rollback.rs
    deploy.rs, deploy_remote.rs, reload.rs
    plugins.rs, profile.rs, repo.rs
    search.rs, doctor.rs, edit.rs, watch.rs
  core/                  # Business logic
    manifest.rs          # Tool tracking (manifest.toml)
    config.rs            # App settings (config.toml)
    module.rs            # Tier 1 option databases (include_str!)
    snapshot.rs          # SQLite snapshot engine
    plugin.rs            # Plugin management (clone, loader, update)
    plugin_info.rs       # README scanning for plugin info
    detect.rs            # Tool and plugin manager detection
    deploy.rs            # Local symlink deployment
    remote.rs            # SSH/SCP remote deployment
    profile.rs           # Profile save/load
    repo.rs              # Git repo sync
    reload.rs            # Tool reload commands
    validate.rs          # Config syntax validation
    errors.rs            # Error types (thiserror)
  tui/                   # Interactive terminal UI (ratatui)
    mod.rs               # App struct, view routing, event loop
    dashboard/           # Dashboard view (tool list + actions)
    explore/             # Option explorer (3-panel layout)
    diff/                # Diff viewer
    history/             # Snapshot history browser
    plugins/             # Plugin management view
    widgets/             # Shared widgets (help bar, status bar)
  util/                  # Shared utilities
    paths.rs             # Tilde expansion/contraction
    fs.rs                # Atomic write, file operations
    diff.rs              # Unified diff generation
data/
  modules/               # Tier 1 tool definitions
    tmux/                # module.toml + options.toml
    zsh/
    git/
    kitty/
    neovim/
    alacritty/
    awesomewm/
tests/                   # Integration tests
  init_test.rs, add_test.rs, remove_test.rs, list_test.rs
  snapshot_test.rs, plugin_test.rs, profile_test.rs, remote_test.rs
```

## Code Guidelines

### Error Handling

- `anyhow::Result` for CLI functions (top-level error reporting)
- `thiserror` for library error types in `core/errors.rs`
- No `unwrap()` on user data or file paths -- use proper error handling
- Return errors, don't panic

### Security

- No shell injection -- always use `Command::new()` with explicit args, never `sh -c`
- Atomic writes for any file dotsmith creates (write to `.tmp` then rename)
- All new files must have `0600` permissions (via `util::fs::atomic_write`)
- Path safety checks before symlink/deploy operations

### Testing

- Unit tests go in `#[cfg(test)] mod tests` inside the source file
- Integration tests go in `tests/` using `assert_cmd` and `predicates`
- Use `DOTSMITH_CONFIG_DIR` env var for test isolation (each test gets its own `TempDir`)
- Plugin tests use local `file://` git repos in TempDir (no network access)
- Deploy tests use `TempDir::new_in($HOME)` to pass path safety checks
- Remote tests gate SSH-dependent assertions with fallback error checks for CI

### Style

- `cargo clippy -- -D warnings` must pass (CI enforces this)
- Rust 2024 edition -- `set_var`/`remove_var` are `unsafe`, use let-chains where applicable
- Library functions take `&Path` parameters; env var resolution only at CLI boundary
- Use `symlink_metadata()` to detect without following symlinks, `metadata()` to follow
- Use `contract_tilde`/`expand_tilde` for portable path storage

## Adding a Tier 1 Module

1. **Create module definition**: `data/modules/<tool>/module.toml`
   - Set metadata: name, display_name, description, homepage
   - List config_paths, detect_command, reload_command (if applicable)
   - Set config_format, plugins_supported
   - Define categories

2. **Create option database**: `data/modules/<tool>/options.toml`
   - Add `[[options]]` entries with: name, type, default, category, description, example
   - Optional: why, tags, related, url, since, values (for enums)

3. **Register in `src/core/module.rs`**: Add `include_str!` entries for both files in the `ModuleRegistry`

4. **Add detection logic**: If the tool has a plugin manager, add detection in `src/core/detect.rs`

5. **Add tests**: Unit tests for the new module's option parsing, integration test for `dotsmith add <tool>`

See `data/modules/tmux/` for a complete reference example.

## Adding Plugin Options to an Existing Module

1. Add new `plugin:*` categories to `module.toml`
2. Add `[[options]]` entries in `options.toml` with the plugin category and a `url` field
3. Verify with `cargo test` (option count assertions use `>=` to accommodate additions)

## Commit Style

```
feat: short description of new functionality
fix: what was broken and how it was fixed
docs: documentation changes
```

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0.

## See Also

- [Supported Tools](supported-tools.md) -- Tier 1/2 system and per-tool details
- [Configuration](configuration.md) -- file formats and structure
