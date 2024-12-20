fn main() {
    tracing_subscriber::fmt::init();

    gauntlet_management_client::start_management_client();
}
