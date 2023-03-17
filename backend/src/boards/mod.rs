use mongodb::bson::Uuid;
use serde::Deserialize;
use serde::Serialize;
use std::fmt::Debug;

pub mod get_boards_id;

#[derive(Serialize, Deserialize, Debug)]
pub struct Board {
    pub id: Uuid,
    pub url: String,
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
