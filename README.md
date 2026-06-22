# linux-cmd-dashboard

A native Linux tiling terminal manager for running many manual shell sessions in
one window.

The app uses Rust, GTK4/libadwaita, and VTE terminal widgets. Each pane starts a
normal shell, defaulting to `fish`, so tools such as `codex` are launched
manually inside whichever pane should own that session.

## Project Status

This is an early hobby project. It is usable for local terminal workflows, but
it has only been developed against a modern Linux desktop with GTK4,
libadwaita, VTE GTK4, and a compositor that can blend transparent terminal
backgrounds.

## Features

- Single native window with recursive tiled terminal panes.
- VTE-backed terminal behavior: ANSI colors, scrollback, selection, paste, and
  interactive programs.
- Configurable shell, font, scrollback, pane cwd inheritance, theme, and
  keybindings.
- Pane operations for splitting, closing, restarting, focusing, moving,
  resizing, fullscreening, and viewing an overview dialog.
- Per-pane metadata for title, cwd, process status, and accent color.
- TOML settings at `~/.config/linux-cmd-dashboard/config.toml`.
- Missing settings fields fall back to built-in defaults, so new versions can
  add keys without breaking older config files.
- Project logo and app icon under `assets/icons/hicolor`.
- Terminal colors can follow Alacritty, Kitty, the system theme, or custom
  settings.
- Semi-transparent terminal backgrounds are supported when the compositor allows
  alpha blending.
- Image-only clipboard paste saves the image to a cache file and pastes the
  quoted file path into the focused terminal.

## Install

The easiest install path is the user-local installer. It downloads the latest
GitHub AppImage release, verifies `SHA256SUMS`, extracts the bundled app so FUSE
is not required at runtime, and installs:

- `~/.local/bin/linux-cmd-dashboard`
- `~/.local/opt/linux-cmd-dashboard`
- `~/.local/share/applications/linux-cmd-dashboard.desktop`
- icons under `~/.local/share/icons/hicolor`

From a checkout:

```sh
./scripts/install-linux.sh
```

Or from the latest release:

```sh
curl -L -o install-linux.sh \
  https://github.com/Csanindzsa/linux-cmd-dashboard/releases/latest/download/install-linux.sh
chmod +x install-linux.sh
./install-linux.sh
```

Launch from the desktop menu as **Linux Command Dashboard**, or run:

```sh
~/.local/bin/linux-cmd-dashboard
```

To install a specific release:

```sh
./scripts/install-linux.sh --version 0.1.2
```

To remove the user-local install:

```sh
./scripts/install-linux.sh --uninstall
```

## Install From Source

Source installs are useful for development. Install the Rust toolchain plus
GTK4, libadwaita, VTE GTK4, pkg-config, and `fish`. The VTE package must provide
`vte-2.91-gtk4.pc`.

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

To build and install a native local binary:

```sh
./scripts/install-linux.sh --from-source
```

Manual install commands are still available if you want to control every file:

```sh
cargo build --release
install -Dm755 target/release/linux-cmd-dashboard ~/.local/bin/linux-cmd-dashboard
install -Dm644 linux-cmd-dashboard.desktop ~/.local/share/applications/linux-cmd-dashboard.desktop
install -Dm644 assets/icons/hicolor/scalable/apps/dev.codex.LinuxCmdDashboard.svg \
  ~/.local/share/icons/hicolor/scalable/apps/dev.codex.LinuxCmdDashboard.svg
gtk-update-icon-cache ~/.local/share/icons/hicolor
```

## Releases

GitHub releases provide prebuilt Linux x86_64 downloads:

- `install-linux.sh` for a user-local install that does not require root.
- `.AppImage` for a portable, no-install launch path.
- `.deb` for Debian/Ubuntu-style systems.
- `.tar.gz` for manual installs on other distributions.
- `SHA256SUMS` for verifying downloads.

The release workflow is defined in `.github/workflows/release.yml`, and the
packaging script lives at `scripts/package-linux.sh`.
Release notes are tracked in [CHANGELOG.md](CHANGELOG.md).

## Configuration

The app creates `~/.config/linux-cmd-dashboard/config.toml` on first launch.
Important defaults:

```toml
shell = "fish"
inherit_focused_cwd = true
font = "Monospace 11"
scrollback_lines = 20000

[theme]
source = "alacritty"
transparent_background = true
background_opacity = 0.7
```

By default, terminals import `~/.config/alacritty/alacritty.toml`, including
`colors.primary`, cyan accent colors, `window.opacity`, and dark/light window
preference.

To use a Kitty theme, set `source = "kitty"` and point `kitty_config` to a theme
source. The value can be a full path, a file under
`~/.config/kitty/themes/<name>.conf`, or a Kitty built-in theme name supported by
`kitty +kitten themes --dump-theme <name>`.

```toml
[theme]
source = "kitty"
kitty_config = "Dark+"
```

To use custom colors instead:

```toml
[theme]
source = "custom"
foreground = "#d8dee9"
background = "#111318"
titlebar_background = "#191a21"
cursor = "#f2f4f8"
accent = "#4cc9f0"
transparent_background = true
background_opacity = 0.82
```

## Shortcuts

| Action | Default Shortcut |
| --- | --- |
| New pane | `Ctrl+Shift+Enter` |
| Close pane | `Ctrl+Shift+W` |
| Restart pane | `Ctrl+Shift+R` |
| Focus left/down/up/right | `Ctrl+Shift+H/J/K/L` |
| Move left/down/up/right | `Ctrl+Shift+Alt+H/J/K/L` |
| Fullscreen focused pane | `Ctrl+Shift+F` |
| Overview | `Ctrl+Shift+O` |
| Resize split | `Ctrl+Shift+-` / `Ctrl+Shift+=` |

Keybindings are configurable in `config.toml` under `[keybindings]`. Resize
bindings are currently fixed in code.

## Development

Useful checks before sending changes:

```sh
cargo fmt --all
bash -n scripts/install-linux.sh
cargo test --no-default-features --lib
cargo check --no-default-features
cargo check
```

Core layout and config tests can be run without GTK/VTE development files:

```sh
cargo test --no-default-features --lib
```

Project notes live in `knowledge-base/`.

## Contributing

Forks, patches, and hobby/non-commercial changes are welcome. Before opening a
pull request, run the checks listed above and include a short note about any
manual GTK/VTE testing you did.

See [CONTRIBUTING.md](CONTRIBUTING.md) for the contribution workflow and
[SECURITY.md](SECURITY.md) for security reporting.

## License

This project is licensed under the PolyForm Noncommercial License 1.0.0. It
allows non-commercial use, forks, redistribution, and changes, including hobby
projects and personal experimentation. Commercial use is not permitted under
this license.

This is source-available software, not OSI-approved open source. See
[LICENSE.md](LICENSE.md) for the full terms.
