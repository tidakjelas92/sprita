pub fn handle_error(error: &str) {
    error!("{}", error);
    std::process::exit(1);
}
