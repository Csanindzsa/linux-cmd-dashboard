# Testing

## Current Automated Checks

```sh
cargo fmt --all
bash -n scripts/install-linux.sh
cargo test --no-default-features --lib
cargo check --no-default-features
cargo check
```

## Current Result

- Core layout/config tests pass.
- Core tests now cover nested split-ratio persistence.
- Config tests cover fallback defaults for missing settings fields.
- Config tests cover parsing Alacritty TOML colors and opacity.
- Runtime environment tests cover stripping extracted AppImage variables from
  spawned terminal shells.
- Installer syntax is checked with `bash -n scripts/install-linux.sh`.
- Full GUI `cargo check` works after installing `vte4`.

## Manual Acceptance Targets

- Open 20 or more panes in one window.
- Run separate `codex` sessions manually in multiple panes.
- Navigate, move, resize, fullscreen, and close panes with shortcuts.
- Verify copy/paste, scrollback, ANSI colors, and interactive shell behavior.
