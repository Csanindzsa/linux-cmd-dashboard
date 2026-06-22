use std::{env, path::Path};

const WRAPPER_ONLY_ENV: &[&str] = &["APPDIR", "APPIMAGE", "ARGV0", "OWD"];

pub fn terminal_environment() -> Vec<String> {
    let appdir = env::var_os("APPDIR");
    let appdir = appdir.as_deref().map(Path::new);
    let vars = env::vars_os()
        .filter_map(|(key, value)| Some((key.into_string().ok()?, value.into_string().ok()?)));
    terminal_environment_from(vars, appdir)
}

fn terminal_environment_from<I, K, V>(vars: I, appdir: Option<&Path>) -> Vec<String>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: AsRef<str>,
{
    vars.into_iter()
        .filter_map(|(key, value)| {
            let key = key.as_ref();
            let value = value.as_ref();

            if WRAPPER_ONLY_ENV.contains(&key) {
                return None;
            }

            let value = match key {
                "LD_LIBRARY_PATH" => clean_appdir_path_list(value, appdir, "usr/lib"),
                "XDG_DATA_DIRS" => clean_appdir_path_list(value, appdir, "usr/share"),
                _ => Some(value.to_string()),
            }?;

            Some(format!("{key}={value}"))
        })
        .collect()
}

fn clean_appdir_path_list(value: &str, appdir: Option<&Path>, relative: &str) -> Option<String> {
    let Some(appdir) = appdir else {
        return Some(value.to_string());
    };

    let appdir_path = appdir.join(relative);
    let cleaned = value
        .split(':')
        .filter(|entry| !entry.is_empty())
        .filter(|entry| Path::new(entry) != appdir_path)
        .collect::<Vec<_>>()
        .join(":");

    (!cleaned.is_empty()).then_some(cleaned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn terminal_environment_removes_appimage_wrapper_state() {
        let env = terminal_environment_from(
            [
                ("APPDIR", "/home/me/.local/opt/linux-cmd-dashboard"),
                (
                    "APPIMAGE",
                    "/home/me/.local/bin/linux-cmd-dashboard.AppImage",
                ),
                ("ARGV0", "linux-cmd-dashboard"),
                ("OWD", "/home/me"),
                ("HOME", "/home/me"),
            ],
            Some(Path::new("/home/me/.local/opt/linux-cmd-dashboard")),
        );

        assert_eq!(env, vec!["HOME=/home/me"]);
    }

    #[test]
    fn terminal_environment_strips_only_bundled_library_paths() {
        let env = terminal_environment_from(
            [
                (
                    "LD_LIBRARY_PATH",
                    "/home/me/.local/opt/linux-cmd-dashboard/usr/lib:/opt/dev/lib",
                ),
                (
                    "XDG_DATA_DIRS",
                    "/home/me/.local/opt/linux-cmd-dashboard/usr/share:/usr/local/share:/usr/share",
                ),
                ("PATH", "/usr/local/bin:/usr/bin"),
            ],
            Some(Path::new("/home/me/.local/opt/linux-cmd-dashboard")),
        );

        assert_eq!(
            env,
            vec![
                "LD_LIBRARY_PATH=/opt/dev/lib",
                "XDG_DATA_DIRS=/usr/local/share:/usr/share",
                "PATH=/usr/local/bin:/usr/bin",
            ]
        );
    }

    #[test]
    fn terminal_environment_drops_empty_path_lists_after_cleaning() {
        let env = terminal_environment_from(
            [
                (
                    "LD_LIBRARY_PATH",
                    "/home/me/.local/opt/linux-cmd-dashboard/usr/lib",
                ),
                (
                    "XDG_DATA_DIRS",
                    "/home/me/.local/opt/linux-cmd-dashboard/usr/share",
                ),
                ("SHELL", "/usr/bin/fish"),
            ],
            Some(Path::new("/home/me/.local/opt/linux-cmd-dashboard")),
        );

        assert_eq!(env, vec!["SHELL=/usr/bin/fish"]);
    }

    #[test]
    fn terminal_environment_preserves_library_paths_without_appdir() {
        let env = terminal_environment_from(
            [
                ("LD_LIBRARY_PATH", "/opt/dev/lib"),
                ("XDG_DATA_DIRS", "/usr/local/share:/usr/share"),
            ],
            None,
        );

        assert_eq!(
            env,
            vec![
                "LD_LIBRARY_PATH=/opt/dev/lib",
                "XDG_DATA_DIRS=/usr/local/share:/usr/share",
            ]
        );
    }
}
