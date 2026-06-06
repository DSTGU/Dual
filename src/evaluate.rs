use crate::morph::pattern::{DATABASE};
use crate::types::board::BoardPosition;

pub fn pattern_evaluate(board_position: &BoardPosition) -> i32 {
    let db = DATABASE.read().unwrap();
    let eval = db.db.evaluate(board_position);

    // board_position.print_board();
    // println!("Position patterns: {:?}", board_position.extract_patterns());
    // println!("Evaluated to: {} wdl, {} cp", eval, (eval * 1000.0 - 500.0) as i32);
    (eval * 1000.0 - 500.0) as i32 + 15 // convert to cp, stm bonus
}
