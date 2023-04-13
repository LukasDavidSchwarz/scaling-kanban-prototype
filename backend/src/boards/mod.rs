use chrono::{DateTime, Utc};
use mongodb::bson::Uuid;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

pub mod get_boards;
pub mod get_boards_id;
pub mod post_boards;
pub mod put_boards_id;

pub type BoardId = Uuid;

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub id: BoardId,
    pub version: u64,
    // TODO: Differentiate between JSON and BSON serialization to avoid sending bson to frontend
    #[serde(with = "mongodb::bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub lists: Vec<TaskList>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TaskList {
    pub id: Uuid,
    pub name: String,
    pub tasks: Vec<Task>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Task {
    pub id: Uuid,
    pub name: String,
}

impl Board {
    pub fn new(name: impl ToString, lists: Vec<TaskList>) -> Self {
        Board {
            id: Uuid::new(),
            version: 0,
            created_at: Utc::now(),
            name: name.to_string(),
            lists,
        }
    }

    pub fn pubsub_subject(&self) -> String {
        pubsub_subject(&self.id)
    }
}

impl TaskList {
    pub fn new(name: impl ToString, tasks: Vec<Task>) -> Self {
        TaskList {
            id: Uuid::new(),
            name: name.to_string(),
            tasks,
        }
    }
}

impl Task {
    pub fn new(name: impl ToString) -> Self {
        Task {
            id: Uuid::new(),
            name: name.to_string(),
        }
    }
}

pub fn pubsub_subject(board_id: &BoardId) -> String {
    format!("board.{}", board_id.clone())
}
