# Contributing to dotsmith

Thanks for your interest in contributing to dotsmith.

## Getting Started

1. Fork and clone the repo
2. Install Rust 1.85+ via [rustup](https://rustup.rs)
3. Run `make check` to verify everything builds and tests pass

## Development

```sh
cargo build              # build
cargo test               # run tests
cargo clippy             # lint
make check               # clippy + tests together
```

## Guidelines

- Run `cargo clippy -- -D warnings` before submitting -- CI enforces this
- Add tests for new functionality (unit tests in-module, integration tests in `tests/`)
- Keep error handling consistent: `anyhow` for CLI boundaries, `thiserror` for library errors
- No `unwrap()` on user data paths -- use proper error handling
- No shell injection -- use `Command::new()` with explicit args, never `sh -c`
- Atomic writes for any file dotsmith creates (write to `.tmp` then rename)
- New config files must have `0600` permissions

## Adding a Tier 1 Module

1. Create `data/modules/<tool>/module.toml` with metadata
2. Create `data/modules/<tool>/options.toml` with curated options
3. Register in `src/core/module.rs` `ModuleRegistry` (add `include_str!` entries)
4. Add detection logic in `src/core/detect.rs` if the tool has a plugin manager
5. Add tests

See `data/modules/tmux/` for a complete example.

## Commit Style

```
feat: short description of the change
fix: what was broken and how it was fixed
```

## License

By contributing, you agree that your contributions will be licensed under the GPL-3.0.
