#!/usr/bin/env bash
set -euo pipefail

APP_NAME="linux-cmd-dashboard"
APP_ID="dev.codex.LinuxCmdDashboard"
DISPLAY_NAME="Linux Command Dashboard"
REPO_API="https://api.github.com/repos/Csanindzsa/linux-cmd-dashboard"

MODE="release"
VERSION="${VERSION:-}"
APPIMAGE_PATH=""
KEEP_DOWNLOADS="${KEEP_DOWNLOADS:-0}"
INSTALL_PREFIX="${INSTALL_PREFIX:-$HOME/.local}"
BIN_DIR="${BIN_DIR:-$INSTALL_PREFIX/bin}"
APP_DIR="${APP_DIR:-$INSTALL_PREFIX/opt/$APP_NAME}"
XDG_DATA_HOME="${XDG_DATA_HOME:-$HOME/.local/share}"
APPLICATIONS_DIR="${APPLICATIONS_DIR:-$XDG_DATA_HOME/applications}"
ICONS_DIR="${ICONS_DIR:-$XDG_DATA_HOME/icons/hicolor}"

usage() {
  cat <<USAGE
Install $DISPLAY_NAME for the current user.

Usage:
  scripts/install-linux.sh [options]

Options:
  --from-release        Download and install the latest GitHub AppImage release.
                        This is the default and does not require root.
  --version VERSION     Install a specific release version, for example 0.1.0.
  --appimage FILE       Install from an already downloaded AppImage.
  --from-source         Build with Cargo and install the native binary.
                        Requires GTK4, libadwaita, VTE GTK4, pkg-config, fish,
                        and the Rust toolchain.
  --prefix DIR          Install under DIR instead of ~/.local.
  --uninstall           Remove the user-local installation.
  -h, --help            Show this help.

Environment:
  INSTALL_PREFIX        Same as --prefix.
  KEEP_DOWNLOADS=1      Keep downloaded release files in the temporary directory.
USAGE
}

log() {
  printf '%s\n' "==> $*"
}

die() {
  printf '%s\n' "error: $*" >&2
  exit 1
}

need_command() {
  command -v "$1" >/dev/null 2>&1 || die "$1 is required"
}

download_file() {
  local url="$1"
  local output="$2"

  if command -v curl >/dev/null 2>&1; then
    curl -fsSL --retry 3 -o "$output" "$url"
  elif command -v wget >/dev/null 2>&1; then
    wget -O "$output" "$url"
  else
    die "curl or wget is required to download release files"
  fi
}

release_api_url() {
  if [[ -n "$VERSION" ]]; then
    printf '%s/releases/tags/v%s\n' "$REPO_API" "${VERSION#v}"
  else
    printf '%s/releases/latest\n' "$REPO_API"
  fi
}

safe_replace_dir() {
  local source="$1"
  local target="$2"

  [[ -n "$target" ]] || die "empty install target"
  [[ "$target" != "/" ]] || die "refusing to replace /"
  [[ "$(basename "$target")" == "$APP_NAME" ]] ||
    die "APP_DIR must end in $APP_NAME: $target"

  rm -rf "$target"
  install -d "$(dirname "$target")"
  cp -a "$source" "$target"
}

install_desktop_entry() {
  install -d "$APPLICATIONS_DIR"
  cat >"$APPLICATIONS_DIR/$APP_NAME.desktop" <<DESKTOP
[Desktop Entry]
Name=$DISPLAY_NAME
Comment=Terminal tiles dashboard for Linux commands
Exec=$BIN_DIR/$APP_NAME
Icon=$APP_ID
Type=Application
Categories=System;TerminalEmulator;
Terminal=false
StartupNotify=true
DESKTOP
}

install_icon_cache_index() {
  if [[ ! -f "$ICONS_DIR/index.theme" && -f /usr/share/icons/hicolor/index.theme ]]; then
    install -Dm644 /usr/share/icons/hicolor/index.theme "$ICONS_DIR/index.theme"
  fi
}

refresh_desktop_caches() {
  install_icon_cache_index

  if command -v gtk-update-icon-cache >/dev/null 2>&1 && [[ -f "$ICONS_DIR/index.theme" ]]; then
    gtk-update-icon-cache -q "$ICONS_DIR" >/dev/null 2>&1 || true
  fi

  if command -v update-desktop-database >/dev/null 2>&1; then
    update-desktop-database "$APPLICATIONS_DIR" >/dev/null 2>&1 || true
  fi
}

install_wrapper_for_appdir() {
  install -d "$BIN_DIR"
  cat >"$BIN_DIR/$APP_NAME" <<WRAPPER
#!/usr/bin/env sh
APPDIR="$APP_DIR"
export APPDIR
export LD_LIBRARY_PATH="\$APPDIR/usr/lib\${LD_LIBRARY_PATH:+:\$LD_LIBRARY_PATH}"
export XDG_DATA_DIRS="\$APPDIR/usr/share\${XDG_DATA_DIRS:+:\$XDG_DATA_DIRS}"
exec "\$APPDIR/usr/bin/$APP_NAME" "\$@"
WRAPPER
  chmod 755 "$BIN_DIR/$APP_NAME"
}

copy_icons_from_appdir() {
  if [[ -d "$APP_DIR/usr/share/icons/hicolor" ]]; then
    install -d "$ICONS_DIR"
    cp -a "$APP_DIR/usr/share/icons/hicolor/." "$ICONS_DIR/"
  elif [[ -f "$APP_DIR/$APP_ID.svg" ]]; then
    install -Dm644 "$APP_DIR/$APP_ID.svg" \
      "$ICONS_DIR/scalable/apps/$APP_ID.svg"
  fi
}

verify_sha256() {
  local appimage="$1"
  local sums="$2"
  local asset_name
  local expected
  local actual

  need_command sha256sum

  asset_name="$(basename "$appimage")"
  expected="$(
    awk -v name="$asset_name" '$2 == name || $2 == "dist/" name { print $1; exit }' "$sums"
  )"

  [[ -n "$expected" ]] || die "no SHA256 entry found for $asset_name"

  actual="$(sha256sum "$appimage" | awk '{ print $1 }')"
  [[ "$actual" == "$expected" ]] ||
    die "SHA256 mismatch for $asset_name"
}

extract_and_install_appimage() {
  local appimage="$1"
  local work_dir="$2"

  chmod +x "$appimage"
  rm -rf "$work_dir/squashfs-root"

  log "Extracting AppImage payload"
  (
    cd "$work_dir"
    "$appimage" --appimage-extract >/dev/null
  )

  [[ -x "$work_dir/squashfs-root/usr/bin/$APP_NAME" ]] ||
    die "extracted AppImage does not contain usr/bin/$APP_NAME"

  log "Installing app files to $APP_DIR"
  safe_replace_dir "$work_dir/squashfs-root" "$APP_DIR"
  install_wrapper_for_appdir
  copy_icons_from_appdir
  install_desktop_entry
  refresh_desktop_caches
}

install_from_release() {
  local tmp_dir="$1"
  local release_json="$tmp_dir/release.json"
  local appimage_url
  local sums_url
  local appimage
  local sums

  log "Reading release metadata"
  download_file "$(release_api_url)" "$release_json"

  appimage_url="$(
    sed -nE 's/.*"browser_download_url": "([^"]*linux-cmd-dashboard-[^"]*-x86_64\.AppImage)".*/\1/p' \
      "$release_json" | head -n1
  )"
  sums_url="$(
    sed -nE 's/.*"browser_download_url": "([^"]*SHA256SUMS)".*/\1/p' \
      "$release_json" | head -n1
  )"

  [[ -n "$appimage_url" ]] || die "release does not include an x86_64 AppImage"
  [[ -n "$sums_url" ]] || die "release does not include SHA256SUMS"

  appimage="$tmp_dir/$(basename "$appimage_url")"
  sums="$tmp_dir/SHA256SUMS"

  log "Downloading $(basename "$appimage")"
  download_file "$appimage_url" "$appimage"
  log "Downloading SHA256SUMS"
  download_file "$sums_url" "$sums"
  log "Verifying release checksum"
  verify_sha256 "$appimage" "$sums"

  extract_and_install_appimage "$appimage" "$tmp_dir"
}

install_from_appimage() {
  local tmp_dir="$1"
  local appimage_abs

  [[ -f "$APPIMAGE_PATH" ]] || die "AppImage not found: $APPIMAGE_PATH"
  need_command realpath
  appimage_abs="$(realpath "$APPIMAGE_PATH")"
  extract_and_install_appimage "$appimage_abs" "$tmp_dir"
}

install_source_icons() {
  install -Dm644 "assets/icons/hicolor/scalable/apps/$APP_ID.svg" \
    "$ICONS_DIR/scalable/apps/$APP_ID.svg"
  install -Dm644 "assets/icons/hicolor/256x256/apps/$APP_ID.png" \
    "$ICONS_DIR/256x256/apps/$APP_ID.png"
}

install_from_source() {
  need_command cargo
  need_command pkg-config

  pkg-config --exists gtk4 ||
    die "missing GTK4 development files; install gtk4/libgtk-4-dev"
  pkg-config --exists libadwaita-1 ||
    die "missing libadwaita development files; install libadwaita"
  pkg-config --exists vte-2.91-gtk4 ||
    die "missing VTE GTK4 development files; install vte4/libvte-2.91-gtk4-dev"
  command -v fish >/dev/null 2>&1 ||
    die "fish is required because it is the default configured shell"

  log "Building release binary"
  cargo build --release

  log "Installing binary to $BIN_DIR"
  install -Dm755 "target/release/$APP_NAME" "$BIN_DIR/$APP_NAME"
  install_source_icons
  install_desktop_entry
  refresh_desktop_caches
}

uninstall_app() {
  log "Removing $DISPLAY_NAME user-local installation"
  rm -f "$BIN_DIR/$APP_NAME"
  rm -f "$APPLICATIONS_DIR/$APP_NAME.desktop"
  rm -f "$ICONS_DIR/scalable/apps/$APP_ID.svg"
  rm -f "$ICONS_DIR/256x256/apps/$APP_ID.png"

  if [[ -n "$APP_DIR" && "$APP_DIR" != "/" && "$(basename "$APP_DIR")" == "$APP_NAME" ]]; then
    rm -rf "$APP_DIR"
  fi

  refresh_desktop_caches
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --from-release)
      MODE="release"
      shift
      ;;
    --from-source)
      MODE="source"
      shift
      ;;
    --appimage)
      [[ $# -ge 2 ]] || die "--appimage requires a file path"
      MODE="appimage"
      APPIMAGE_PATH="$2"
      shift 2
      ;;
    --version)
      [[ $# -ge 2 ]] || die "--version requires a version"
      VERSION="$2"
      shift 2
      ;;
    --prefix)
      [[ $# -ge 2 ]] || die "--prefix requires a directory"
      INSTALL_PREFIX="$2"
      BIN_DIR="$INSTALL_PREFIX/bin"
      APP_DIR="$INSTALL_PREFIX/opt/$APP_NAME"
      shift 2
      ;;
    --uninstall)
      MODE="uninstall"
      shift
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      die "unknown option: $1"
      ;;
  esac
done

case "$MODE" in
  release|appimage)
    tmp_dir="$(mktemp -d "${TMPDIR:-/tmp}/$APP_NAME-install.XXXXXX")"
    if [[ "$KEEP_DOWNLOADS" != "1" ]]; then
      trap 'rm -rf "$tmp_dir"' EXIT
    else
      log "Using temporary directory $tmp_dir"
    fi

    if [[ "$MODE" == "release" ]]; then
      install_from_release "$tmp_dir"
    else
      install_from_appimage "$tmp_dir"
    fi
    ;;
  source)
    install_from_source
    ;;
  uninstall)
    uninstall_app
    ;;
  *)
    die "unsupported install mode: $MODE"
    ;;
esac

if [[ "$MODE" != "uninstall" ]]; then
  log "Installed $DISPLAY_NAME"
  log "Run: $BIN_DIR/$APP_NAME"
fi
