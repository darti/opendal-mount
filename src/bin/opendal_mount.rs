use anyhow::Ok;

use async_graphql::{
    http::{playground_source, GraphQLPlaygroundConfig},
    EmptySubscription, Schema,
};
use async_graphql_axum::GraphQL;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    serve::Serve,
    Router,
};
use clap::Parser;
use log::info;
use opendal::{services::Fs, Operator};
use opendal_mount::{
    schema::{Mutation, Query},
    serve, MultiplexedFs,
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
    #[arg(short, long, default_value = "127.0.0.1:12000")]
    fs_addr: String,

    #[arg(short, long, default_value = "127.0.0.1:8080")]
    graphql_addr: String,
}

async fn graphql_playground() -> impl IntoResponse {
    Html(playground_source(GraphQLPlaygroundConfig::new("/")))
}

async fn serve_graphql(addr: &str) -> Serve<Router, Router> {
    let schema = Schema::build(Query, Mutation, EmptySubscription)
        .data(MultiplexedFs::default())
        .finish();

    let app = Router::new().route(
        "/",
        get(graphql_playground).post_service(GraphQL::new(schema)),
    );

    axum::serve(TcpListener::bind(addr).await.unwrap(), app)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    console_subscriber::init();
    let args = Args::parse();

    let mut builder = Fs::default();
    builder.root("/Users/matthieudartiguenave/Projects/rmk/dump/xochitl");

    let op = Operator::new(builder)?.finish();

    info!("Starting FS");
    tokio::spawn(async move { serve(&args.fs_addr, op).await });

    info!("Starting GraphQL");
    tokio::spawn(async move { serve_graphql(&args.graphql_addr).await.await });

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
