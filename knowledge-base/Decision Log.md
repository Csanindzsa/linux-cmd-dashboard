# Decision Log

## Use VTE Instead Of Embedding Alacritty

Alacritty is not designed to be embedded as many panes inside one GUI process.
VTE provides a native GTK terminal widget, so it is the practical fit for a
single-window tiled terminal app.

## Keep Layout Core Independent

The tiling model is independent from GTK and VTE so pane operations can be unit
tested without a display server or terminal system libraries.

## Make GUI Dependencies Optional For Core Tests

The default build includes the GUI. The core tests can run with
`--no-default-features` because this environment currently lacks the
`vte-2.91-gtk4` pkg-config package needed by the VTE Rust crate.
