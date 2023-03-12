mod connection;

use crate::connection::websocket_handler;

use async_nats::Client as NatsClient;
use axum::{extract::Path, extract::State, response::Html, routing::get, Router};
use clap::Parser;
use std::error::Error;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
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
    /// The connection url of the Publish/Subscriber server
    pubsub_connection_url: String,

    #[arg(env)]
    /// The path to the .html file that contains the frontend
    frontend_file: PathBuf,
}

pub struct AppState {
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

    let nats = async_nats::connect(cli.pubsub_connection_url).await?;
    let frontend_app = fs::read_to_string(cli.frontend_file)?;
    let shared_state = Arc::new(AppState { nats, frontend_app });

    let app = Router::new()
        .route("/", get(index_handler))
        .route("/board/:board_id", get(board_handler))
        .route("/board/:board_id/ws", get(websocket_handler))
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

async fn index_handler() -> &'static str {
    trace!("Sending greetings from index...");
    "Hello, please navigate to /board/0"
}

async fn board_handler(
    State(state): State<Arc<AppState>>,
    Path(_board_id): Path<u128>,
) -> Html<String> {
    trace!("Sending frontend");
    Html(state.frontend_app.clone())
}
