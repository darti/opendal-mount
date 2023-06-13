use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{services::Sftp, Operator};
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

    let endpoint = "10.11.99.1";
    let user = "root";
    let key_file = "~/.ssh/id_remarkable";
    let base = "/home/root/.local/share/remarkable";

    let mut builder = Sftp::default();
    builder
        .endpoint(endpoint)
        .user(user)
        .key(key_file)
        .root(base);

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
