fn main() {
    tracing_subscriber::fmt::init();

    server::start_server();
}
