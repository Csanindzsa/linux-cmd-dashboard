use crate::{
    config::{EffectiveTheme, TerminalConfig},
    layout::{Direction, LayoutTree, PaneId, SplitOrientation, Workspace},
    runtime_env,
};
use adw::prelude::*;
use gtk::{gdk, gio, glib};
use std::{
    cell::RefCell,
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
    rc::Rc,
    time::{SystemTime, UNIX_EPOCH},
};
use vte::prelude::*;

const APP_ID: &str = "dev.codex.LinuxCmdDashboard";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PaneStatus {
    Running,
    Exited(i32),
}

#[derive(Clone)]
struct Pane {
    id: PaneId,
    title: String,
    cwd: PathBuf,
    status: PaneStatus,
    terminal: vte::Terminal,
    accent: String,
}

struct UiState {
    workspace: Workspace,
    panes: HashMap<PaneId, Pane>,
    config: TerminalConfig,
    theme: EffectiveTheme,
    content: gtk::Box,
    window: adw::ApplicationWindow,
    fullscreen: bool,
}

pub fn run() {
    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_startup(|_| {
        adw::init().expect("libadwaita initialization failed");
    });
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &adw::Application) {
    let config = TerminalConfig::load_or_create();
    let style_manager = adw::StyleManager::default();
    let theme = config.effective_theme(style_manager.is_dark());
    configure_style_manager(&theme);
    install_icon_theme();
    install_css(&theme);

    let window = adw::ApplicationWindow::builder()
        .application(app)
        .title("Terminal Tiles")
        .icon_name(APP_ID)
        .default_width(1320)
        .default_height(860)
        .build();

    let toolbar = adw::ToolbarView::new();
    toolbar.add_css_class("terminal-toolbar");
    let header = adw::HeaderBar::new();
    header.add_css_class("terminal-titlebar");
    let title_box = gtk::Box::new(gtk::Orientation::Horizontal, 8);
    let logo = gtk::Image::from_icon_name(APP_ID);
    logo.set_pixel_size(22);
    title_box.append(&logo);
    title_box.append(&gtk::Label::new(Some("Terminal Tiles")));
    header.set_title_widget(Some(&title_box));

    let split_h = gtk::Button::from_icon_name("view-dual-symbolic");
    split_h.set_tooltip_text(Some("Split right"));
    let split_v = gtk::Button::from_icon_name("view-grid-symbolic");
    split_v.set_tooltip_text(Some("Split down"));
    let close = gtk::Button::from_icon_name("window-close-symbolic");
    close.set_tooltip_text(Some("Close pane"));
    let restart = gtk::Button::from_icon_name("view-refresh-symbolic");
    restart.set_tooltip_text(Some("Restart pane"));
    let overview = gtk::Button::from_icon_name("view-list-symbolic");
    overview.set_tooltip_text(Some("Overview"));

    header.pack_start(&split_h);
    header.pack_start(&split_v);
    header.pack_end(&overview);
    header.pack_end(&close);
    header.pack_end(&restart);
    toolbar.add_top_bar(&header);

    let content = gtk::Box::new(gtk::Orientation::Vertical, 0);
    content.add_css_class("terminal-content");
    toolbar.set_content(Some(&content));
    window.set_content(Some(&toolbar));

    let workspace = Workspace::new();
    let first_id = workspace.focused();
    let first_pane = create_pane(first_id, inherited_cwd(None), &config, &theme);

    let state = Rc::new(RefCell::new(UiState {
        workspace,
        panes: HashMap::from([(first_id, first_pane)]),
        config,
        theme,
        content,
        window: window.clone(),
        fullscreen: false,
    }));

    connect_pane_signals(&state, first_id);
    install_window_clipboard_shortcuts(&window, &state);
    install_actions(app, &state);

    {
        let state = state.clone();
        split_h.connect_clicked(move |_| split_current(&state, Direction::Right));
    }
    {
        let state = state.clone();
        split_v.connect_clicked(move |_| split_current(&state, Direction::Down));
    }
    {
        let state = state.clone();
        close.connect_clicked(move |_| close_current(&state));
    }
    {
        let state = state.clone();
        restart.connect_clicked(move |_| restart_current(&state));
    }
    {
        let state = state.clone();
        overview.connect_clicked(move |_| show_overview(&state));
    }

    render(&state);
    window.present();
}

fn install_actions(app: &adw::Application, state: &Rc<RefCell<UiState>>) {
    add_action(app, "new-pane", state, |state| {
        split_current(state, Direction::Right)
    });
    add_action(app, "close-pane", state, close_current);
    add_action(app, "restart-pane", state, restart_current);
    add_action(app, "focus-left", state, |state| {
        focus_neighbor(state, Direction::Left)
    });
    add_action(app, "focus-down", state, |state| {
        focus_neighbor(state, Direction::Down)
    });
    add_action(app, "focus-up", state, |state| {
        focus_neighbor(state, Direction::Up)
    });
    add_action(app, "focus-right", state, |state| {
        focus_neighbor(state, Direction::Right)
    });
    add_action(app, "move-left", state, |state| {
        move_current(state, Direction::Left)
    });
    add_action(app, "move-down", state, |state| {
        move_current(state, Direction::Down)
    });
    add_action(app, "move-up", state, |state| {
        move_current(state, Direction::Up)
    });
    add_action(app, "move-right", state, |state| {
        move_current(state, Direction::Right)
    });
    add_action(app, "fullscreen", state, toggle_fullscreen);
    add_action(app, "overview", state, show_overview);
    add_action(app, "resize-left", state, |state| {
        resize_current(state, Direction::Left)
    });
    add_action(app, "resize-down", state, |state| {
        resize_current(state, Direction::Down)
    });
    add_action(app, "resize-up", state, |state| {
        resize_current(state, Direction::Up)
    });
    add_action(app, "resize-right", state, |state| {
        resize_current(state, Direction::Right)
    });

    let config = &state.borrow().config.keybindings;
    app.set_accels_for_action("app.new-pane", &[&config.new_pane]);
    app.set_accels_for_action("app.close-pane", &[&config.close_pane]);
    app.set_accels_for_action("app.restart-pane", &[&config.restart_pane]);
    app.set_accels_for_action("app.focus-left", &[&config.focus_left]);
    app.set_accels_for_action("app.focus-down", &[&config.focus_down]);
    app.set_accels_for_action("app.focus-up", &[&config.focus_up]);
    app.set_accels_for_action("app.focus-right", &[&config.focus_right]);
    app.set_accels_for_action("app.move-left", &[&config.move_left]);
    app.set_accels_for_action("app.move-down", &[&config.move_down]);
    app.set_accels_for_action("app.move-up", &[&config.move_up]);
    app.set_accels_for_action("app.move-right", &[&config.move_right]);
    app.set_accels_for_action("app.fullscreen", &[&config.fullscreen]);
    app.set_accels_for_action("app.overview", &[&config.overview]);
    app.set_accels_for_action("app.resize-left", &["<Ctrl><Shift>minus"]);
    app.set_accels_for_action("app.resize-right", &["<Ctrl><Shift>equal"]);
}

fn add_action(
    app: &adw::Application,
    name: &str,
    state: &Rc<RefCell<UiState>>,
    f: impl Fn(&Rc<RefCell<UiState>>) + 'static,
) {
    let action = gio::SimpleAction::new(name, None);
    let state = state.clone();
    action.connect_activate(move |_, _| f(&state));
    app.add_action(&action);
}

fn split_current(state: &Rc<RefCell<UiState>>, direction: Direction) {
    let (id, cwd, config, theme) = {
        let mut state = state.borrow_mut();
        let old_focus = state.workspace.focused();
        let cwd = if state.config.inherit_focused_cwd {
            state
                .panes
                .get(&old_focus)
                .and_then(current_terminal_cwd)
                .unwrap_or_else(|| inherited_cwd(state.panes.get(&old_focus).map(|pane| &pane.cwd)))
        } else {
            inherited_cwd(None)
        };
        let id = state.workspace.split_focused_toward(direction);
        (id, cwd, state.config.clone(), state.theme.clone())
    };

    let pane = create_pane(id, cwd, &config, &theme);
    state.borrow_mut().panes.insert(id, pane);
    connect_pane_signals(state, id);
    render(state);
}

fn close_current(state: &Rc<RefCell<UiState>>) {
    let closed = state.borrow_mut().workspace.close_focused();
    if let Some(id) = closed {
        state.borrow_mut().panes.remove(&id);
        render(state);
    }
}

fn restart_current(state: &Rc<RefCell<UiState>>) {
    let (id, cwd, config, theme) = {
        let state_ref = state.borrow();
        let id = state_ref.workspace.focused();
        let Some(pane) = state_ref.panes.get(&id) else {
            return;
        };
        let cwd = current_terminal_cwd(pane).unwrap_or_else(|| pane.cwd.clone());
        (id, cwd, state_ref.config.clone(), state_ref.theme.clone())
    };

    let pane = create_pane(id, cwd, &config, &theme);
    state.borrow_mut().panes.insert(id, pane);
    connect_pane_signals(state, id);
    render(state);
}

fn focus_neighbor(state: &Rc<RefCell<UiState>>, direction: Direction) {
    if state
        .borrow_mut()
        .workspace
        .focus_neighbor(direction)
        .is_some()
    {
        render(state);
    }
}

fn move_current(state: &Rc<RefCell<UiState>>, direction: Direction) {
    if state
        .borrow_mut()
        .workspace
        .move_focused(direction)
        .is_some()
    {
        render(state);
    }
}

fn resize_current(state: &Rc<RefCell<UiState>>, direction: Direction) {
    if state.borrow_mut().workspace.resize_focused(direction, 0.05) {
        render(state);
    }
}

fn toggle_fullscreen(state: &Rc<RefCell<UiState>>) {
    {
        let mut state = state.borrow_mut();
        state.fullscreen = !state.fullscreen;
    }
    render(state);
}

fn show_overview(state: &Rc<RefCell<UiState>>) {
    let state_ref = state.borrow();
    let dialog = gtk::Window::builder()
        .title("Panes")
        .default_width(560)
        .default_height(420)
        .transient_for(&state_ref.window)
        .modal(true)
        .build();
    let list = gtk::ListBox::new();
    list.add_css_class("boxed-list");

    for id in state_ref.workspace.pane_ids() {
        let Some(pane) = state_ref.panes.get(&id) else {
            continue;
        };
        let row = adw::ActionRow::builder()
            .title(format!("{} - pane {}", pane.title, pane.id.0))
            .subtitle(format!(
                "{} - {}",
                status_label(pane.status),
                pane.cwd.display()
            ))
            .build();
        list.append(&row);
    }

    let scroller = gtk::ScrolledWindow::builder()
        .hscrollbar_policy(gtk::PolicyType::Never)
        .child(&list)
        .build();
    dialog.set_child(Some(&scroller));
    dialog.present();
}

fn render(state: &Rc<RefCell<UiState>>) {
    let (content, root, focused, fullscreen) = {
        let state_ref = state.borrow();
        while let Some(child) = state_ref.content.first_child() {
            state_ref.content.remove(&child);
        }
        (
            state_ref.content.clone(),
            state_ref.workspace.root().clone(),
            state_ref.workspace.focused(),
            state_ref.fullscreen,
        )
    };

    let widget = if fullscreen {
        render_leaf(state, focused)
    } else {
        render_node(state, &root, Vec::new())
    };

    content.append(&widget);
    widget.grab_focus();
    let focused_terminal = state
        .borrow()
        .panes
        .get(&focused)
        .map(|pane| pane.terminal.clone());
    if let Some(terminal) = focused_terminal {
        terminal.grab_focus();
    }
}

fn render_node(state: &Rc<RefCell<UiState>>, node: &LayoutTree, path: Vec<bool>) -> gtk::Widget {
    match node {
        LayoutTree::Leaf(id) => render_leaf(state, *id),
        LayoutTree::Split {
            orientation,
            ratio,
            first,
            second,
        } => {
            let paned = gtk::Paned::new(match orientation {
                SplitOrientation::Horizontal => gtk::Orientation::Horizontal,
                SplitOrientation::Vertical => gtk::Orientation::Vertical,
            });
            paned.set_wide_handle(true);
            let mut first_path = path.clone();
            first_path.push(false);
            let mut second_path = path.clone();
            second_path.push(true);
            paned.set_start_child(Some(&render_node(state, first, first_path)));
            paned.set_end_child(Some(&render_node(state, second, second_path)));
            paned.connect_map({
                let ratio = *ratio;
                move |paned| {
                    let size = if paned.orientation() == gtk::Orientation::Horizontal {
                        paned.width()
                    } else {
                        paned.height()
                    };
                    if size > 0 {
                        paned.set_position((f64::from(size) * ratio).round() as i32);
                    }
                }
            });
            paned.connect_position_notify({
                let state = state.clone();
                move |paned| {
                    let size = if paned.orientation() == gtk::Orientation::Horizontal {
                        paned.width()
                    } else {
                        paned.height()
                    };
                    if size > 0 {
                        let ratio = f64::from(paned.position()) / f64::from(size);
                        state.borrow_mut().workspace.set_split_ratio(&path, ratio);
                    }
                }
            });
            paned.upcast()
        }
    }
}

fn render_leaf(state: &Rc<RefCell<UiState>>, id: PaneId) -> gtk::Widget {
    let (pane, focused) = {
        let state_ref = state.borrow();
        (
            state_ref
                .panes
                .get(&id)
                .expect("pane exists for layout leaf")
                .clone(),
            state_ref.workspace.focused() == id,
        )
    };

    if pane.terminal.parent().is_some() {
        pane.terminal.unparent();
    }

    let frame = gtk::Box::new(gtk::Orientation::Vertical, 0);
    frame.add_css_class("terminal-pane");
    if focused {
        frame.add_css_class("focused");
    }

    let title = gtk::Label::new(Some(&format!(
        "{}  -  {}  -  {}",
        pane.title,
        status_label(pane.status),
        pane.cwd.display()
    )));
    title.set_tooltip_text(Some(&format!(
        "Pane {} - accent {}",
        pane.id.0, pane.accent
    )));
    title.set_xalign(0.0);
    title.add_css_class("pane-title");
    title.set_ellipsize(gtk::pango::EllipsizeMode::End);

    let click = gtk::GestureClick::new();
    {
        let state = state.clone();
        click.connect_pressed(move |_, _, _, _| {
            state.borrow_mut().workspace.focus(id);
            render(&state);
        });
    }
    frame.add_controller(click);
    frame.append(&title);
    frame.append(&pane.terminal);
    frame.upcast()
}

fn create_pane(id: PaneId, cwd: PathBuf, config: &TerminalConfig, theme: &EffectiveTheme) -> Pane {
    let terminal = vte::Terminal::new();
    terminal.set_scrollback_lines(config.scrollback_lines);
    terminal.set_font(Some(&gtk::pango::FontDescription::from_string(
        &config.font,
    )));
    terminal.add_css_class("terminal-view");
    apply_terminal_theme(&terminal, theme);
    terminal.set_hexpand(true);
    terminal.set_vexpand(true);

    let cwd_string = cwd.to_string_lossy().into_owned();
    let argv = [config.shell.as_str()];
    let envv = runtime_env::terminal_environment();
    let envv = envv.iter().map(String::as_str).collect::<Vec<_>>();
    terminal.spawn_async(
        vte::PtyFlags::DEFAULT,
        Some(&cwd_string),
        &argv,
        &envv,
        glib::SpawnFlags::SEARCH_PATH,
        || {},
        -1,
        None::<&gio::Cancellable>,
        move |result| {
            if let Err(error) = result {
                eprintln!("failed to spawn terminal pane {}: {error}", id.0);
            }
        },
    );

    Pane {
        id,
        title: config.shell.clone(),
        cwd,
        status: PaneStatus::Running,
        terminal,
        accent: theme.accent.clone(),
    }
}

fn connect_pane_signals(state: &Rc<RefCell<UiState>>, id: PaneId) {
    let Some(terminal) = state
        .borrow()
        .panes
        .get(&id)
        .map(|pane| pane.terminal.clone())
    else {
        return;
    };

    let focus_controller = gtk::EventControllerFocus::new();
    {
        let state = state.clone();
        focus_controller.connect_enter(move |_| {
            if let Ok(mut state) = state.try_borrow_mut() {
                state.workspace.focus(id);
            }
        });
    }
    terminal.add_controller(focus_controller);

    {
        let state = state.clone();
        terminal.connect_window_title_changed(move |terminal| {
            if let Some(title) = terminal.window_title() {
                let Ok(mut state) = state.try_borrow_mut() else {
                    return;
                };
                if let Some(pane) = state.panes.get_mut(&id) {
                    pane.title = title.to_string();
                }
            }
        });
    }

    {
        let state = state.clone();
        terminal.connect_child_exited(move |_, status| {
            {
                let Ok(mut state_ref) = state.try_borrow_mut() else {
                    return;
                };
                if let Some(pane) = state_ref.panes.get_mut(&id) {
                    pane.status = PaneStatus::Exited(status);
                }
            }
            render(&state);
        });
    }
}

fn install_window_clipboard_shortcuts(
    window: &adw::ApplicationWindow,
    state: &Rc<RefCell<UiState>>,
) {
    let key_controller = gtk::EventControllerKey::new();
    key_controller.set_propagation_phase(gtk::PropagationPhase::Capture);
    {
        let state = state.clone();
        key_controller.connect_key_pressed(move |_, key, _, modifiers| {
            if !modifiers.contains(gdk::ModifierType::CONTROL_MASK) {
                return glib::Propagation::Proceed;
            }

            match key {
                gdk::Key::c | gdk::Key::C => {
                    let state_ref = state.borrow();
                    let focused = state_ref.workspace.focused();
                    let selected_terminal = state_ref
                        .panes
                        .get(&focused)
                        .filter(|pane| pane.terminal.has_selection())
                        .or_else(|| {
                            state_ref
                                .panes
                                .values()
                                .find(|pane| pane.terminal.has_selection())
                        })
                        .map(|pane| pane.terminal.clone());
                    drop(state_ref);

                    if let Some(terminal) = selected_terminal {
                        terminal.copy_clipboard_format(vte::Format::Text);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
                gdk::Key::v | gdk::Key::V => {
                    let terminal = {
                        let state_ref = state.borrow();
                        state_ref
                            .panes
                            .get(&state_ref.workspace.focused())
                            .map(|pane| pane.terminal.clone())
                    };

                    if let Some(terminal) = terminal {
                        paste_clipboard_into_terminal(&terminal);
                        glib::Propagation::Stop
                    } else {
                        glib::Propagation::Proceed
                    }
                }
                _ => glib::Propagation::Proceed,
            }
        });
    }
    window.add_controller(key_controller);
}

fn paste_clipboard_into_terminal(terminal: &vte::Terminal) {
    let Some(display) = gdk::Display::default() else {
        terminal.paste_clipboard();
        return;
    };

    let clipboard = display.clipboard();
    let formats = clipboard.formats();
    let mime_types = formats.mime_types();
    let has_plain_text = mime_types.iter().any(|mime| {
        matches!(
            mime.as_str(),
            "text/plain" | "text/plain;charset=utf-8" | "UTF8_STRING" | "STRING"
        )
    });
    let has_image = formats.contains_type(gdk::Texture::static_type())
        || mime_types
            .iter()
            .any(|mime| mime.as_str().starts_with("image/"));

    if has_plain_text || !has_image {
        terminal.paste_clipboard();
        return;
    }

    let terminal = terminal.clone();
    clipboard.read_texture_async(gio::Cancellable::NONE, move |result| {
        let Ok(Some(texture)) = result else {
            return;
        };

        match save_clipboard_texture(&texture) {
            Ok(path) => terminal.paste_text(&shell_quote_path(&path)),
            Err(error) => eprintln!("failed to paste clipboard image: {error}"),
        }
    });
}

fn save_clipboard_texture(texture: &gdk::Texture) -> anyhow::Result<PathBuf> {
    let directory = dirs::cache_dir()
        .unwrap_or_else(env::temp_dir)
        .join("linux-cmd-dashboard")
        .join("clipboard");
    fs::create_dir_all(&directory)?;

    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
    let path = directory.join(format!("clipboard-image-{timestamp}.png"));
    texture.save_to_png(&path)?;
    Ok(path)
}

fn shell_quote_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    format!("'{}'", text.replace('\'', "'\\''"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shell_quote_path_handles_spaces_and_quotes() {
        assert_eq!(
            shell_quote_path(Path::new("/tmp/codex image's.png")),
            "'/tmp/codex image'\\''s.png'"
        );
    }
}

fn apply_terminal_theme(terminal: &vte::Terminal, theme: &EffectiveTheme) {
    terminal.set_clear_background(!theme.transparent_background);
    terminal.set_bold_is_bright(false);

    let foreground = gdk::RGBA::parse(&theme.foreground).ok();
    let background = terminal_background_color(theme).and_then(|mut color| {
        color.set_alpha(terminal_background_alpha(theme) as f32);
        Some(color)
    });
    let terminal_colors = parse_ansi_palette(theme);

    match (foreground, terminal_colors) {
        (Some(foreground), Some(colors)) => {
            let colors = colors.iter().collect::<Vec<_>>();
            terminal.set_colors(Some(&foreground), background.as_ref(), &colors);
        }
        (Some(foreground), None) => terminal.set_color_foreground(&foreground),
        (None, Some(colors)) => {
            let colors = colors.iter().collect::<Vec<_>>();
            terminal.set_colors(None, background.as_ref(), &colors);
        }
        (None, None) => {
            if let Some(background) = background.as_ref() {
                terminal.set_color_background(background);
            }
        }
    }

    if let Ok(color) = gdk::RGBA::parse(&theme.cursor) {
        terminal.set_color_cursor(Some(&color));
    }

    if let Some(background) = background {
        terminal.set_color_background(&background);
    }
}

fn terminal_background_alpha(theme: &EffectiveTheme) -> f64 {
    if theme.transparent_background {
        f64::from(terminal_background_opacity(theme))
    } else {
        1.0
    }
}

fn terminal_background_rgba(theme: &EffectiveTheme) -> Option<String> {
    let color = gdk::RGBA::parse(&theme.background).ok()?;
    Some(format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.3})",
        color.red() * 255.0,
        color.green() * 255.0,
        color.blue() * 255.0,
        terminal_background_alpha(theme)
    ))
}

fn terminal_background_color(theme: &EffectiveTheme) -> Option<gdk::RGBA> {
    let color = gdk::RGBA::parse(&theme.background).ok()?;
    Some(color)
}

fn parse_ansi_palette(theme: &EffectiveTheme) -> Option<Vec<gdk::RGBA>> {
    if theme.ansi_colors.is_empty() {
        return None;
    }

    let mut colors = Vec::with_capacity(theme.ansi_colors.len());
    for color in &theme.ansi_colors {
        colors.push(gdk::RGBA::parse(color).ok()?);
    }

    if colors.len() == theme.ansi_colors.len() {
        Some(colors)
    } else {
        None
    }
}

fn current_terminal_cwd(pane: &Pane) -> Option<PathBuf> {
    pane.terminal
        .current_directory_uri()
        .and_then(|uri| gio::File::for_uri(&uri).path())
}

fn inherited_cwd(fallback: Option<&PathBuf>) -> PathBuf {
    fallback
        .cloned()
        .or_else(|| env::current_dir().ok())
        .unwrap_or_else(|| PathBuf::from("/"))
}

fn status_label(status: PaneStatus) -> String {
    match status {
        PaneStatus::Running => "running".to_string(),
        PaneStatus::Exited(status) => format!("exited {status}"),
    }
}

fn configure_style_manager(theme: &EffectiveTheme) {
    let scheme = match theme.prefer_dark {
        Some(true) => adw::ColorScheme::PreferDark,
        Some(false) => adw::ColorScheme::PreferLight,
        None => adw::ColorScheme::Default,
    };
    adw::StyleManager::default().set_color_scheme(scheme);
}

fn install_icon_theme() {
    let Some(display) = gdk::Display::default() else {
        return;
    };
    let icon_theme = gtk::IconTheme::for_display(&display);
    icon_theme.add_search_path(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/icons"));
    gtk::Window::set_default_icon_name(APP_ID);
}

fn install_css(theme: &EffectiveTheme) {
    let provider = gtk::CssProvider::new();
    let title_background =
        css_rgba(&theme.titlebar_background, 1.0).unwrap_or_else(|| "#202326".to_string());
    let terminal_background =
        terminal_background_rgba(theme).unwrap_or_else(|| theme.background.clone());
    let border = css_rgba(&theme.foreground, 0.18).unwrap_or_else(|| "#252a33".to_string());
    let foreground = &theme.foreground;
    let accent = &theme.accent;
    provider.load_from_string(&format!(
        "
        window {{
            background: transparent;
        }}
        .terminal-toolbar,
        .terminal-toolbar > *,
        .terminal-content {{
            background: transparent;
        }}
        .terminal-titlebar {{
            background: {title_background};
            border-bottom: 1px solid {border};
        }}
        .terminal-pane {{
            background: transparent;
            border: 1px solid {border};
            min-width: 220px;
            min-height: 150px;
        }}
        .terminal-view {{
            background: {terminal_background};
        }}
        .terminal-pane.focused {{
            border-color: {accent};
        }}
        .pane-title {{
            background: {title_background};
            color: {foreground};
            padding: 5px 8px;
            font-size: 12px;
        }}
        paned > separator {{
            background: {border};
            min-width: 5px;
            min-height: 5px;
        }}
        ",
    ));

    gtk::style_context_add_provider_for_display(
        &gdk::Display::default().expect("display is available"),
        &provider,
        gtk::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );
}

fn terminal_background_opacity(theme: &EffectiveTheme) -> f32 {
    theme.background_opacity.clamp(0.1, 1.0)
}

fn css_rgba(color: &str, alpha: f32) -> Option<String> {
    let color = gdk::RGBA::parse(color).ok()?;
    Some(format!(
        "rgba({:.0}, {:.0}, {:.0}, {:.3})",
        color.red() * 255.0,
        color.green() * 255.0,
        color.blue() * 255.0,
        alpha.clamp(0.0, 1.0)
    ))
}
