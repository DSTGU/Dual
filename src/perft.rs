use std::time::SystemTime;
use crate::move_gen::{generate_moves, make_move};
use crate::search_state::{self, SearchState};
use crate::shared::{BoardPosition, MoveDirection, print_board};

pub fn perft_driver(search_state: &mut SearchState, depth: usize) -> usize {
    
    if depth == 0 {
        return 1;
    }
    
    let movelist = generate_moves(&search_state.get_board_position());
    
    let mut movecount = 0;

    for i in movelist {

        let og_board = search_state.get_board_position();
        let board = make_move(&search_state.get_board_position(),&i, MoveDirection::Move);

        if let Some(board) = board {
            search_state.make_move_for_state(board);
            movecount += perft_driver(search_state, depth - 1);
            search_state.take_back_for_state(og_board);
        }
    }
    movecount

}


pub fn perft(search_state: &mut SearchState, depth: usize) {
    
    print_board(&search_state.get_board_position());
    
    if depth == 0 {
        return;
    }
    let now = SystemTime::now();
    let movelist = generate_moves(&search_state.get_board_position());

    let mut movecount = 0;

    for i in movelist {
        
        let og_board = search_state.get_board_position();
        let board = make_move(&search_state.get_board_position(),&i, MoveDirection::Move);

        let mut cnt = 0;
        if let Some(board) = board {
            search_state.make_move_for_state(board);
            cnt = perft_driver(search_state, depth - 1);
            search_state.take_back_for_state(og_board);
            println!("{:?}, Moves: {}", i, cnt);

        }
        movecount += cnt;

    }

    match now.elapsed() {
        Ok(elapsed) => {
            println!("Perft Time {} ms", elapsed.as_millis());
        }
        Err(e) => {
            // an error occurred!
            println!("Error: {e:?}");
        }
    }
    println!("Moves: {}", movecount);
    
}