use mongodb::bson::Uuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

pub mod get_boards_id;
pub mod put_boards_id;

pub type BoardId = Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub id: BoardId,
    pub url: String,
    pub version: u64,
    pub name: String,
    pub lists: Vec<TaskList>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskList {
    pub id: Uuid,
    pub name: String,
    pub tasks: Vec<Card>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Card {
    pub id: Uuid,
    pub name: String,
    pub is_done: bool,
}

impl Board {
    pub fn pubsub_subject(&self) -> String {
        pubsub_subject(&self.id)
    }
}

pub fn pubsub_subject(board_id: &BoardId) -> String {
    format!("board.{}", board_id.clone())
}
