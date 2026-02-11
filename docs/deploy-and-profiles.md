# Deploy & Profiles

dotsmith provides several ways to move configs between locations and machines: local symlink deployment, remote deployment via SSH, named configuration profiles, and git-based repo sync.

## Local Deploy

Create symlinks from a source directory (where your managed configs live) to a target location (where the tool expects them).

```sh
dotsmith deploy ~/dots/tmux ~/.config/tmux --dry-run   # preview
dotsmith deploy ~/dots/tmux ~/.config/tmux              # apply
```

### What Happens

For each source file or directory, dotsmith classifies the target:

| Situation | Action |
|-----------|--------|
| Nothing at target | Create symlink |
| Symlink already points to source | No action (already correct) |
| Symlink points elsewhere | Remove old symlink, create new one |
| Regular file/directory at target | Back up to `backups/`, then create symlink |
| Source doesn't exist | Skip with warning |

Backups are stored at `~/.config/dotsmith/backups/` as `<name>.<timestamp>.bak`.

## Remote Deploy

Deploy tracked configs to a remote host via SSH and SCP.

```sh
dotsmith deploy-remote myserver                              # deploy all
dotsmith deploy-remote myserver --dry-run                    # preview
dotsmith deploy-remote myserver --tool tmux --tool zsh       # specific tools
dotsmith deploy-remote myserver --user alice                 # specify SSH user
```

### How It Works

1. For each tracked tool (or `--tool` filtered set), collects all config files
2. Checks which files exist on the remote (via `ssh test -e`)
3. Displays the plan: new files in green, overwrites in yellow
4. With `--dry-run`: stops here
5. Otherwise:
   - Creates remote parent directories (`mkdir -p`)
   - Backs up existing remote files as `<path>.dotsmith-bak.<timestamp>`
   - Copies files via `scp`

### SSH Configuration

dotsmith uses your system `ssh` and `scp` commands with `BatchMode=yes` (non-interactive, fails fast if no key auth). Your `~/.ssh/config` is fully respected:

- Host aliases
- ProxyJump / bastion hosts
- Agent forwarding
- Custom ports and users

## Configuration Profiles

Save and restore named configuration snapshots -- useful for switching between setups or migrating to a new machine.

### Save

```sh
dotsmith profile save workstation
```

Saves all tracked tools and their config file contents to `~/.config/dotsmith/profiles/workstation/`. Each file is checksummed with SHA-256 for integrity verification.

**Profile names** must be 1-64 characters, alphanumeric plus `-` and `_`. No spaces or dots.

### List

```sh
dotsmith profile list
```

Shows saved profiles with creation date, tool count, and file count.

### Load

```sh
dotsmith profile load workstation --dry-run        # preview first
dotsmith profile load workstation                  # restore configs
dotsmith profile load workstation --add-untracked  # also add new tools
```

**What happens on load:**
- For each tool in the profile:
  - If currently tracked: restore config files (backs up existing files first)
  - If not tracked and `--add-untracked`: add the tool and restore its configs
  - If not tracked without the flag: skip
- Backs up any existing files to `~/.config/dotsmith/backups/`

### Delete

```sh
dotsmith profile delete old-setup
```

Removes the profile directory.

### Use Cases

- **Machine migration**: `profile save` on old machine, copy `profiles/` directory, `profile load` on new machine
- **Setup switching**: save `workstation` and `laptop` profiles, switch with `profile load`
- **Experimentation**: save current state, make changes, load the saved profile to revert

## Repo Sync

Keep a git-backed copy of your configs for version control and remote backup.

### Initialize

```sh
dotsmith repo init ~/dots
```

Creates a git repo at the specified path and saves it in `config.toml`.

### Sync

```sh
dotsmith repo sync
```

Copies all tracked config files into the repo directory (organized by tool name), stages everything, and commits if there are changes. The commit message includes the tool and file count.

### Status

```sh
dotsmith repo status
```

Shows whether the repo has uncommitted changes.

### Pushing to a Remote

dotsmith manages the local repo only. To push:

```sh
cd ~/dots
git remote add origin <url>
git push -u origin main
```

### TUI

Press `g` on the dashboard to trigger a repo sync.

## Workflow: New Machine Setup

1. On your current machine:
   ```sh
   dotsmith profile save mysetup
   dotsmith repo sync && cd ~/dots && git push
   ```

2. On the new machine:
   ```sh
   # Install dotsmith, then:
   dotsmith init
   git clone <repo-url> ~/dots
   # Copy the profiles directory from old machine (via scp, USB, etc.)
   dotsmith profile load mysetup --add-untracked
   ```

Or use remote deploy for a quick push:
```sh
dotsmith deploy-remote newmachine --user me
```

## See Also

- [Command Reference](commands.md#deployment) -- full deploy/profile/repo commands
- [Configuration](configuration.md) -- config.toml settings including repo_path
- [Getting Started](getting-started.md) -- initial setup
