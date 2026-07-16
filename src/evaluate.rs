use crate::types::board::BoardPosition;
use crate::types::search_state::SearchState;

pub fn nnue_evaluate(board_position: &BoardPosition, search_state: &SearchState) -> i32 {
    search_state.network_state.evaluate(board_position.side)
}

// pub fn evaltest(board_position: &BoardPosition) {

//     println!("NNUE: {}", nnue_evaluate(board_position));

// }

