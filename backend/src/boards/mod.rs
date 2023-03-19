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
}

impl Board {
    pub fn pubsub_subject(&self) -> String {
        pubsub_subject(&self.id)
    }
}

impl Default for Board {
    fn default() -> Self {
        let card = Card {
            id: Uuid::new(),
            name: "Buy apples".to_string(),
        };

        let list = TaskList {
            id: Uuid::new(),
            name: "Grocery list".to_string(),
            tasks: vec![card],
        };

        Board {
            id: Uuid::new(),
            version: 0,
            url: "shopping-list".to_string(),
            name: "Shopping list".to_string(),
            lists: vec![list],
        }
    }
}

pub fn pubsub_subject(board_id: &BoardId) -> String {
    format!("board.{}", board_id.clone())
}
