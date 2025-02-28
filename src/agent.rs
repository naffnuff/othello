use std::sync::mpsc;
use rand::Rng;

use crate::common::MoveRequest;
use crate::board::Player;
use crate::board::Cell;
use crate::board::Board;
use crate::referee::Referee;

const SIZE: usize = 8;

// a cache for re-use to avoid unnecesary memory allocations
pub struct CellList {
    pub list: [(usize, usize); 64],
    pub count: usize,
}

impl CellList {

    pub fn push_back(&mut self, cell: (usize, usize)) {
        self.list[self.count] = cell;
        self.count += 1;
    }

    pub fn iter(&self) -> impl Iterator<Item = &(usize, usize)> {
        self.list[..self.count].iter()
    }
}

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
            valid_moves: CellList { list: [(Board::SIZE, Board::SIZE); 64], count: 0 },
            referee: Referee::default(),
        }
    }

    pub fn run(&mut self) {

        while let Ok(move_request) = self.move_request_receiver.recv() {
            println!("AI thread: Received new request...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("AI thread: 1 sec passed...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("AI thread: 2 sec passed...");
            std::thread::sleep(std::time::Duration::from_secs(1));
            println!("AI thread: 3 sec passed...");
            let next_move = self.make_next_move(&move_request.board, move_request.current_player);
            println!("AI thread: Done processing request");
            self.move_result_sender.send(next_move).unwrap();
            println!("AI thread: Message sent");
        }
        println!("AI thread: exiting...");
    }

    pub fn make_next_move(&mut self, board: &Board, player: Player) -> (usize, usize) {

        self.valid_moves.count = 0;

        let mut row;
        let mut col;

        loop {

            row = self.rng.random_range(0..SIZE);
            col = self.rng.random_range(0..SIZE);

            if board.grid[row][col] == Cell::Empty && self.referee.validate_move(board, player, (row, col)) {
                break
            }

        }
        
        (row, col)
    }
}