use anyhow::anyhow;
use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::{raw::Operation, services::Fs, Operator};
use opendal_mount::{OpendalFs, Overlay};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

const HOSTPORT: u32 = 12000;

fn policy(
    path: &str,
    overlay: Operator,
    base: Operator,
    op: Operation,
) -> opendal::Result<Operator> {
    match op {
        Operation::Stat => {
            if path == "world.txt" {
                Ok(overlay)
            } else {
                Ok(base)
            }
        }
        _ => Ok(base),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        return Err(anyhow!("Usage: local_overlay <mnt_point> <base> <overlay>"));
    }

    let mount_point = &args[2];

    let base = {
        let mut builder = Fs::default();
        builder.root(&args[2]);

        Operator::new(builder)?.finish()
    };

    let overlay = {
        let mut builder = Fs::default();
        builder.root(&args[3]);

        Operator::new(builder)?.finish()
    };

    let composite = {
        let mut builder = Overlay::default();
        builder.base(base).overlay(overlay).policy(policy);

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
