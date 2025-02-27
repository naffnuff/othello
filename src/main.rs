use eframe::egui;

const BOARD_SIZE: usize = 8;
const EMPTY: u8 = 0;
const BLACK: u8 = 1;
const WHITE: u8 = 2;

struct Game {
    board: [[u8; BOARD_SIZE]; BOARD_SIZE],
    current_player: u8,
}

impl Default for Game {
    fn default() -> Self {
        let mut board = [[EMPTY; BOARD_SIZE]; BOARD_SIZE];
        board[3][3] = WHITE;
        board[4][4] = WHITE;
        board[3][4] = BLACK;
        board[4][3] = BLACK;

        Game {
            board,
            current_player: BLACK,
        }
    }
}

impl eframe::App for Game {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {

        egui::CentralPanel::default().show(ctx, |ui| {

            // Draw the Othello board
            let rect = ui.available_rect_before_wrap();
            let square_size = rect.width().min(rect.height()) / BOARD_SIZE as f32;
            let line_width = square_size * 0.01;

            let to_color = |player| match player {
                BLACK => egui::Color32::BLACK,
                WHITE => egui::Color32::WHITE,
                _ => egui::Color32::RED,
            };

            let get_square_rect = |row, col| {
                let square_pos = egui::Pos2 { x: rect.left() + col as f32 * square_size, y: rect.top() + row as f32 * square_size };
                egui::Rect::from_min_size(square_pos, egui::Vec2::splat(square_size))
            };

            for row in 0..BOARD_SIZE {
                for col in 0..BOARD_SIZE {

                    let square_rect = get_square_rect(row, col);

                    ui.painter().rect_filled(square_rect, 0.0, egui::Color32::DARK_GREEN);

                    let stroke = egui::Stroke { width: line_width, color: egui::Color32::BLACK };
                    ui.painter().rect_stroke(square_rect, 0.0, stroke, egui::StrokeKind::Inside);

                    let cell_state = self.board[row][col];
                    
                    match cell_state {
                        EMPTY => {}
                        _ => {
                            ui.painter().circle_filled(square_rect.center(), square_size / 2.0 * 0.93, to_color(cell_state));
                        }
                    }
                }
            }

            // Mouse handling
            let mut row = BOARD_SIZE;
            let mut col = BOARD_SIZE;
            
            if let Some(mouse_pos) = ui.input(|i| i.pointer.latest_pos()) {
                    
                row = ((mouse_pos.y - rect.top()) / square_size) as usize;
                col = ((mouse_pos.x - rect.left()) / square_size) as usize;
            
                if row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col] == EMPTY {
                    
                    let stroke = egui::Stroke { width: line_width, color: to_color(self.current_player) };
                    ui.painter().circle_stroke(get_square_rect(row, col).center(), square_size / 2.0 * 0.9, stroke);
                }
            }

            // Handle mouse clicks to make moves
            if ui.input(|i| i.pointer.any_down()) {
            
                if row < BOARD_SIZE && col < BOARD_SIZE && self.board[row][col] == EMPTY {
                    // Place the current player's piece
                    self.board[row][col] = self.current_player;
                    // Switch players
                    self.current_player = if self.current_player == BLACK { WHITE } else { BLACK };
                }
            }
        });
    }
}

fn main() {
    let app = Game::default();
    let _ = eframe::run_native(
        "Othello Game",
        eframe::NativeOptions {
            ..Default::default()
        },
        Box::new(|_cc| Ok(Box::new(app))),
    );
}

