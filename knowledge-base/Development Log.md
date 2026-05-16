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
- Committed the initial scaffold and Obsidian vault as `a3166e6`.

## Next Development Step

- Persist user-adjusted split ratios from GTK paned drag handles back into the
  layout tree.
- Add a restart action for the focused pane so exited shells can be relaunched
  without closing and recreating the pane.

## Continued After Initial Commit

- Added layout support for updating nested split ratios by path.
- Wired GTK paned position changes back into the workspace layout.
- Added focused-pane restart action and `Ctrl+Shift+R` shortcut.
- Added a unit test for nested split-ratio updates.
- Made settings deserialization tolerate missing fields.
- Added configurable restart shortcut and first-launch config file creation.

## After VTE Installation

- Confirmed `vte-2.91-gtk4` is available through pkg-config at version `0.84.0`.
- Confirmed the full GUI build path reaches Rust compilation successfully.
- Cleaned up full-build warnings from deprecated CSS loading, unused mutability,
  and unread pane metadata fields.

## Runtime Debugging

- Fixed a startup panic caused by holding a `RefCell` borrow while GTK emitted a
  focus-enter signal from `grab_focus()`.
- Made terminal signal handlers avoid panicking on re-entrant GTK callbacks by
  using `try_borrow_mut()`.
- Set the app's dark preference through `AdwStyleManager`. A startup warning can
  still appear if the desktop/session sets GTK's unsupported
  `gtk-application-prefer-dark-theme` setting globally.

## Known Local Environment Gap

Full GUI checks are blocked until a package providing `vte-2.91-gtk4.pc` is
installed. On Arch Linux this is typically `vte4`; on Debian/Ubuntu package
names vary by release.
