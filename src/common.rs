use crate::board::Board;
use crate::board::Player;

pub struct MoveRequest {
    pub board: Board,
    pub current_player: Player,
}