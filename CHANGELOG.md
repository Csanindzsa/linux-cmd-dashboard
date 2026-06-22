# Changelog

## v0.1.1 - 2026-06-22

### Fixed

- Prevent extracted AppImage runtime libraries from leaking into terminal panes.
  Shells spawned inside the dashboard now receive a sanitized environment that
  strips the bundled `LD_LIBRARY_PATH`, extracted AppImage `XDG_DATA_DIRS`, and
  wrapper-only variables such as `APPDIR`, `APPIMAGE`, `ARGV0`, and `OWD`.
- Fix commands such as `fish`, `ls`/`eza`, and other dynamically linked tools
  accidentally loading bundled AppImage libraries instead of host system
  libraries.

### Added

- Add a user-local Linux installer at `scripts/install-linux.sh`.
- The installer downloads release metadata, verifies `SHA256SUMS`, extracts the
  AppImage payload when needed, installs the launcher, desktop entry, and icons,
  and supports source installs with `--from-source`.
- Publish `install-linux.sh` as a release asset so users can install without
  cloning the repository.

### Changed

- Document the low-effort install flow, source install flow, and uninstall
  command in the README.
- Add installer and runtime-environment tests to the documented check list.

## v0.1.0 - 2026-06-21

Initial public release with Linux x86_64 packages.
