fn main() {
    if let Err(error) = nova_go::run_and_print(std::env::args()) {
        eprintln!("{error}");
        std::process::exit(1);
    }
}
