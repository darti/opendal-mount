use log::{error, info};
use nfsserve::service::NFSService;
use opendal::{services::Fs, Operator};
use opendal_mount::{
    mount::{FsMounter, Mounter},
    OpendalFs,
};
use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();

    let mut sig_term = signal(SignalKind::terminate())?;

    let mut builder = Fs::default();
    builder.root(".");
    let op = Operator::new(builder)?.finish();

    let cancellation_token = CancellationToken::new();
    let task_tracker = TaskTracker::new();

    let nfs = NFSService::new(
        OpendalFs::new(op),
        "localhost:0",
        Some(cancellation_token.clone()),
        Some(task_tracker.clone()),
    )
    .await?;

    let addr = nfs.local_addr();

    let _h = task_tracker.spawn(async move {
        match nfs.handle().await {
            Ok(_) => info!("NFS service stopped"),
            Err(e) => error!("Error handling NFS service: {:?}", e),
        };
    });

    info!("Registered NFS service , listening on {:?}", addr);

    let mount_point = "../mnt";

    FsMounter::mount(&addr.ip().to_string(), addr.port(), "", &mount_point, false).await?;

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
