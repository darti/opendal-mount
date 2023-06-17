use anyhow::anyhow;
use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{services::Fs, Operator};
use opendal_mount::{OpendalFs, Overlay};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

const HOSTPORT: u32 = 12000;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        return Err(anyhow!(
            "Usage: local_overlay <mnt_point> <folder_1> <folder_2>"
        ));
    }

    let fs1 = {
        let mut builder = Fs::default();
        builder.root(&args[1]);

        Operator::new(builder)?.finish()
    };

    let fs2 = {
        let mut builder = Fs::default();
        builder.root(&args[2]);

        Operator::new(builder)?.finish()
    };

    let composite = {
        let mut builder = Overlay::default();

        Operator::new(builder)?.finish()
    };

    let fs = OpendalFs::new(composite, false);

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
