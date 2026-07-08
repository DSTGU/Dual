use std::time::SystemTime;
use crate::move_gen::{generate_moves};
use crate::types::search_state::{SearchState};
use crate::types::shared::{MoveSuccess};

pub fn perft_driver(search_state: &mut SearchState, depth: usize) -> usize {

    if depth == 0 {
        return 1;
    }

    //print_board(&search_state.board_position);
    let movelist = generate_moves(&search_state.board_position, false);
    
    let mut movecount = 0;
    
    for i in movelist {

        let result = search_state.board_position.make_move(i);

        if result == MoveSuccess::Success {
            let old_hash = search_state.board_position.hash;
            movecount += perft_driver(search_state, depth - 1);
            search_state.board_position.take_back(i, old_hash);
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
    let movelist = generate_moves(&search_state.board_position, false);

    let mut movecount = 0;

    for i in movelist {
        let result = search_state.board_position.make_move(i);
        
        if result == MoveSuccess::Success {
            let old_hash = search_state.board_position.hash;
            let cnt= perft_driver(search_state, depth - 1);
            search_state.board_position.take_back(i, old_hash);
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

#[cfg(test)]
mod tests{
    use std::thread;
    use crate::move_gen::generate_moves;
    use crate::nnue::NNUE;
    use crate::perft::perft_driver;
    use crate::types::shared::{ENDGAME_PERFT, KIWIPETE, MoveSuccess, START_POSITION};
    use crate::types::search_state::SearchState;
    use crate::types::tt::compute_hash;

    #[test]
    fn test_perft_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(KIWIPETE);
                let movecnt = perft_driver(&mut search_state, 5);
                assert_eq!(movecnt, 193690690);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_perft_endgame() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(ENDGAME_PERFT);
                let movecnt = perft_driver(&mut search_state, 6);
                assert_eq!(movecnt, 11030083);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_perft_startpos_intermediate_depths() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let expected = [20, 400, 8902, 197281, 4865609, 119060324];
                let mut search_state = SearchState::new(START_POSITION);
                for (depth, &exp) in expected.iter().enumerate() {
                    let movecnt = perft_driver(&mut search_state, depth + 1);
                    assert_eq!(movecnt, exp, "Perft mismatch at depth {}", depth + 1);
                }
            })
            .unwrap();
        handler.join().unwrap();
    }

    pub fn test_perft_driver_occupancies(search_state: &mut SearchState, depth: usize) -> usize {

        if depth == 0 {
            return 1;
        }
        
        let movelist = generate_moves(&search_state.board_position, false);
        
        let mut movecount = 0;

        for i in movelist {
            let result = search_state.make_move(i);
            assert_eq!(search_state.board_position.occupancies[0], search_state.board_position.bitboards[0..6].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &search_state.board_position.format_board());
            assert_eq!(search_state.board_position.occupancies[1], search_state.board_position.bitboards[6..12].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &search_state.board_position.format_board());
            
            if result == MoveSuccess::Success {
                movecount += test_perft_driver_occupancies(search_state, depth - 1);
                search_state.take_back(i);
                assert_eq!(search_state.board_position.occupancies[0], search_state.board_position.bitboards[0..6].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &search_state.board_position.format_board());
                assert_eq!(search_state.board_position.occupancies[1], search_state.board_position.bitboards[6..12].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &search_state.board_position.format_board());
            }
        }
        movecount

    }


    #[test]
    fn test_occupancy_calculation() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let mut search_state = SearchState::new(KIWIPETE);
                test_perft_driver_occupancies(&mut search_state, 5);
            })
            .unwrap();
        handler.join().unwrap();
    }


    pub fn test_perft_driver_copy_make(search_state: &mut SearchState, depth: usize) -> usize {

        if depth == 0 {
            return 1;
        }
        
        let movelist = generate_moves(&search_state.board_position, false);
        
        let mut movecount = 0;

        let board_clone = search_state.board_position.clone();

        for i in movelist {
            //null move test
            let old_ep = search_state.board_position.enpassant;
            search_state.make_null_move();
            search_state.take_back_null_move(old_ep);
            assert_eq!(compute_hash(&search_state.board_position), search_state.board_position.hash);
            assert_eq!(board_clone, search_state.board_position);

            let result = search_state.make_move(i);
                           
            let mut board_clone_after_move = search_state.board_position.clone();
            board_clone_after_move.refresh_nnue(&NNUE);
            assert_eq!(board_clone_after_move.accumulators, search_state.board_position.accumulators);
            
            if result == MoveSuccess::Success {
                movecount += test_perft_driver_copy_make(search_state, depth - 1);
                search_state.take_back(i);
            }

            assert_eq!(compute_hash(&search_state.board_position), search_state.board_position.hash);
            assert_eq!(board_clone, search_state.board_position);

            if depth == 3 {
                println!("Move: {:?}, movecount: {}", i, movecount);
            }
        }
        movecount

    }

    #[test]
    fn test_properly_unmake() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let mut search_state = SearchState::new(KIWIPETE);
                let clone_board = search_state.board_position.clone();
                test_perft_driver_copy_make(&mut search_state, 3);
                assert_eq!(search_state.board_position, clone_board);
            })
            .unwrap();
        handler.join().unwrap();
    }
}