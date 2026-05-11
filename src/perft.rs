use std::time::SystemTime;
use crate::move_gen::{generate_moves};
use crate::types::search_state::{SearchState};
use crate::shared::{MoveSuccess};

pub fn perft_driver(search_state: &mut SearchState, depth: usize) -> usize {

    if depth == 0 {
        return 1;
    }

    //print_board(&search_state.board_position);
    let movelist = generate_moves(&search_state.board_position);
    
    let mut movecount = 0;
    
    for i in movelist {

        let result = search_state.board_position.make_move(i);

        if result == MoveSuccess::Success {
            movecount += perft_driver(search_state, depth - 1);
            search_state.board_position.take_back(i);
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
        let result = search_state.board_position.make_move(i);

        if result == MoveSuccess::Success {
            let cnt= perft_driver(search_state, depth - 1);
            search_state.board_position.take_back(i);
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

use crate::{gui::parse_position, move_gen::generate_moves, perft::perft_driver, shared::{ENDGAME_PERFT_COMMAND, KIWIPETE_COMMAND, MoveSuccess, START_POSITION_COMMAND}, types::search_state::SearchState};

    #[test]
    fn test_perft_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut board_pos = parse_position(KIWIPETE_COMMAND); //Rook on e3
                let movecnt = perft_driver(&mut board_pos, 5);
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
                let mut board_pos = parse_position(ENDGAME_PERFT_COMMAND); //Rook on e3
                let movecnt = perft_driver(&mut board_pos, 6);
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
                let mut board_pos = parse_position(START_POSITION_COMMAND);
                for (depth, &exp) in expected.iter().enumerate() {
                    let movecnt = perft_driver(&mut board_pos, depth + 1);
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
        
        let movelist = generate_moves(&search_state.board_position);
        
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
                let mut board_pos = parse_position(KIWIPETE_COMMAND);
                test_perft_driver_occupancies(&mut board_pos, 5);
            })
            .unwrap();
        handler.join().unwrap();
    }


    pub fn test_perft_driver_copy_make(search_state: &mut SearchState, depth: usize) -> usize {

        if depth == 0 {
            return 1;
        }
        
        let movelist = generate_moves(&search_state.board_position);
        
        let mut movecount = 0;

        let board_clone = search_state.board_position.clone();

        for i in movelist {
            let result = search_state.make_move(i);
                    
            if result == MoveSuccess::Success {
                movecount += test_perft_driver_copy_make(search_state, depth - 1);
                search_state.take_back(i);
            }

            assert_eq!(board_clone, search_state.board_position);
        }
        movecount

    }

        #[test]
    fn test_properly_unmake() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let mut board_pos = parse_position(KIWIPETE_COMMAND);
                let clone_board = board_pos.board_position.clone();
                test_perft_driver_copy_make(&mut board_pos, 4);
                assert_eq!(board_pos.board_position, clone_board);
            })
            .unwrap();
        handler.join().unwrap();
    }
}