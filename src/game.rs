use std::thread;
use std::sync::mpsc;

use eframe::egui;

use crate::common::CellList;
use crate::common::MoveRequest;
use crate::board::Player;
use crate::board::Cell;
use crate::board::Board;
use crate::agent::Agent;
use crate::referee::Referee;

pub struct Game {
    board: Board,
    current_player: Player,
    black_ai_enabled: bool,
    white_ai_enabled: bool,
    help_enabled: bool,
    awaiting_ai_move: bool,
    ai_thread: Option<thread::JoinHandle<()>>,
    move_request_sender: Option<mpsc::Sender<MoveRequest>>,
    move_result_receiver: mpsc::Receiver<(usize, usize)>,
    referee: Referee,
    flip_cells: CellList,
}

impl Default for Game {
    fn default() -> Self {

        let (move_request_sender, move_request_receiver) = mpsc::channel::<MoveRequest>();
        let (move_result_sender, move_result_receiver) = mpsc::channel::<(usize, usize)>();
        
        let ai_thread = thread::spawn(move || {
            
            let mut agent = Agent::new(move_request_receiver, move_result_sender);
            agent.run();
        });

        Game {
            board: Board::default(),
            current_player: Player::Black,
            black_ai_enabled: false,
            white_ai_enabled: false,
            help_enabled: false,
            awaiting_ai_move: false,
            ai_thread: Some(ai_thread),
            move_request_sender: Some(move_request_sender),
            move_result_receiver: move_result_receiver,
            referee: Referee::default(),
            flip_cells: CellList::default(),
        }
    }
}

impl Drop for Game {
    fn drop(&mut self) {

        println!("OthelloApp is being dropped. Cleaning up AI thread...");
        // Drop the sender so AI thread exits
        self.move_request_sender = None;

        // Wait for AI thread to exit
        if let Some(ai_thread) = self.ai_thread.take() {
            let _ = ai_thread.join();
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {

            // Draw the Othello board
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


            match (self.current_player, self.black_ai_enabled, self.white_ai_enabled) {
                (Player::Black, true, _) | (Player::White, _, true) => {
                    // AI move
                    if self.awaiting_ai_move {
                        if let Some((row, col)) = self.move_result_receiver.try_recv().ok() {
                            println!("UI: Received AI move: {row}, {col}");
                            // Place the current player's piece
                            self.board.grid[row][col] = Cell::Taken(self.current_player);
                            
                            // Validate and collect flip cells for ai move
                            if self.referee.find_flip_cells_for_move(&self.board, self.current_player, (row, col), &mut self.flip_cells) {
                                
                                // flip cells
                                for (flip_row, flip_col) in self.flip_cells.iter() {
                                    
                                    self.board.grid[flip_row][flip_col] = Cell::Taken(self.current_player);
                                }
                                // Switch players
                                self.current_player = if self.current_player == Player::Black { Player::White } else { Player::Black };
                            }
                            self.awaiting_ai_move = false;
                        }
                    } else {
                        if let Some(tx) = &self.move_request_sender {
                            self.awaiting_ai_move = true;
                            let _ = tx.send(MoveRequest { board: self.board.clone(), current_player: self.current_player });
                        }
                    }
                }
                _ => {
                    // Awaiting human move

                    // Mouse handling
                    let mut row = Board::SIZE;
                    let mut col = Board::SIZE;

                    let mut is_valid_move = false;
                    
                    if let Some(mouse_pos) = ui.input(|i| i.pointer.latest_pos()) {
                            
                        row = ((mouse_pos.y - rect.top()) / square_size) as usize;
                        col = ((mouse_pos.x - rect.left()) / square_size) as usize;
                    
                        if row < Board::SIZE && col < Board::SIZE {

                            is_valid_move = self.referee.find_flip_cells_for_move(&self.board, self.current_player, (row, col), &mut self.flip_cells);

                            if is_valid_move {

                                if self.help_enabled {
                            
                                    let square_rect = get_square_rect(row, col);
                                    ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93 * 0.8, to_color(self.current_player));

                                    for (flip_row, flip_col) in self.flip_cells.iter() {
                                        
                                        let square_rect = get_square_rect(flip_row, flip_col);
                                        ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93 * 0.8, to_color(self.current_player));
                                    }
                                }
                            }
                        }
                    }
        
                    // Handle mouse clicks to make moves
                    if ui.input(|i| i.pointer.any_down()) {
                    
                        if row < Board::SIZE && col < Board::SIZE && is_valid_move {

                            // Place the current player's piece
                            self.board.grid[row][col] = Cell::Taken(self.current_player);
                            
                            // flip cells
                            for (flip_row, flip_col) in self.flip_cells.iter() {
                                
                                self.board.grid[flip_row][flip_col] = Cell::Taken(self.current_player);
                            }
                                    
                            // Switch players
                            self.current_player = if self.current_player == Player::Black { Player::White } else { Player::Black };
                        }
                    }
                }
            }

            ctx.request_repaint();

        });

        egui::SidePanel::right("right_panel").show(ctx, move |ui| {

            let message = match (self.current_player, self.awaiting_ai_move, self.black_ai_enabled, self.white_ai_enabled) {
                (Player::Black, true, true, _) => "Black is thinking...",
                (Player::White, true, _, true) => "White is thinking...",
                (Player::Black, _, _, _) => "Black's turn",
                (Player::White, _, _, _) => "White's turn"
            };
            ui.label(message);  // Display the message
            
            // UI controls
            ui.checkbox(&mut self.black_ai_enabled, "Enable Black AI");
            
            // UI controls
            ui.checkbox(&mut self.white_ai_enabled, "Enable White AI");
            
            // UI controls
            ui.checkbox(&mut self.help_enabled, "Enable Help");

            if ui.button("Restart Game").clicked() {
                *self = Game::default();  // Reset the game
            }
        });
    }
}