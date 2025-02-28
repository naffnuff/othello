use crate::board::Cell;
use crate::board::Board;
use crate::board::Player;
use crate::common::CellList;

// not thread-safe, every thread needs its own Referee
pub struct Referee {

    // a cache for the result of find_adjacent_opposites
    adjacent_opposites: CellList,

    // a cache for the result of find_flip_cells
    flip_cells: CellList,
}

impl Default for Referee {

    fn default() -> Self {

        Referee {
            adjacent_opposites: CellList::default(),
            flip_cells: CellList::default(),
        }
    }
}

impl Referee {
    
    // public
    pub fn validate_move(&mut self, board: &Board, player: Player, maybe_move: (usize, usize)) -> bool {

        if Self::find_adjacent_opposites(board, player, maybe_move, &mut self.adjacent_opposites) {

            Self::find_flip_cells(board, player, maybe_move, &self.adjacent_opposites, &mut self.flip_cells)

        } else {

            false
        }
    }

    pub fn find_flip_cells_for_move(&mut self, board: &Board, player: Player, maybe_move: (usize, usize), result: &mut CellList) -> bool {
        
        if Self::find_adjacent_opposites(board, player, maybe_move, &mut self.adjacent_opposites) {

            Self::find_flip_cells(board, player, maybe_move, &self.adjacent_opposites, result)

        } else {

            false
        }
    }

    // internal
    fn find_adjacent_opposites(board: &Board, player: Player, (row, col): (usize, usize), result: &mut CellList) -> bool {

        let start_row = match row {
            0 => 0,
            other => other - 1
        };
        let end_row = match row + 1 {
            Board::SIZE => Board::SIZE,
            other => other + 1
        };
        let start_col = match col {
            0 => 0,
            other => other - 1
        };
        let end_col = match col + 1 {
            Board::SIZE => Board::SIZE,
            other => other + 1
        };

        result.count = 0;

        for other_row in start_row..end_row {
            for other_col in start_col..end_col {
                if other_row != row || other_col != col {
                    if let Cell::Taken(other_disk) = board.grid[other_row][other_col] {
                        if other_disk != player {
                            result.push_back((other_row, other_col));
                        }
                    }
                }
            }
        };

        result.count != 0
    }

    // expects result to already be filled with adjacent opposites
    fn find_flip_cells(board: &Board, player: Player, (row, col): (usize, usize), adjacent_opposites: & CellList, result: &mut CellList) -> bool {

        result.count = 0;

        for (adjacent_row, adjacent_col) in adjacent_opposites.iter() {

            let direction = (adjacent_row as i32 - row as i32, adjacent_col  as i32 - col as i32);
            Self::cast_ray(board, player, (adjacent_row, adjacent_col), direction, result);
        }

        result.count != 0
    }

    fn cast_ray(board: &Board, player: Player, (row, col): (usize, usize), (row_direction, col_direction): (i32, i32), result: &mut CellList) -> bool {
        
        match board.grid[row][col] {
            Cell::Empty => false,
            Cell::Taken(color) if color == player => {
                result.push_back((row, col));
                true
            },
            Cell::Taken(_) => {
                let new_row = row as i32 + row_direction;
                let new_col = col as i32 + col_direction;
                if new_row < 0 || new_row >= Board::SIZE as i32 || new_col < 0 || new_col >= Board::SIZE as i32 {
                    false
                } else {
                    if Self::cast_ray(board, player, (new_row as usize, new_col as usize), (row_direction, col_direction), result) {
                        result.push_back((row, col));
                        true
                    } else {
                        false
                    }
                }
            }
        }
    }

}