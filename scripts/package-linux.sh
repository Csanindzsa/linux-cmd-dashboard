#!/usr/bin/env bash
set -euo pipefail

APP_NAME="linux-cmd-dashboard"
APP_ID="dev.codex.LinuxCmdDashboard"
DISPLAY_NAME="Linux Command Dashboard"
ARCH="${ARCH:-x86_64}"
DEB_ARCH="${DEB_ARCH:-amd64}"
DIST_DIR="${DIST_DIR:-dist}"
VERSION="${VERSION:-}"
PACKAGE_TAR="${PACKAGE_TAR:-1}"
PACKAGE_DEB="${PACKAGE_DEB:-1}"
PACKAGE_APPIMAGE="${PACKAGE_APPIMAGE:-1}"
LINUXDEPLOY="${LINUXDEPLOY:-linuxdeploy}"

if [[ -z "$VERSION" ]]; then
  VERSION="$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n1)"
fi

if [[ -z "$VERSION" ]]; then
  echo "Could not determine package version" >&2
  exit 1
fi

rm -rf "$DIST_DIR"
mkdir -p "$DIST_DIR"

cargo build --release

install_payload() {
  local root="$1"

  install -Dm755 "target/release/$APP_NAME" "$root/usr/bin/$APP_NAME"
  install -Dm644 "linux-cmd-dashboard.desktop" \
    "$root/usr/share/applications/linux-cmd-dashboard.desktop"
  install -Dm644 "assets/icons/hicolor/scalable/apps/$APP_ID.svg" \
    "$root/usr/share/icons/hicolor/scalable/apps/$APP_ID.svg"
  install -Dm644 "assets/icons/hicolor/256x256/apps/$APP_ID.png" \
    "$root/usr/share/icons/hicolor/256x256/apps/$APP_ID.png"
  install -Dm644 "README.md" "$root/usr/share/doc/$APP_NAME/README.md"
  install -Dm644 "LICENSE.md" "$root/usr/share/doc/$APP_NAME/LICENSE.md"
}

if [[ "$PACKAGE_TAR" == "1" ]]; then
  tar_root="$DIST_DIR/tar-root/$APP_NAME-$VERSION-$ARCH"
  mkdir -p "$tar_root"
  install_payload "$tar_root"
  tar -C "$DIST_DIR/tar-root" -czf "$DIST_DIR/$APP_NAME-$VERSION-$ARCH.tar.gz" \
    "$APP_NAME-$VERSION-$ARCH"
fi

if [[ "$PACKAGE_DEB" == "1" ]]; then
  if ! command -v dpkg-deb >/dev/null 2>&1; then
    echo "dpkg-deb is required for PACKAGE_DEB=1" >&2
    exit 1
  fi

  deb_root="$DIST_DIR/deb-root"
  install_payload "$deb_root"
  install -dm755 "$deb_root/DEBIAN"
  installed_size="$(du -sk "$deb_root/usr" | cut -f1)"

  cat >"$deb_root/DEBIAN/control" <<CONTROL
Package: $APP_NAME
Version: $VERSION
Section: utils
Priority: optional
Architecture: $DEB_ARCH
Maintainer: Csanindzsa
Depends: libgtk-4-1, libadwaita-1-0, libvte-2.91-gtk4-0, fish
Installed-Size: $installed_size
Homepage: https://github.com/Csanindzsa/linux-cmd-dashboard
Description: Single-window tiling terminal manager for Linux
 A native GTK4/libadwaita and VTE terminal dashboard for running many manual
 shell sessions in one window.
CONTROL

  dpkg-deb --build --root-owner-group "$deb_root" \
    "$DIST_DIR/${APP_NAME}_${VERSION}_${DEB_ARCH}.deb"
fi

if [[ "$PACKAGE_APPIMAGE" == "1" ]]; then
  if command -v "$LINUXDEPLOY" >/dev/null 2>&1; then
    linuxdeploy_path="$(command -v "$LINUXDEPLOY")"
  elif [[ -x "$LINUXDEPLOY" ]]; then
    linuxdeploy_path="$(realpath "$LINUXDEPLOY")"
  else
    echo "linuxdeploy is required for PACKAGE_APPIMAGE=1" >&2
    exit 1
  fi

  appdir="$DIST_DIR/appimage/$DISPLAY_NAME.AppDir"
  appimage_out="$DIST_DIR/appimage-out"
  mkdir -p "$appdir" "$appimage_out"
  install_payload "$appdir"
  appdir="$(realpath "$appdir")"
  appimage_out="$(realpath "$appimage_out")"

  (
    cd "$appimage_out"
    APPIMAGE_EXTRACT_AND_RUN=1 VERSION="$VERSION" "$linuxdeploy_path" \
      --appdir "$appdir" \
      --executable "$appdir/usr/bin/$APP_NAME" \
      --desktop-file "$appdir/usr/share/applications/linux-cmd-dashboard.desktop" \
      --icon-file "$appdir/usr/share/icons/hicolor/256x256/apps/$APP_ID.png" \
      --output appimage
  )

  appimage="$(find "$appimage_out" -maxdepth 1 -name '*.AppImage' -print -quit)"
  if [[ -z "$appimage" ]]; then
    echo "linuxdeploy did not produce an AppImage" >&2
    exit 1
  fi
  mv "$appimage" "$DIST_DIR/$APP_NAME-$VERSION-$ARCH.AppImage"
  chmod +x "$DIST_DIR/$APP_NAME-$VERSION-$ARCH.AppImage"
fi

find "$DIST_DIR" -maxdepth 1 -type f -print | sort
