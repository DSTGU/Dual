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

        let board = search_state.make_move(i);
        if let Some(_) = board {
            movecount += perft_driver(search_state, depth - 1);
            search_state.take_back(i);
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
        
        let board = search_state.make_move(i);

        if let Some(_) = board {
            let cnt= perft_driver(search_state, depth - 1);
            search_state.take_back(i);
            println!("{:?}, Moves: {}", i, cnt);

            movecount += cnt;
        }

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