use std::time::SystemTime;
use crate::move_gen::{generate_moves};
use crate::search_state::{SearchState};
use crate::shared::{MoveSuccess};

pub fn perft_driver(search_state: &mut SearchState, depth: usize) -> usize {

    if depth == 0 {
        return 1;
    }

    //print_board(&search_state.board_position);
    let movelist = generate_moves(&search_state.board_position);
    
    let mut movecount = 0;
    
    for i in movelist {
        //println!("|{}{:?}", "-".to_string().repeat(4-depth), i);
        //search_state.board_position.print_board();
        let result = search_state.make_move(i);
        
        if result == MoveSuccess::Success {
            movecount += perft_driver(search_state, depth - 1);
            search_state.take_back(i);
        }
    }

    movecount

}


pub fn perft(search_state: &mut SearchState, depth: usize) {
    
    search_state.board_position.print_board();
    
    if depth == 0 {
        return;
    }
    let now = SystemTime::now();
    let movelist = generate_moves(&search_state.board_position);

    let mut movecount = 0;

    for i in movelist {
        
        let result = search_state.make_move(i);

        if result == MoveSuccess::Success {
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