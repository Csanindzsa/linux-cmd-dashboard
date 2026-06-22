# Architecture

## App Shape

The application is a Rust desktop app using GTK4, libadwaita, and VTE.

The implementation is split into:

- `src/layout.rs`: pure tiling layout model and pane navigation logic.
- `src/config.rs`: TOML-backed settings model.
- `src/app.rs`: GTK/libadwaita/VTE UI and terminal lifecycle.

## Layout Model

The layout tree is recursive:

- `Leaf(PaneId)` represents one terminal pane.
- `Split` stores orientation, split ratio, and two child layout nodes.

The layout model does not depend on GTK. It can be tested with normal Rust unit
tests and is responsible for split, close, focus, move, resize, and neighbor
lookup.

## Terminal Runtime

Each pane owns a VTE terminal widget. New panes spawn the configured shell,
defaulting to `fish`, through VTE's pseudo-terminal support. When possible, new
panes inherit the focused terminal's current directory.

## Theme Runtime

The default terminal theme source is Alacritty. The app reads
`~/.config/alacritty/alacritty.toml` when present and imports primary
foreground/background colors, a cyan accent, opacity, and dark/light preference.
VTE panes use clear backgrounds plus RGBA terminal colors for compositor-backed
transparency.

The app can also use a coarse system-theme palette or explicit custom colors
from `~/.config/linux-cmd-dashboard/config.toml`.

## Branding

The project icon lives under `assets/icons/hicolor` in scalable SVG and 256px
PNG forms. During development, the app adds `assets/icons` to GTK's icon search
path and uses `dev.codex.LinuxCmdDashboard` as its icon name.

## Linux Install Flow

The preferred user install path is `scripts/install-linux.sh`. By default it
downloads the latest GitHub AppImage release, verifies the published checksum,
extracts the AppImage payload, and installs the extracted bundle under
`~/.local/opt/linux-cmd-dashboard`. The wrapper at
`~/.local/bin/linux-cmd-dashboard` sets `LD_LIBRARY_PATH` and `XDG_DATA_DIRS`
for the bundled libraries and assets, so the install works even on systems
without AppImage FUSE support.

The same script can build and install a native source binary with
`--from-source` when GTK4, libadwaita, VTE GTK4, pkg-config, fish, and Cargo are
available.

Terminal child processes receive an explicit sanitized environment. This strips
the extracted AppImage `LD_LIBRARY_PATH`, `XDG_DATA_DIRS`, and wrapper-only
variables before spawning the configured shell, so commands inside panes use the
host system libraries instead of the bundled GTK/VTE runtime.
