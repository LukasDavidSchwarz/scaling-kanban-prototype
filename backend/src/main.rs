use mongodb::{options::ClientOptions, Client};

pub struct AppState {
    db: Client,
}

fn main() {
    println!("Hello, world!");
}
