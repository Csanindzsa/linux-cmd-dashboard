# Testing

## Current Automated Checks

```sh
cargo fmt --all
cargo test --no-default-features --lib
cargo check --no-default-features
cargo check
```

## Current Result

- Core layout/config tests pass.
- Core tests now cover nested split-ratio persistence.
- Config tests cover fallback defaults for missing settings fields.
- Full GUI `cargo check` works after installing `vte4`.

## Manual Acceptance Targets

- Open 20 or more panes in one window.
- Run separate `codex` sessions manually in multiple panes.
- Navigate, move, resize, fullscreen, and close panes with shortcuts.
- Verify copy/paste, scrollback, ANSI colors, and interactive shell behavior.
