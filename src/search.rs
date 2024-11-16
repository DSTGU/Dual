use crate::moveGen::generate_moves;
use crate::shared::{moveToAlg, BoardPosition};

pub fn rand_search(board_position: &BoardPosition) {

    let mut moves = generate_moves(board_position);
    
    let mut mv = moves.pop();
    
    while mv.is_none() {
        mv = moves.pop();
    }
    
    println!("bestmove {}", moveToAlg(&mv.unwrap()))
}