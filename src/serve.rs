use std::io;

use nfsserve::tcp::{NFSTcp, NFSTcpListener};
use opendal::Operator;

use crate::OpendalFs;

pub async fn serve(ipstr: &str, op: Operator) -> io::Result<()> {
    let fs = OpendalFs::new(op);
    let listener = NFSTcpListener::bind(ipstr, fs).await?;

    listener.handle_forever().await
}
