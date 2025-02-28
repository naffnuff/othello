#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Player {
    Black,
    White
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cell {
    Empty,
    Taken(Player),
}

#[derive(Clone, Debug)]
pub struct Board {
    pub state: [[Cell; Board::SIZE]; Board::SIZE],
}

impl Board {
    pub const SIZE: usize = 8;
}

impl Default for Board {
    fn default() -> Self {

        let mut state = [[Cell::Empty; Board::SIZE]; Board::SIZE];
        state[3][3] = Cell::Taken(Player::White);
        state[4][4] = Cell::Taken(Player::White);
        state[3][4] = Cell::Taken(Player::Black);
        state[4][3] = Cell::Taken(Player::Black);

        Board {
            state,
        }
    }
}