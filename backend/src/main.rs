use async_nats::Client as NatsClient;
use axum::extract::{ConnectInfo, Path};
use axum::response::{Html, IntoResponse};
use axum::{
    extract::{
        ws::{Message, WebSocket, WebSocketUpgrade},
        State,
    },
    routing::get,
    Router,
};
use futures_util::stream::{SplitSink, SplitStream};
use futures_util::{SinkExt, StreamExt};
use mongodb::{options::ClientOptions, Client as MongoClient, Client};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use std::{env, fs};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: Use HashMap to map board id to all sockets listening for changes of that board
pub struct AppState {
    mongo: MongoClient,
    nats: NatsClient,
    frontend_app: String,
}

// TODO: use structopt for environment variable parsing
#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "kanban_backend=trace,tower_http=info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    println!("Starting server...");

    let mongo = connect_to_mongo().await?;
    let nats_connection_url = env::var("PUBSUB_CONNECTION_URL")
        .expect("Environment variable ´PUBSUB_CONNECTION_URL´ not set!");
    let nats = async_nats::connect(nats_connection_url).await?;
    let frontend_app = fs::read_to_string("static/board.html")?;
    let shared_state = Arc::new(AppState {
        mongo,
        nats,
        frontend_app,
    });

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

    let address = env::var("BACKEND_ADDRESS")
        .expect("Environment variable `BACKEND_ADDRESS` not set!")
        .parse()
        .expect("Failed to parse `BACKEND_ADDRESS`!");

    println!("Listening on http://{}...", address);
    axum::Server::bind(&address)
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
        .await?;

    println!("Server stopped.");
    Ok(())
}

async fn connect_to_mongo() -> Result<Client, Box<dyn Error>> {
    let db_uri =
        env::var("DB_CONNECTION_URL").expect("Environment variable ´DB_CONNECTION_URL´ not set!");
    let mut client_options = ClientOptions::parse(db_uri)
        .await
        .expect("Failed to parse environment variable ´DB_CONNECTION_URL´");
    client_options.app_name = Some("kanban backend".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(10));
    let db = MongoClient::with_options(client_options)?;

    println!("Database names:");
    for db_name in db.list_database_names(None, None).await? {
        println!(" - {}", db_name);
    }
    Ok(db)
}

async fn index_handler() -> &'static str {
    println!("Sending greetings from index...");
    "Hello, please navigate to /board/0"
}

async fn board_handler(
    State(state): State<Arc<AppState>>,
    Path(_board_id): Path<u128>,
) -> Html<String> {
    println!("Sending frontend...");
    Html(state.frontend_app.clone())
}

async fn websocket_handler(
    socket_upgrade: WebSocketUpgrade,
    ConnectInfo(address): ConnectInfo<SocketAddr>,
    State(state): State<Arc<AppState>>,
    Path(board_id): Path<u128>,
) -> impl IntoResponse {
    let connection_info = ConnectionInfo {
        app_state: state,
        address,
        board_id,
    };
    println!("Received websocket request from {connection_info}");
    socket_upgrade.on_upgrade(move |socket| handle_connection(socket, connection_info))
}

#[derive(Clone)]
struct ConnectionInfo {
    app_state: Arc<AppState>,
    address: SocketAddr,
    board_id: u128,
}

impl ConnectionInfo {
    pub fn nats_subject(&self) -> String {
        format!("board.{}", self.board_id)
    }
}

impl Display for ConnectionInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "[board.{}@{}]", self.board_id, self.address)
    }
}

async fn handle_connection(mut socket: WebSocket, connection_info: ConnectionInfo) {
    println!("Upgraded websocket of connection {connection_info}");
    match socket.send(Message::Ping(vec![1])).await {
        Ok(()) => println!("Pinged {connection_info}..."),
        Err(err) => {
            println!("Failed to ping {connection_info}: {err}");
            return;
        }
    }

    let nats = connection_info.app_state.nats.clone();
    let nats_subject = connection_info.nats_subject();
    let subscriber = nats.subscribe(nats_subject).await.expect(&format!(
        "Failed to subscribe to Nats for {connection_info}!"
    ));

    let (socket_sender, socket_receiver) = socket.split();

    let mut subscriber_task = tokio::spawn(handle_subscriber(
        subscriber,
        socket_sender,
        connection_info.clone(),
    ));
    let mut receiver_task = tokio::spawn(handle_socket_receiver(
        nats,
        socket_receiver,
        connection_info.clone(),
    ));

    tokio::select! {
        sub_result = (&mut subscriber_task) => {
            println!("Closed subscriber of {connection_info}: {:?}",sub_result);
            receiver_task.abort();
        },
        rec_result = (&mut receiver_task) => {
            println!("Closed websocket of {connection_info}: {:?}", rec_result);
            subscriber_task.abort();
        }
    }

    println!("Context of {connection_info} was destroyed.");
}

async fn handle_subscriber(
    mut subscriber: async_nats::Subscriber,
    mut socket_sender: SplitSink<WebSocket, Message>,
    connection: ConnectionInfo,
) -> Result<(), String> {
    loop {
        let message = subscriber
            .next()
            .await
            .ok_or(format!("Nats subscriber for {connection} was closed."))?;

        let message = String::from_utf8(message.payload.to_vec()).map_err(|e| e.to_string())?;
        println!("Received Nats message for {connection}: {message}");
        socket_sender
            .send(Message::Text(message))
            .await
            .map_err(|e| e.to_string())?;
    }
}

async fn handle_socket_receiver(
    nats: NatsClient,
    mut socket_receiver: SplitStream<WebSocket>,
    connection: ConnectionInfo,
) -> Result<(), String> {
    loop {
        let message = socket_receiver
            .next()
            .await
            .ok_or(format!("Socket for {connection} was closed."))?
            .map_err(|e| e.to_string())?;

        if let Message::Text(message) = message {
            println!("Received socket message from {connection}: {message}");

            println!("Sending Nats message for {connection}: {message}");
            let message = message.into();
            nats.publish(connection.nats_subject(), message)
                .await
                .map_err(|e| e.to_string())?;
        } else if let Message::Binary(_) = message {
            return Err(format!(
                "Received binary socket message from {connection}. Closing connection..."
            ));
        } else if let Message::Close(_) = message {
            return Err(format!(
                "Socket closed by {connection}! Cleaning up connection..."
            ));
        } else {
            println!("Received socket message {:?} from {connection}", {
                message
            });
        }
    }
}
