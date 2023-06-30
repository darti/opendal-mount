use ctor::ctor;

#[ctor]
fn init() {
    console_subscriber::init();
}
