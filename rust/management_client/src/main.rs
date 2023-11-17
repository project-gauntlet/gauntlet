fn main() {
    tracing_subscriber::fmt::init();

    management_client::start_management_client();
}
