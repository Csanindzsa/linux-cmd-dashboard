fn main() {
    #[cfg(feature = "gui")]
    linux_cmd_dashboard::app::run();

    #[cfg(not(feature = "gui"))]
    eprintln!("linux-cmd-dashboard was built without the `gui` feature");
}
