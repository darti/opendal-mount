use anyhow::Ok;

use clap::Parser;
use log::info;
use opendal::{services::Fs, Operator};
use opendal_mount::serve;

use tokio::{
    select,
    signal::{
        self,
        unix::{signal, SignalKind},
    },
};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// ip address to bind to
    #[arg(short, long, default_value = "127.0.0.1:12000")]
    ip: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    let args = Args::parse();

    let mut builder = Fs::default();
    builder.root("/Users/matthieudartiguenave/Projects/rmk/dump/xochitl");

    let op = Operator::new(builder)?.finish();

    info!("Starting server");
    tokio::spawn(async move { serve(&args.ip, op).await });

    let mut sig_term = signal(SignalKind::terminate())?;

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

    Ok(())
}
