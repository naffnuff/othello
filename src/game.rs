use std::thread;
use std::sync::mpsc;
use std::time::Duration;
use std::time::Instant;

use eframe::egui;

use crate::common::CellList;
use crate::common::MoveRequest;
use crate::board::Player;
use crate::board::Cell;
use crate::board::Board;
use crate::agent::Agent;
use crate::referee::Outcome;
use crate::referee::Referee;

#[derive(Clone, Copy)]
enum Phase {
    Turn(Player),
    Win(Player),
    Tie,
}

pub struct Game {
    board: Board,
    current_phase: Phase,
    black_ai_enabled: bool,
    white_ai_enabled: bool,
    show_effects_of_move: bool,
    show_valid_moves: bool,
    awaiting_ai_move: bool,
    ai_thread: Option<thread::JoinHandle<()>>,
    move_request_sender: Option<mpsc::Sender<MoveRequest>>,
    move_result_receiver: mpsc::Receiver<(usize, usize)>,
    referee: Referee,
    valid_moves: CellList,
    flip_cells: CellList,
    black_ai_type: usize,
    white_ai_type: usize,
    auto_restart: bool,
    pace_ai: bool,
    pause_at_win: bool,
    scheduled_restart: Instant,
}

impl Default for Game {
    fn default() -> Self {

        let (move_request_sender, move_request_receiver) = mpsc::channel::<MoveRequest>();
        let (move_result_sender, move_result_receiver) = mpsc::channel::<(usize, usize)>();
        
        let ai_thread = thread::spawn(move || {
            
            let mut agent = Agent::new(move_request_receiver, move_result_sender);
            agent.run();
        });

        let mut game = Game {
            board: Board::default(),
            current_phase: Phase::Turn(Player::Black),
            black_ai_enabled: false,
            white_ai_enabled: false,
            show_effects_of_move: false,
            show_valid_moves: false,
            awaiting_ai_move: false,
            ai_thread: Some(ai_thread),
            move_request_sender: Some(move_request_sender),
            move_result_receiver: move_result_receiver,
            referee: Referee::default(),
            valid_moves: CellList::default(),
            flip_cells: CellList::default(),
            black_ai_type: 0,
            white_ai_type: 0,
            auto_restart: false,
            pace_ai: true,
            pause_at_win: true,
            scheduled_restart: Instant::now(),
        };

        game.reset();

        game
    }
}

impl Drop for Game {
    fn drop(&mut self) {

        println!("Game is being dropped. Cleaning up AI thread...");

        // Drop the sender so AI thread exits
        self.move_request_sender = None;

        // Wait for AI thread to exit
        if let Some(ai_thread) = self.ai_thread.take() {
            let _ = ai_thread.join();
            println!("...and joined AI thread");
        }
    }
}

impl Game {
    fn reset(&mut self) {
        self.board = Board::default();
        self.current_phase = Phase::Turn(Player::Black);
        self.referee.find_all_valid_moves(&self.board, Player::Black, &mut self.valid_moves);
    }

    // always call this from the UI thread to avoid nasty race conditions
    fn make_move(&mut self, (row, col): (usize, usize), player: Player) -> bool {

        // Validate and collect flip cells for ai move
        if self.referee.find_flip_cells_for_move(&self.board, player, (row, col), &mut self.flip_cells) {
            
            // Place the current player's piece
            self.board.grid[row][col] = Cell::Taken(player);
            
            // flip cells
            for (flip_row, flip_col) in self.flip_cells.iter() {
                
                self.board.grid[flip_row][flip_col] = Cell::Taken(player);
            }

            let other_player = match player {
                Player::Black => Player::White,
                Player::White => Player::Black
            };

            if self.referee.find_all_valid_moves(&self.board, other_player, &mut self.valid_moves) {

                // switch players if the other player has valid moves
                self.current_phase = Phase::Turn(other_player);

            } else if !self.referee.find_all_valid_moves(&self.board, player, &mut self.valid_moves) {

                // no player has any valid moves, game ends
                self.current_phase = match Referee::check_outcome(&self.board) {
                    Outcome::Won(player) => Phase::Win(player),
                    Outcome::Tie => Phase::Tie,
                };

                // only used if auto_restart is enabled
                self.scheduled_restart = Instant::now();
                if self.pause_at_win {
                    self.scheduled_restart += Duration::from_secs(1);
                }
            }

            true

        } else {

            false
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {

            // UI drawing

            let rect = ui.available_rect_before_wrap();
            let square_size = rect.width().min(rect.height()) / Board::SIZE as f32;
            let line_width = square_size * 0.01;

            let to_color = |player| match player {
                
                Player::Black => egui::Color32::BLACK,
                Player::White => egui::Color32::WHITE,
            };

            let get_square_rect = |row, col| {

                let square_pos = egui::Pos2 { x: rect.left() + col as f32 * square_size, y: rect.top() + row as f32 * square_size };
                egui::Rect::from_min_size(square_pos, egui::Vec2::splat(square_size))
            };

            // draw the current board state
            for row in 0..Board::SIZE {
                for col in 0..Board::SIZE {

                    let square_rect = get_square_rect(row, col);

                    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::DARK_GREEN);

                    let stroke = egui::Stroke { width: line_width, color: egui::Color32::BLACK };
                    ui.painter().rect_stroke(square_rect, 0.0, stroke, egui::StrokeKind::Inside);

                    if let Cell::Taken(cell_state) = self.board.grid[row][col] {
                        ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93, to_color(cell_state));
                    }
                }
            }

            match (self.current_phase, self.black_ai_enabled, self.white_ai_enabled) {

                (Phase::Turn(player @ Player::Black), true, _) | (Phase::Turn(player @ Player::White), _, true) => {
                    
                    // AI moves

                    // either poll for ai response, non-blocking...
                    if self.awaiting_ai_move {

                        if let Some((row, col)) = self.move_result_receiver.try_recv().ok() {

                            if row < Board::SIZE && col < Board::SIZE {

                                assert!(self.make_move((row, col), player));

                            } else {

                                // unable to come up with a valid move, it seems
                                match player {
                                    Player::Black => self.black_ai_enabled = false,
                                    Player::White => self.white_ai_enabled = false,
                                }
                            }
                            self.awaiting_ai_move = false;
                        }

                    } else { //...or ask ai to start thinking about the next move

                        if let Some(tx) = &self.move_request_sender {

                            self.awaiting_ai_move = true;
                            let _ = tx.send(MoveRequest { board: self.board.clone(), current_player: player, pace_ai: self.pace_ai });
                        }
                    }
                }
                (Phase::Turn(player), _, _) => {

                    // Awaiting human move

                    // show a dot for every valid move for the current player
                    if self.show_valid_moves {

                        for (flip_row, flip_col) in self.valid_moves.iter() {
                            
                            let square_rect = get_square_rect(flip_row, flip_col);
                            ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93 * 0.5, to_color(player));
                        }
                    }

                    // Mouse handling
                    let mut row = Board::SIZE;
                    let mut col = Board::SIZE;

                    let mut is_valid_move = false;
                    
                    // check mouse hovering
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.latest_pos()) {
                            
                        row = ((mouse_pos.y - rect.top()) / square_size) as usize;
                        col = ((mouse_pos.x - rect.left()) / square_size) as usize;
                    
                        if row < Board::SIZE && col < Board::SIZE {

                            // this could be optimized by only doing it when the mouse changes cells
                            is_valid_move = self.referee.find_flip_cells_for_move(&self.board, player, (row, col), &mut self.flip_cells);

                            if is_valid_move {

                                // show how the hovered move would change the board
                                if self.show_effects_of_move {
                            
                                    let square_rect = get_square_rect(row, col);
                                    ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93, to_color(player));

                                    for (flip_row, flip_col) in self.flip_cells.iter() {
                                        
                                        let square_rect = get_square_rect(flip_row, flip_col);
                                        ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93 * 0.5, to_color(player));
                                    }
                                }
                            }
                        }
                    }
        
                    // handle mouse clicks to make moves
                    if ui.input(|i| i.pointer.any_down()) {
                    
                        if row < Board::SIZE && col < Board::SIZE && is_valid_move {
                        
                            assert!(self.make_move((row, col), player));
                        }
                    }
                }
                (Phase::Win(_) | Phase::Tie, _, _) => {
                    
                    if self.auto_restart && Instant::now() >= self.scheduled_restart {

                        self.reset();
                    }
                }
            }

            ctx.request_repaint();

        });

        egui::SidePanel::right("right_panel").show(ctx, move |ui| {

            // Current-status message
            let message = match (self.current_phase, self.awaiting_ai_move, self.black_ai_enabled, self.white_ai_enabled) {
                (Phase::Turn(Player::Black), true, true, _) => "Black is thinking...",
                (Phase::Turn(Player::White), true, _, true) => "White is thinking...",
                (Phase::Turn(Player::Black), _, _, _) => "Black's turn",
                (Phase::Turn(Player::White), _, _, _) => "White's turn",
                (Phase::Win(Player::Black), _, _, _) => "Black won",
                (Phase::Win(Player::White), _, _, _) => "White won",
                (Phase::Tie, _, _, _) => "Tie",
            };
            ui.label(message);
            
            // White AI type checkbox
            ui.checkbox(&mut self.black_ai_enabled, "Enable Black AI");
            let options = ["Random", "Minimax"];
            for (i, option) in options.iter().enumerate() {
                if ui.radio(self.black_ai_type == i, *option).clicked() {
                    self.black_ai_type = i;
                }
            }
            
            // White AI type checkbox
            ui.checkbox(&mut self.white_ai_enabled, "Enable White AI");
            let options = ["Random", "Minimax"];
            for (i, option) in options.iter().enumerate() {
                if ui.radio(self.white_ai_type == i, *option).clicked() {
                    self.white_ai_type = i;
                }
            }

            // Buttons and checkboxes
            if ui.button("Restart Game").clicked() {
                self.reset();
            }
            ui.checkbox(&mut self.auto_restart, "Auto Restart");
            ui.checkbox(&mut self.show_valid_moves, "Show Valid Moves");
            ui.checkbox(&mut self.show_effects_of_move, "Show Effects of Move");
            ui.checkbox(&mut self.pace_ai, "Pace AI");
            ui.checkbox(&mut self.pause_at_win, "Pause at Win");
        });
    }
}