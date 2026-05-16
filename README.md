# linux-cmd-dashboard

A native Linux tiling terminal manager for running many manual shell sessions in one window.

The app uses GTK4/libadwaita and VTE terminal widgets. Each pane starts a normal
`fish` shell, so tools such as `codex` are launched manually inside whichever
pane should own that session.

## Features

- Single native window with recursive tiled terminal panes
- VTE-backed terminal behavior: ANSI colors, scrollback, selection, paste, and
  interactive programs
- Default shell: `fish`
- Pane operations:
  - split right/down
  - close focused pane
  - focus neighbors
  - move focused pane
  - resize focused split
  - fullscreen focused pane
  - overview dialog
- Per-pane metadata for title, cwd, process status, and accent color
- TOML settings at `~/.config/linux-cmd-dashboard/config.toml`

## Shortcuts

| Action | Shortcut |
| --- | --- |
| New pane | `Ctrl+Shift+Enter` |
| Close pane | `Ctrl+Shift+W` |
| Restart pane | `Ctrl+Shift+R` |
| Focus left/down/up/right | `Ctrl+Shift+H/J/K/L` |
| Move left/down/up/right | `Ctrl+Shift+Alt+H/J/K/L` |
| Fullscreen focused pane | `Ctrl+Shift+F` |
| Overview | `Ctrl+Shift+O` |
| Resize split | `Ctrl+Shift+-` / `Ctrl+Shift+=` |

## Build

Install the Rust toolchain plus GTK4, libadwaita, VTE GTK4, pkg-config, and
`fish`. The VTE package must provide `vte-2.91-gtk4.pc`.

Examples:

```sh
# Arch Linux
sudo pacman -S rust gtk4 libadwaita vte4 fish pkgconf

# Debian/Ubuntu package names vary by release
sudo apt install cargo libgtk-4-dev libadwaita-1-dev libvte-2.91-gtk4-dev fish pkg-config
```

Then run:

```sh
cargo run
```

Core layout and config tests can be run without GTK/VTE development files:

```sh
cargo test --no-default-features --lib
```
