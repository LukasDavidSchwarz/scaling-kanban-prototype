use std::fmt::{Debug, Display, Formatter};
use std::net::SocketAddr;

#[derive(Clone, Copy, Debug)]
pub struct ConnectionInfo {
    pub address: SocketAddr,
    pub board_id: u128,
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
