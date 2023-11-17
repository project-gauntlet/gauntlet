fn main() {
    tracing_subscriber::fmt::init();

    client::start_client();
}
