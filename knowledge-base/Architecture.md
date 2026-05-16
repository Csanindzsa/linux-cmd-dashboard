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
