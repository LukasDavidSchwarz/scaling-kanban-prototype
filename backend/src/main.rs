mod boards;
mod connection;

use crate::boards::{get_boards, get_boards_id, post_boards, put_boards_id, Board, Task, TaskList};
use crate::connection::websocket_handler;

use async_nats::Client as NatsClient;

use axum::routing::get_service;
use axum::{
    routing::{get, post, put},
    Router,
};
use clap::Parser;
use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::{Client as MongoClient, Collection};
use std::error::Error;
use std::fs::canonicalize;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
use tower_http::services::ServeDir;
use tracing::{error, info};
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

    #[arg(env)]
    /// The name of the mongo database
    db_name: String,

    #[arg(env, default_value("10"))]
    /// The timeout for establishing connections to the database in seconds
    db_connect_timeout_s: usize,

    #[arg(env)]
    /// The path to the frontend build (relative to $CARGO_MANIFEST_DIR environment variable)
    frontend_build: Option<PathBuf>,
}

pub struct AppState {
    boards_table: Collection<Board>,
    nats: NatsClient,
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

    // setup DB and Pub/Sub connections
    let mongo = connect_to_mongo(&cli).await?;
    let boards_table = mongo
        .default_database()
        .expect("No database specified in mongo connection string!")
        .collection::<Board>("boards");
    ensure_at_least_one_board(&boards_table).await?;
    info!("Connecting to nats at '{}'...", cli.pubsub_connection_url);
    let nats = async_nats::connect(cli.pubsub_connection_url).await?;
    let shared_state = Arc::new(AppState { boards_table, nats });

    let app = app(shared_state, cli.frontend_build)?;
    info!("Starting server...");
    info!("Open http://{} to get started", cli.backend_address);
    axum::Server::bind(&cli.backend_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    info!("Server stopped");
    Ok(())
}

fn app(
    shared_state: Arc<AppState>,
    frontend_app_dir: Option<PathBuf>,
) -> Result<Router, Box<dyn Error>> {
    let mut app = Router::new()
        .route("/api/v1/boards", get(get_boards::handler))
        .route("/api/v1/boards", post(post_boards::handler))
        .route("/api/v1/boards/:board_id", get(get_boards_id::handler))
        .route("/api/v1/boards/:board_id", put(put_boards_id::handler))
        .route("/api/v1/boards/:board_id/watch", get(websocket_handler))
        .with_state(shared_state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::default().include_headers(true),
            ),
        )
        // TODO: Set better CORS policy:
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        );

    // register service for serving frontend app
    if let Some(mut frontend_app_dir) = frontend_app_dir {
        frontend_app_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(frontend_app_dir);
        frontend_app_dir = canonicalize(&frontend_app_dir).map_err(|err| {
            error!(
                "Failed to canonicalize path to frontend build '{:?}': {}",
                frontend_app_dir, err
            );
            err
        })?;
        info!("Serving frontend from: {:?}", frontend_app_dir);

        app = app.fallback_service(get_service(ServeDir::new(frontend_app_dir)))
    }

    Ok(app)
}

async fn connect_to_mongo(cli: &Cli) -> Result<MongoClient, Box<dyn Error>> {
    let mut client_options = MongoClientOptions::parse(&cli.db_connection_url).await?;
    client_options.default_database = Some(cli.db_name.clone());
    client_options.app_name = Some("kanban backend".into());
    client_options.connect_timeout = Some(Duration::from_secs(cli.db_connect_timeout_s as u64));
    info!("Connecting to mongodb at {:?}...", client_options.hosts);
    let db = MongoClient::with_options(client_options)?;
    Ok(db)
}

async fn ensure_at_least_one_board(
    board_table: &Collection<Board>,
) -> Result<Board, Box<dyn Error>> {
    let board_count = board_table.estimated_document_count(None).await?;
    info!("The database contains {board_count} boards!");

    if board_count == 0 {
        let initial_boards = create_initial_boards();
        board_table.insert_many(initial_boards, None).await?;
    }

    let board = board_table
        .find_one(None, None)
        .await?
        .ok_or_else(|| format!("Board query without filter returned None!"))?;
    Ok(board)
}

fn create_initial_boards() -> Vec<Board> {
    let grocery_list = TaskList::new(
        "Grocery list",
        vec![Task::new("4-6 Apples"), Task::new("Milk")],
    );
    let tutorial_list = TaskList::new(
        "Click here to rename",
        vec![Task::new("Drag tasks and lists to rearrange them")],
    );
    let shopping_board = Board::new("Shopping", vec![grocery_list, tutorial_list]);

    vec![
        shopping_board,
        Board::new("Empty Board 1", vec![]),
        Board::new("Empty Board 2", vec![]),
        Board::new("Empty Board 3", vec![]),
        Board::new("Empty Board 4", vec![]),
    ]
}
