mod boards;
mod connection;

use crate::boards::{get_boards_id, put_boards_id, Board};
use crate::connection::websocket_handler;

use async_nats::Client as NatsClient;

use axum::{
    extract::State,
    response::Html,
    routing::{get, put},
    Router,
};
use clap::Parser;
use mongodb::options::ClientOptions as MongoClientOptions;
use mongodb::{Client as MongoClient, Collection};
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tower_http::cors::{Any, CorsLayer};
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

    #[arg(env)]
    /// The name of the mongo database
    db_name: String,

    #[arg(env, default_value("10"))]
    /// The timeout for establishing connections to the database in seconds
    db_connect_timeout_s: usize,

    #[arg(env, default_value("static/board.html"))]
    /// The path to the .html file that contains the frontend
    frontend_file: PathBuf,
}

pub struct AppState {
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
    let boards_table = mongo
        .default_database()
        .expect("No database specified in mongo connection string!")
        .collection::<Board>("boards");
    let board = ensure_at_least_one_board(&boards_table).await?;

    let nats = async_nats::connect(cli.pubsub_connection_url).await?;
    let frontend_app = fs::read_to_string(cli.frontend_file)?;

    let shared_state = Arc::new(AppState {
        boards_table,
        nats,
        frontend_app,
    });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/api/v1/board/:board_id", get(get_boards_id::handler))
        .route("/api/v1/board/:board_id", put(put_boards_id::handler))
        .route("/api/v1/board/:board_id/watch", get(websocket_handler))
        .with_state(shared_state)
        .layer(
            tower_http::trace::TraceLayer::new_for_http().make_span_with(
                tower_http::trace::DefaultMakeSpan::default().include_headers(true),
            ),
        )
        // TODO: Set better Cors policy:
        .layer(
            CorsLayer::new()
                .allow_methods(Any)
                .allow_origin(Any)
                .allow_headers(Any),
        );

    info!("Listening on http://{}", cli.backend_address);
    info!(
        "Open http://{}?boardId={} to get started",
        cli.backend_address, board.id
    );
    axum::Server::bind(&cli.backend_address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    info!("Server stopped");
    Ok(())
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

    let board = if board_count == 0 {
        let board_0 = Board::default();
        board_table.insert_one(&board_0, None).await?;
        board_0
    } else {
        board_table.find_one(None, None).await?.ok_or_else(|| {
            format!(
                "Board query without filter returned None, even though board count is {board_count}!"
            )
        })?
    };

    Ok(board)
}

async fn index_handler(State(state): State<Arc<AppState>>) -> Html<String> {
    trace!("Sending frontend");
    Html(state.frontend_app.clone())
}
