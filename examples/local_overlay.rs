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

fn init_service(base_root: &str, overlay_root: &str) -> opendal::Result<Operator> {
    let mut base_builder = Fs::default();
    base_builder.root(base_root);

    let mut overlay_builder = Fs::default();
    overlay_builder.root(overlay_root);

    let overlay = Overlay::new(overlay_builder)?;

    Ok(Operator::new(base_builder)?.layer(overlay).finish())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let args: Vec<String> = std::env::args().collect();

    if args.len() != 4 {
        return Err(anyhow!("Usage: local_overlay <mnt_point> <base> <overlay>"));
    }

    let mount_point = &args[1];

    let fs = init_service(&args[2], &args[3])?;
    let fs = OpendalFs::new(fs, false);

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
