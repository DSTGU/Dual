mod shared;
mod attacks;
mod moveGen;

use std::default;
use std::thread;
use shared::Sq;
use shared::BoardPosition;
use shared::SQUARE_TO_COORDINATES;
use stacker::grow;

/**********************************\
 ==================================

          Bit manipulations

 ==================================
\**********************************/

use shared::set_bit;
use shared::get_bit;
use shared::pop_bit;

use shared::print_bitboard;

/**********************************\
 ==================================

              Attacks

 ==================================
\**********************************/

use attacks::PAWN_ATTACKS;
use attacks::KNIGHT_ATTACKS;
use attacks::KING_ATTACKS;
use attacks::ROOK_ATTACKS;
use attacks::BISHOP_ATTACKS;
use crate::moveGen::{generate_moves, make_move};

use crate::shared::{cmk_position, empty_board, Move, parse_fen, Piece, print_board, start_position, tricky_position};



/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

fn main() {


    let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
    let handler = builder.spawn(|| {
        // thread code

        let mut boardPos = parse_fen(tricky_position);
        print_board(&boardPos);
        //boardPos.side = 1;
        let moveList = generate_moves(&boardPos);

        for i in moveList{
            print!("{}-{}", SQUARE_TO_COORDINATES[i.source_square as usize], SQUARE_TO_COORDINATES[i.target_square as usize]);
            let alt = make_move(&boardPos, i);
            print_board(&alt);
        }
        //print_board(boardPos);

    }).unwrap();
    handler.join().unwrap();
}
