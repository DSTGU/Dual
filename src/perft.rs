use std::time::SystemTime;
use crate::moveGen::{generate_moves, make_move};
use crate::shared::{coordinates_to_squares, print_board, BoardPosition, Move, SQUARE_TO_COORDINATES};

pub fn perft_driver(board: &BoardPosition, depth: usize) -> usize {
    
    if depth == 0 {
        return 1;
    }
    
    let movelist = generate_moves(board);
    
    let mut movecount = 0;

    for i in movelist {
        let board = make_move(board,&i);

        if let Some(board) = board {
            movecount += perft_driver(&board, depth - 1);
        }
    }
    movecount

}


pub fn perft(board: &BoardPosition, depth: usize) {
    
    print_board(board);
    
    if depth == 0 {
        return;
    }
    let now = SystemTime::now();
    let movelist = generate_moves(board);

    let mut movecount = 0;

    for i in movelist {
        
        let board = make_move(board,&i);

        let mut cnt = 0;
        if let Some(board) = board {
            cnt = perft_driver(&board, depth - 1);
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

pub fn pure_perft(board: &BoardPosition, depth: usize) {
    
    let movelist = generate_moves(board);
    
    let mut movecount = 0;
    
    for i in movelist {
        let board = make_move(board,&i);
        let mut cnt = 0;
            if let Some(board) = board {
                cnt = perft_driver(&board, depth - 1);
                println!("{}{} {}", SQUARE_TO_COORDINATES[i.source_square as usize], SQUARE_TO_COORDINATES[i.target_square as usize], cnt);
            }
        movecount += cnt;
        
    }
    
    println!("\n{}", movecount);

}