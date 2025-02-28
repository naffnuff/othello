mod board;
mod game;
mod agent;
mod common;
mod referee;

use game::Game;

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

