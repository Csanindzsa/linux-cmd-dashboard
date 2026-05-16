# Testing

## Current Automated Checks

```sh
cargo fmt --all
cargo test --no-default-features --lib
cargo check --no-default-features
```

## Current Result

- Core layout/config tests pass.
- GUI build is blocked in the current environment because `pkg-config` cannot
  find `vte-2.91-gtk4`.

## Manual Acceptance Targets

- Open 20 or more panes in one window.
- Run separate `codex` sessions manually in multiple panes.
- Navigate, move, resize, fullscreen, and close panes with shortcuts.
- Verify copy/paste, scrollback, ANSI colors, and interactive shell behavior.
