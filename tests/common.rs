use ctor::ctor;

#[ctor]
fn init() {
    #[cfg(feature = "tracing")]
    console_subscriber::init();

    #[cfg(not(feature = "tracing"))]
    pretty_env_logger::init();
}
