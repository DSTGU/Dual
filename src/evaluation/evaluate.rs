use crate::primitives::board::BoardPosition;
use crate::search_objs::search_state::SearchState;

pub fn nnue_evaluate(board_position: &BoardPosition, search_state: &SearchState) -> i32 {
    search_state.network_state.evaluate(board_position.side)
}

pub fn evaltest(board_position: &BoardPosition, search_state: &SearchState) {
    println!("NNUE: {}", nnue_evaluate(board_position, search_state));
}

#[cfg(test)]
mod tests {
    use std::thread;
    use crate::evaluation::evaluate::nnue_evaluate;
    use crate::gui::parse_position_command;
    use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::SearchState;
    use crate::primitives::shared::{Move, MoveCode};


    #[test]
    fn test_undoing() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {

        let command = "position startpos";
        let mut search_state = SearchState::new(&EngineConfig::thin());
        let board_position = parse_position_command(&mut search_state, command);
        let mv = Move::create(62 , 53 , MoveCode::QuietMove); // Nf3
        let board_after_move = board_position.make_move(mv).unwrap();

        let eval1 = nnue_evaluate(&board_position, &search_state);
        search_state.make_move(mv, &board_position);
        let eval2 = nnue_evaluate(&board_after_move, &search_state);
        search_state.take_back();
        let eval3 = nnue_evaluate(&board_position, &search_state);
        search_state.make_move(mv, &board_position);
        let eval4 = nnue_evaluate(&board_after_move, &search_state);
        println!("{} - {} - {} - {}", eval1, eval2, eval3, eval4);

        assert_eq!(eval1, eval3);
        assert_eq!(eval2, eval4);
                })
            .unwrap();
        handler.join().unwrap();
    }
}