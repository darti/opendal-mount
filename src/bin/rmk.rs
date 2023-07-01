use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{
    services::{Fs, Sftp},
    Operator,
};
use opendal_mount::{overlay::policy::OsFilesPolicy, OpendalFs, Overlay};
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

    let overlay_root = "./overlay";

    let mut remote_builder = Sftp::default();

    remote_builder
        .endpoint(endpoint)
        .user(user)
        .key(key_file)
        .root(base);

    let mut overlay_builder = Fs::default();
    overlay_builder.root(overlay_root);

    let overlay = Overlay::new(overlay_builder, OsFilesPolicy)?;

    let opperator = Operator::new(remote_builder)?.layer(overlay).finish();

    let fs = OpendalFs::new(opperator, false);

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
