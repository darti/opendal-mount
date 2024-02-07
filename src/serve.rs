use std::io;

use log::info;
use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::Operator;

use crate::OpendalFs;

pub async fn serve(ipstr: &str, op: Operator) -> io::Result<()> {
    info!("Serving on {}", ipstr);
    let fs = OpendalFs::new(op);
    let listener = NFSTcpListener::bind(ipstr, fs).await?;

    listener.handle_forever().await
}
