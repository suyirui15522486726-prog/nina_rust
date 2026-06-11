fn main() {
    if let Err(error) = nina_rust::cli::run() {
        eprintln!("error: {error}");
        std::process::exit(1);
    }
}
