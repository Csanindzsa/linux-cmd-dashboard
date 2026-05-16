# Development Log

## 2026-05-16

- Scaffolded the Rust project.
- Added GTK4/libadwaita/VTE dependencies behind the default `gui` feature.
- Implemented the pure layout workspace with pane IDs, recursive splits, focus
  navigation, move, close, and resize operations.
- Added TOML settings defaults, including `fish` as the default shell.
- Added GTK app shell with a header bar, pane action buttons, keyboard actions,
  VTE terminal spawning, fullscreen mode, and pane overview.
- Added README build instructions and shortcut reference.
- Verified core tests with `cargo test --no-default-features --lib`.

## Known Local Environment Gap

Full GUI checks are blocked until a package providing `vte-2.91-gtk4.pc` is
installed. On Arch Linux this is typically `vte4`; on Debian/Ubuntu package
names vary by release.
