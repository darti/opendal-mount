use log::info;
use opendal::{services::Fs, Operator};
use opendal_mount::{
    mount::{FsMounter, Mounter},
    NFSServer,
};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let mut builder = Fs::default();
    builder.root(".");
    let op = Operator::new(builder)?.finish();

    let nfs = NFSServer::default();

    let id = nfs.register("localhost:0", op).await?;
    let addr = nfs.local_addr(&id).await;

    info!(
        "Registered NFS service with id {}, listening on {:?}",
        id, addr
    );

    let mount_point = "../mnt";

    if let Some(addr) = addr {
        FsMounter::mount(&addr.ip().to_string(), addr.port(), "", &mount_point, false).await?;
    }

    info!("Running, press Ctrl-C to stop");

    loop {
        select! {
            _ = signal::ctrl_c() => {
                info!("Received Ctrl-C, stopping");
                break;
            }
            _ = sig_term.recv() => {
                info!("Received SIGTERM, stopping");
                break;
            }
        }
    }

    info!("Unmounting NFS service");
    FsMounter::umount(&mount_point).await?;

    info!("Clean exit");

    Ok(())
}
