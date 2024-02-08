use anyhow::Ok;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use clap::Parser;
use log::info;
use nfsserve::tcp::NFSTcp;
use nfsserve::tcp::NFSTcpListener;
use opendal_mount::{
    schema::{Mutation, Query},
    MultiplexedFs,
};

use tokio::{
    net::TcpListener,
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
    #[arg(long, default_value = "127.0.0.1")]
    host: String,

    #[arg(long, default_value = "1200")]
    port: u16,

    #[arg(long, default_value = "127.0.0.1:8080")]
    graphql_addr: String,
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    let args = Args::parse();

    let fs = MultiplexedFs::new(&args.host, args.port);
    let fs_nfs = fs.clone();

    info!("Starting FS");
    tokio::spawn(async move {
        info!("Serving FS on {}:{}", args.host, args.port);

        let listener =
            NFSTcpListener::bind(&format!("{}:{}", args.host, args.port), fs_nfs).await?;

        listener.handle_forever().await
    });

    info!("Starting GraphQL");
    tokio::spawn(async move {
        let schema = Schema::build(Query, Mutation, EmptySubscription)
            .data(fs)
            .finish();

        let app = Router::new().route(
            "/",
            get(graphql_playground).post_service(GraphQL::new(schema)),
        );

        axum::serve(TcpListener::bind(&args.graphql_addr).await.unwrap(), app).await
    });

    let mut sig_term = signal(SignalKind::terminate())?;

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

    Ok(())
}
