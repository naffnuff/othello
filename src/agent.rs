use std::sync::mpsc;
use rand::Rng;
use num_enum::TryFromPrimitive;

use crate::common::CellList;
use crate::board::Player;
use crate::board::Board;
use crate::board::Cell;
use crate::referee::Referee;

type Move = (usize, usize);

// Message-passing types
pub struct MoveRequest {

    pub board: Board,
    pub player: Player,
    pub pace_response: bool,
    pub algorithm_choice: AiType,
    pub recursion_depth: usize,
}

pub struct MoveResult {

    pub board: Board,
    pub player: Player,
    pub next_move: Move,
}

#[derive(Debug, Clone, Copy, PartialEq, TryFromPrimitive)]
#[repr(usize)]
pub enum AiType {
    Random,
    Minimax,
}

pub struct Agent {
    rng: rand::prelude::ThreadRng,
    move_request_receiver: mpsc::Receiver<MoveRequest>,
    move_result_sender: mpsc::Sender<MoveResult>,
    valid_moves: CellList,
    referee: Referee,
}

impl Agent {

    pub fn new(move_request_receiver: mpsc::Receiver<MoveRequest>, move_result_sender: mpsc::Sender<MoveResult>) -> Self {

        Agent {
            rng: rand::rng(),
            move_request_receiver: move_request_receiver,
            move_result_sender: move_result_sender,
            valid_moves: CellList::default(),
            referee: Referee::default(),
        }
    }

    pub fn run(&mut self) {

        while let Ok(move_request) = self.move_request_receiver.recv() {

            println!("Received move request, ai type: {:?}", move_request.algorithm_choice);

            let next_move = match move_request.algorithm_choice {
                AiType::Random => self.find_random_move(&move_request.board, move_request.player),
                AiType::Minimax => self.find_best_move_using_minimax(&move_request.board, move_request.player, move_request.recursion_depth),
            };

            if move_request.pace_response {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            
            self.move_result_sender.send(MoveResult { board: move_request.board, player: move_request.player, next_move }).unwrap();
            
            println!("Responded with move {:?}", next_move);
        }
    }

    // returns a random valid move
    fn find_random_move(&mut self, board: &Board, player: Player) -> Move {

        if self.referee.find_all_valid_moves(board, player, &mut self.valid_moves) {

            self.valid_moves.list[self.rng.random_range(..self.valid_moves.count)]

        } else {

            (Board::SIZE, Board::SIZE)
        }
    }

    // uses an algorithm that will try to find a move that maximizes oneself and minimizes the opponent
    fn find_best_move_using_minimax(&mut self, board: &Board, player: Player, recursion_depth: usize) -> Move {

        self.find_best_move_recursive(board, player, recursion_depth)
    }

    fn find_best_move_recursive(&mut self, board: &Board, player: Player, recursion_depth: usize) -> Move {

        if recursion_depth == 0 {
            return (Board::SIZE, Board::SIZE);
        }

        let mut row = 0;
        let mut col = 0;
        while row < Board::SIZE {
            while col < Board::SIZE {

                let mut new_board = board.clone();
                new_board.grid[row][col] = Cell::Taken(player);
                (row, col) = self.referee.find_and_apply_next_valid_move(&mut new_board, player, (row, col));
                let board_value = self.evaluate_board(&new_board, player);



                col += 1;
            }
            
            row += 1;
            col = 0;
        }

        (Board::SIZE, Board::SIZE)
    }

    fn evaluate_board(&mut self, board: &Board, player: Player) -> f32 {

        0.0
    }
}