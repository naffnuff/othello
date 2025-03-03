use std::sync::mpsc;
use rand::Rng;
use num_enum::TryFromPrimitive;

use crate::common::CellList;
use crate::board::Player;
use crate::board::Board;
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

            let next_move = match move_request.algorithm_choice {
                AiType::Random => self.find_random_move(&move_request.board, move_request.player),
                AiType::Minimax => self.find_best_move_using_minimax(&move_request.board, move_request.player, move_request.recursion_depth),
            };

            if move_request.pace_response {
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
            
            self.move_result_sender.send(MoveResult { board: move_request.board, player: move_request.player, next_move }).unwrap();
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

        let mut allocation_count = 0;
        let (optimal_move, _optimal_score) = self.find_best_move_recursive(board, player, recursion_depth, &mut allocation_count);
        optimal_move
    }

    // returns (the optimal move, evaluation score given to that move)
    // TODO: add alpha-beta pruning
    // TODO: it's silly to think very hard about the first few moves
    fn find_best_move_recursive(&mut self, board: &Board, player: Player, recursion_depth: usize, allocation_count: &mut i32) -> (Move, f32) {
        let mut optimal_move = (Board::SIZE, Board::SIZE);
        let mut optimal_score = f32::NEG_INFINITY;
        let mut selection_count = 0; // Track number of equally good moves found

        let mut row = 0;
        let mut col = 0;
        while row < Board::SIZE {
            while col < Board::SIZE {

                let mut new_board = board.clone();

                (row, col) = self.referee.find_and_apply_next_valid_move(&mut new_board, player, (row, col));

                if row < Board::SIZE && col < Board::SIZE {

                    *allocation_count += 1;

                    // the evaluated score of this potential move is...
                    let board_score = 
                        // ...(depending on how far we want to think into the future)...
                        if recursion_depth == 1 {

                            // ...either how good it would make the board for us now...
                            self.evaluate_board(&new_board, player)

                        } else {

                            // ...or how good the board will become if the opponent makes their best move next...
                            let (_opponent_move, opponent_score) = self.find_best_move_recursive(&new_board, player.opponent(), recursion_depth - 1, allocation_count);

                            // ...and since this is a symmetric, zero-sum game,
                            // how good it is for us is the inverse of how good it is for them
                            -opponent_score
                        };

                    if optimal_move == (Board::SIZE, Board::SIZE) {

                        // any move is better than no move
                        optimal_score = board_score;
                        optimal_move = (row, col);

                        selection_count = 1;

                    } else if board_score == optimal_score {

                        // online reservoir sampling ensures equally good moves have equal chance of getting picked
                        selection_count += 1;
                        let replacement_probability = 1.0 / selection_count as f64;
                        if self.rng.random_bool(replacement_probability) {

                            optimal_score = board_score;
                            optimal_move = (row, col);
                        }

                    } else if board_score > optimal_score {

                        // this is for sure the best move so far
                        optimal_score = board_score;
                        optimal_move = (row, col);
                        
                        selection_count = 1;
                    }
                }

                col += 1;
            }
            
            row += 1;
            col = 0;
        }

        (optimal_move, optimal_score)
    }

    // for now, the evaluation is only based on the number of pieces
    // TODO: add heuristics, such as strong positions
    // TODO: add end-of-game awareness
    fn evaluate_board(&mut self, board: &Board, player: Player) -> f32 {

        let (player_count, opponent_count) = Referee::count_disks(board, player);
        let score = player_count as f32 - opponent_count as f32;

        score
    }
}