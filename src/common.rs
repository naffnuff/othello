use crate::board::Board;
use crate::board::Player;

// Common utility types
// a cache for re-use to avoid unnecesary memory allocations
pub struct CellList {

    pub list: [(usize, usize); 64],
    pub count: usize,
}

impl Default for CellList {

    fn default() -> Self {

        CellList { list: [(Board::SIZE, Board::SIZE); 64], count: 0 }
    }
}

impl CellList {

    pub fn push_back(&mut self, cell: (usize, usize)) {

        self.list[self.count] = cell;
        self.count += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = (usize, usize)> {

        self.list[..self.count].iter().copied()
    }
}

// Message-passing types
pub struct MoveRequest {

    pub board: Board,
    pub player: Player,
    pub pace_ai: bool,
    pub ai_type: usize,
}

pub struct MoveResult {

    pub board: Board,
    pub player: Player,
    pub next_move: (usize, usize),
}