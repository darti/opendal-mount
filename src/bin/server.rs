use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{services::Fs, Operator};
use opendal_mount::OpendalFs;

const HOSTPORT: u32 = 12000;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut builder = Fs::default();
    builder.root("./src");

    let fs = OpendalFs::new(Operator::new(builder)?.finish());

    let listener = NFSTcpListener::bind(&format!("127.0.0.1:{HOSTPORT}"), fs)
        .await
        .unwrap();

    listener.handle_forever().await.unwrap();

    Ok(())
}
