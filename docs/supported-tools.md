# Supported Tools

dotsmith uses a tiered support system. Tier 1 tools have curated option databases for exploration and search. Tier 2 tools are auto-detected and get full snapshot/diff/rollback support without a curated database.

## Tier System

| Feature | Tier 1 | Tier 2 |
|---------|--------|--------|
| Config path detection | Curated paths | Auto-detected |
| Snapshot, diff, rollback | Yes | Yes |
| Deploy (local and remote) | Yes | Yes |
| Option explorer (`explore`) | Yes | No |
| Option search (`search`) | Yes | No |
| Config generation (`g` key) | Yes | No |
| Syntax validation (`doctor`) | Yes (format-dependent) | No |
| Plugin management | zsh, tmux only | No |

## Tier 1 Tools

### tmux

Terminal multiplexer.

| Property | Value |
|----------|-------|
| **Options** | 55 (31 native + 24 plugin) |
| **Categories** | appearance, behavior, interaction, keybindings, performance, status-bar, windows-panes, plugin:resurrect, plugin:continuum, plugin:yank, plugin:catppuccin, plugin:prefix-highlight, plugin:fingers, plugin:sensible |
| **Config paths** | `~/.config/tmux/tmux.conf`, `~/.tmux.conf` |
| **Reload** | `tmux source-file <path>` |
| **Config format** | tmux (validated by `doctor`) |
| **Plugins** | Supported (TPM detected) |
| **Homepage** | [github.com/tmux/tmux](https://github.com/tmux/tmux) |

### Zsh

Z Shell -- a powerful interactive shell with extensive customization.

| Property | Value |
|----------|-------|
| **Options** | 61 (33 native + 28 plugin) |
| **Categories** | history, completion, prompt, navigation, globbing, safety, performance, interaction, plugin:autosuggestions, plugin:syntax-highlighting, plugin:history-substring-search, plugin:fzf-tab, plugin:powerlevel10k, plugin:zoxide, plugin:fzf |
| **Config paths** | `~/.config/zsh`, `~/.zshrc`, `~/.zshenv`, `~/.zprofile`, `~/.zsh` |
| **Reload** | Shell restart required |
| **Config format** | shell (not validated) |
| **Plugins** | Supported (zinit, oh-my-zsh, zsh_unplugged detected) |
| **Homepage** | [zsh.org](https://www.zsh.org/) |

### Git

Distributed version control system.

| Property | Value |
|----------|-------|
| **Options** | 31 |
| **Categories** | user, core, diff, merge, push, color, aliases, safety |
| **Config paths** | `~/.config/git/config`, `~/.gitconfig` |
| **Reload** | Not applicable (reads config on each invocation) |
| **Config format** | git INI (validated by `doctor`) |
| **Plugins** | Not supported |
| **Homepage** | [git-scm.com](https://git-scm.com) |

### kitty

GPU-accelerated terminal emulator with advanced features.

| Property | Value |
|----------|-------|
| **Options** | 31 |
| **Categories** | appearance, fonts, cursor, scrollback, mouse, performance, tabs, keybindings |
| **Config paths** | `~/.config/kitty/kitty.conf`, `~/.config/kitty` |
| **Reload** | Auto-reloads on config change / SIGUSR1 |
| **Config format** | key-value (validated by `doctor`) |
| **Plugins** | Not supported |
| **Homepage** | [sw.kovidgoyal.net/kitty](https://sw.kovidgoyal.net/kitty/) |

### Neovim

Hyperextensible Vim-based text editor.

| Property | Value |
|----------|-------|
| **Options** | 31 |
| **Categories** | ui, editing, search, indentation, completion, lsp, performance, files |
| **Config paths** | `~/.config/nvim`, `~/.config/nvim/init.lua`, `~/.config/nvim/init.vim` |
| **Reload** | Manual (`:source` or restart) |
| **Config format** | lua (not validated) |
| **Plugins** | Managed by lazy.nvim (detected, not managed by dotsmith) |
| **Homepage** | [neovim.io](https://neovim.io) |

### Alacritty

GPU-accelerated terminal emulator focused on simplicity and performance.

| Property | Value |
|----------|-------|
| **Options** | 31 |
| **Categories** | window, font, colors, cursor, scrolling, shell, keybindings, hints |
| **Config paths** | `~/.config/alacritty/alacritty.toml`, `~/.config/alacritty/alacritty.yml`, `~/.config/alacritty` |
| **Reload** | Auto-reloads on config change |
| **Config format** | TOML (validated by `doctor`) |
| **Plugins** | Not supported |
| **Homepage** | [alacritty.org](https://alacritty.org) |

### awesome

Highly configurable tiling window manager for X11.

| Property | Value |
|----------|-------|
| **Options** | 31 |
| **Categories** | general, tags, layouts, keybindings, rules, wibar, themes, notifications |
| **Config paths** | `~/.config/awesome/rc.lua`, `~/.config/awesome` |
| **Reload** | `awesome-client 'awesome.restart()'` |
| **Config format** | lua (not validated) |
| **Plugins** | Not supported |
| **Homepage** | [awesomewm.org](https://awesomewm.org) |

## Tier 2 Auto-Detection

Any tool not in the Tier 1 list is auto-detected as Tier 2 when you run `dotsmith add <tool>`.

### Path Detection

dotsmith checks these locations (in order, first match wins):

1. `~/.config/<tool>/`
2. `~/.<tool>rc`
3. `~/.<tool>config`
4. `~/.<tool>/`
5. `~/.<tool>`

For example, `dotsmith add ranger` would find `~/.config/ranger/` and track it.

### What Works

- `snapshot`, `history`, `diff`, `rollback`
- `deploy`, `deploy-remote`
- `edit`, `watch`
- `status`, `list`
- `profile save/load`

### What Doesn't Work

- `explore` (no option database)
- `search` (no options to search)
- Config generation (`g` key in TUI)
- Syntax validation in `doctor`
- Plugin management

## Config Validation

The `doctor` command validates config syntax for tools with supported formats:

| Format | Tools | Validator |
|--------|-------|-----------|
| TOML | alacritty | `toml` crate parser |
| Key-value | kitty | Line-by-line `key = value` check |
| Git INI | git | INI section/key parser |
| tmux | tmux | Basic syntax check |
| Shell | zsh | Skipped (too complex) |
| Lua | neovim, awesomewm | Skipped (too complex) |

## See Also

- [Getting Started](getting-started.md) -- adding tools
- [Plugin Management](plugins.md) -- managing plugins for zsh and tmux
- [Contributing](contributing.md) -- adding a new Tier 1 module
