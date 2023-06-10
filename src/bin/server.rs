use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{services::Fs, Operator};
use opendal_mount::OpendalFs;
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

const HOSTPORT: u32 = 12000;

#[tokio::main(flavor = "multi_thread")]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut builder = Fs::default();
    builder.root("./src");

    let fs = OpendalFs::new(Operator::new(builder)?.finish(), false);

    tokio::spawn(async {
        let listener = NFSTcpListener::bind(&format!("127.0.0.1:{HOSTPORT}"), fs)
            .await
            .unwrap();

        listener.handle_forever().await.unwrap();
    });

    let mut sig_term = signal(SignalKind::terminate())?;

    select! {
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C, sending unmount signals");
        }
        _ = sig_term.recv() => {
            info!("Received SIGTERM, sending unmount signal");
        }
    };

    Ok(())
}
