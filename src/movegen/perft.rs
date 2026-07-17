use std::time::SystemTime;
use crate::movegen::move_gen::{generate_moves};
use crate::primitives::board::BoardPosition;

pub fn perft_driver(board_position: &BoardPosition, depth: usize) -> usize {

    if depth == 0 {
        return 1;
    }

    //print_board(&search_state.board_position);
    let movelist = generate_moves(board_position, false);
    
    let mut movecount = 0;
    
    for i in movelist {

        let result = board_position.make_move(i);
        if result.is_none() {
            continue;
        }

        let new_board = result.unwrap();
        movecount += perft_driver(&new_board, depth - 1);
    }

    movecount

}


pub fn perft(board_position: &BoardPosition, depth: usize) {
    
    board_position.print_board();
    
    if depth == 0 {
        return;
    }

    let now = SystemTime::now();
    let movelist = generate_moves(board_position, false);

    let mut movecount = 0;

    for i in movelist {
        let result = board_position.make_move(i);
        if result.is_none() {
            continue;
        }

        let new_board = result.unwrap();

        let cnt= perft_driver(&new_board, depth - 1);
        println!("{:?}, Moves: {}", i, cnt);

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

#[cfg(test)]
mod tests{
    use std::thread;
    use crate::movegen::move_gen::generate_moves;
    use crate::movegen::perft::perft_driver;
    use crate::primitives::board::BoardPosition;
    use crate::primitives::shared::{ENDGAME_PERFT, KIWIPETE, START_POSITION};

    #[test]
    fn test_perft_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_position = BoardPosition::new(KIWIPETE);
                let movecnt = perft_driver(&board_position, 5);
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
                let board_position = BoardPosition::new(ENDGAME_PERFT);
                let movecnt = perft_driver(&board_position, 6);
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
                let board_position = BoardPosition::new(START_POSITION);
                for (depth, &exp) in expected.iter().enumerate() {
                    let movecnt = perft_driver(&board_position, depth + 1);
                    assert_eq!(movecnt, exp, "Perft mismatch at depth {}", depth + 1);
                }
            })
            .unwrap();
        handler.join().unwrap();
    }

    pub fn test_perft_driver_occupancies(board_position: &BoardPosition, depth: usize) -> usize {

        if depth == 0 {
            return 1;
        }
        
        let movelist = generate_moves(board_position, false);
        
        let mut movecount = 0;

        for i in movelist {
            let result = board_position.make_move(i);
            if result.is_none() {
                continue;
            }
            let result = result.unwrap();
            assert_eq!(result.occupancies[0], result.bitboards[0..6].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &result.format_board());
            assert_eq!(result.occupancies[1], result.bitboards[6..12].iter().fold(0, |acc, &b| acc | b), "Board\n{:?}", &result.format_board());
            movecount += test_perft_driver_occupancies(&result, depth - 1);
        }
        movecount

    }


    #[test]
    fn test_occupancy_calculation() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let board_position = BoardPosition::new(KIWIPETE);
                test_perft_driver_occupancies(&board_position, 5);
            })
            .unwrap();
        handler.join().unwrap();
    }
}