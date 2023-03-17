mod boards;
mod connection;
mod errors;

use crate::boards::{get_boards_id, Board};
use crate::connection::websocket_handler;

use async_nats::Client as NatsClient;
use axum::extract::Query;
use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use clap::Parser;
use futures_util::{StreamExt, TryStreamExt};
use mongodb::bson::{doc, Uuid};
use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::{Client as MongoClient, Collection, Database};
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tracing::{info, trace};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
/// Provides a websocket interface for the frontend to be updated about changes made by other users
pub struct Cli {
    #[arg(env)]
    /// The address at which this server should listen
    backend_address: SocketAddr,

    #[arg(env)]
    /// The connection url for the Publish/Subscriber server
    pubsub_connection_url: String,

    #[arg(env)]
    /// The connection url for the database server
    db_connection_url: String,

    #[arg(env, default_value("10"))]
    /// The timeout for establishing connections to the database in seconds
    db_connect_timeout_s: usize,

    #[arg(env, default_value("static/board.html"))]
    /// The path to the .html file that contains the frontend
    frontend_file: PathBuf,
}

pub struct AppState {
    mongo: MongoClient,
    boards_table: Collection<Board>,
    nats: NatsClient,
    frontend_app: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanban_backend=trace,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    info!("Starting server");

    let mongo = connect_to_mongo(&cli).await?;
    let boards_table = mongo.database("kanban").collection::<Board>("boards");
    try_seed_mongo(&boards_table).await?;

    let nats = async_nats::connect(cli.pubsub_connection_url).await?;
    let frontend_app = fs::read_to_string(cli.frontend_file)?;

    let shared_state = Arc::new(AppState {
        mongo,
        boards_table,
        nats,
        frontend_app,
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/board/:board_id", get(get_boards_id::handler))
        .route("/board/:board_id/watch", get(websocket_handler))
        .with_state(shared_state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::default().include_headers(true),
            ),
        );

    info!("Listening on http://{}", cli.backend_address);
    info!("Open http://{}/board/0 to get started", cli.backend_address);
    axum::Server::bind(&cli.backend_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    info!("Server stopped");
    Ok(())
}

async fn connect_to_mongo(cli: &Cli) -> Result<MongoClient, Box<dyn Error>> {
    let mut client_options = MongoClientOptions::parse(&cli.db_connection_url)
        .await
        .expect("Failed to parse mongo connection url!");
    client_options.app_name = Some("kanban backend".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(cli.db_connect_timeout_s as u64));
    info!("Connecting to mongodb at {:?}...", client_options.hosts);
    let db = MongoClient::with_options(client_options)?;

    info!("Connected to mongodb! Listing database names:");
    for db_name in db.list_database_names(None, None).await? {
        info!(" - {db_name}");
    }
    Ok(db)
}

async fn try_seed_mongo(board_table: &Collection<Board>) -> Result<(), Box<dyn Error>> {
    let board_count = board_table.estimated_document_count(None).await?;
    info!("The database contains {board_count} boards!");

    if board_count == 0 {
        let board_0 = Board {
            id: Uuid::new(),
            url: "shopping-list".to_string(),
            name: "Shopping list".to_string(),
            lists: vec![],
        };
        board_table.insert_one(board_0, None).await?;
    }

    Ok(())
}

async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    trace!("Sending frontend");
    Html(state.frontend_app.clone())
}
