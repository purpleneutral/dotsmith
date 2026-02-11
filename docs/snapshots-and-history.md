# Snapshots & History

Snapshots are dotsmith's safety net. Before you change a config, snapshot it. If something breaks, roll back.

## Taking Snapshots

### Manual

```sh
dotsmith snapshot                        # snapshot all tracked tools
dotsmith snapshot tmux                   # snapshot a specific tool
dotsmith snapshot tmux -m "before mouse" # attach a message
```

### Automatic

- **`edit` command** -- automatically snapshots before opening your editor
- **`watch` command** -- automatically snapshots when a config file changes on disk

### TUI

Press `s` on the dashboard to snapshot all tracked tools at once.

## How Storage Works

Snapshots are stored in a SQLite database at `~/.config/dotsmith/snapshots.db` using WAL mode for performance.

Each snapshot records:
- Tool name
- File path (tilde-contracted for portability)
- Full file contents
- SHA-256 content hash
- Optional message
- Timestamp

**Deduplication**: If a file hasn't changed since the last snapshot, no new entry is created. The unique constraint on `(tool, file_path, hash)` prevents duplicate content from being stored.

## Viewing History

```sh
dotsmith history tmux
dotsmith history tmux --limit 5
```

Output shows snapshot IDs, timestamps, content hashes, file paths, and messages:

```
Snapshot history for tmux (showing last 20):

  ID    Date                      Hash      Path                            Message
  5     2025-02-10 15:22:00 UTC   a3f4d5e2  ~/.config/tmux/tmux.conf        enabled mouse
  3     2025-02-10 14:32:15 UTC   b7e8c1f9  ~/.config/tmux/tmux.conf        initial setup
```

### TUI

Press `h` on the dashboard to open the history view for the selected tool. Navigate with `j`/`k`, press `Enter` to view a snapshot's diff, or `r` to rollback directly.

## Viewing Diffs

```sh
dotsmith diff          # diff all tracked tools
dotsmith diff tmux     # diff a specific tool
```

Shows a colored unified diff (like `git diff`) between the current file contents and the last snapshot. No output means no changes.

### TUI

Press `d` on the dashboard to open the diff view. Scroll with `j`/`k`, page with `d`/`u`, jump with `g`/`G`.

## Rolling Back

```sh
dotsmith rollback 5 --dry-run   # preview changes
dotsmith rollback 5             # restore snapshot #5
```

The snapshot ID comes from `history` output.

**Before any rollback:**
1. The current file is backed up to `~/.config/dotsmith/backups/` as `<filename>.<timestamp>.bak`
2. The snapshot content is written to the original file path atomically

Always use `--dry-run` first to see what will change.

### TUI

In the history view, select a snapshot and press `r` to rollback.

## File Watching

```sh
dotsmith watch          # watch all tracked tools
dotsmith watch tmux     # watch a specific tool
```

The watch command monitors tracked config files and auto-snapshots when changes are detected.

**How it works:**
- Polls every 2 seconds (checks file modification time)
- On mtime change, computes SHA-256 hash to verify actual content change
- Only snapshots when content actually differs (ignores touch-only changes)
- Prints timestamps for each detected change and snapshot

```
Watching 3 file(s) for 2 tool(s)... (Ctrl-C to stop)
  ~/.config/tmux/tmux.conf [tmux]
  ~/.config/zsh/.zshrc [zsh]

[14:32:15] tmux ~/.config/tmux/tmux.conf changed
[14:32:16] OK snapshotted 1 file(s)
```

Press `Ctrl-C` to stop watching.

## Typical Workflow

1. **Snapshot** before making changes:
   ```sh
   dotsmith snapshot tmux -m "working config"
   ```

2. **Edit** the config (or use `dotsmith edit tmux` for auto-snapshot):
   ```sh
   dotsmith edit tmux
   ```

3. **Check** what changed:
   ```sh
   dotsmith diff tmux
   ```

4. **Happy?** Take another snapshot to save the new state.

5. **Not happy?** Roll back:
   ```sh
   dotsmith history tmux          # find the snapshot ID
   dotsmith rollback 5 --dry-run  # preview
   dotsmith rollback 5            # restore
   ```

## See Also

- [Command Reference](commands.md#snapshots--history) -- full command details
- [TUI Guide](tui.md#diff-view) -- diff and history views
