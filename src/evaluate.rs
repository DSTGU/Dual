use crate::morph::pattern::{DATABASE};
use crate::types::board::BoardPosition;

pub fn pattern_evaluate(board_position: &BoardPosition) -> i32 {
    let db = DATABASE.read().unwrap();
    let eval = db.db.evaluate(board_position);
    //println!("Evaluated to: {} wdl, {} cp", eval, (eval * 1000.0 - 500.0) as i32);
    (eval * 1000.0 - 500.0) as i32 // convert to cp
}
