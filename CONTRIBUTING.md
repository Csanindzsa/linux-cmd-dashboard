# Contributing

Thanks for taking the time to improve `linux-cmd-dashboard`.

## License Expectations

By contributing, you agree that your contribution is provided under the same
PolyForm Noncommercial License 1.0.0 as the rest of the project.

Forks, experiments, and hobby/non-commercial pull requests are welcome.
Commercial use is not granted by this repository license.

## Before Opening a Pull Request

1. Keep changes focused. Separate UI behavior, docs, and large refactors when
   possible.
2. Run formatting and automated checks:

   ```sh
   cargo fmt --all
   cargo test --no-default-features --lib
   cargo check --no-default-features
   cargo check
   ```

3. For UI changes, include a short note about the desktop environment and
   manual GTK/VTE behavior you tested.
4. Avoid committing build artifacts from `target/` or editor-local files.

## Development Notes

- The layout model in `src/layout.rs` is independent from GTK and should stay
  covered by unit tests.
- Config changes should preserve `#[serde(default)]` compatibility so older
  config files keep loading.
- Project notes and current architecture details live in `knowledge-base/`.
