use std::sync::mpsc;
use rand::Rng;

use crate::common::CellList;
use crate::common::MoveRequest;
use crate::board::Player;
use crate::board::Board;
use crate::referee::MatchState;
use crate::referee::Referee;

pub struct Agent {
    rng: rand::prelude::ThreadRng,
    move_request_receiver: mpsc::Receiver<MoveRequest>,
    move_result_sender: mpsc::Sender<(usize, usize)>,
    valid_moves: CellList,
    referee: Referee,
}

impl Agent {

    pub fn new(move_request_receiver: mpsc::Receiver<MoveRequest>, move_result_sender: mpsc::Sender<(usize, usize)>) -> Self {

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
            std::thread::sleep(std::time::Duration::from_secs(1));
            let next_move = self.find_next_move(&move_request.board, move_request.current_player);
            self.move_result_sender.send(next_move).unwrap();
        }
    }

    pub fn find_next_move(&mut self, board: &Board, player: Player) -> (usize, usize) {

        match Referee::check_match_state(board) {
            MatchState::Ongoing => {

                if self.referee.find_all_valid_moves(board, player, &mut self.valid_moves) {

                    self.valid_moves.list[self.rng.random_range(..self.valid_moves.count)]

                } else {

                    (Board::SIZE, Board::SIZE)
                }
            }
            _ => (Board::SIZE, Board::SIZE)

        }
    }
}