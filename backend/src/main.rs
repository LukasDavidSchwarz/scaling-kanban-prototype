use mongodb::{options::ClientOptions, Client};
use std::env;
use std::time::Duration;

pub struct AppState {
    db: Client,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Hello, world!");

    let db_uri =
        env::var("DB_CONNECTION_URI").expect("Environment variable ´DB_CONNECTION_URI´ not set!");

    let mut client_options = ClientOptions::parse(db_uri)
        .await
        .expect("Failed to parse environment variable ´DB_CONNECTION_URI´");
    client_options.app_name = Some("kanban backend".to_string());
    client_options.connect_timeout = Some(Duration::from_secs(10));
    let db = Client::with_options(client_options)?;

    println!("Database names:");
    for db_name in db.list_database_names(None, None).await? {
        println!("{}", db_name);
    }

    println!("Backend finished executing.");
    Ok(())
}
