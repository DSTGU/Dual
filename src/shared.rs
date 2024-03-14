use std::collections::HashMap;
use std::fmt;
use std::fmt::Formatter;
use std::ops::BitAnd;

#[allow(non_camel_case_types)]
#[allow(unused_variables)]
#[allow(non_upper_case_globals)]
#[derive(Clone, Copy)]
pub struct BoardPosition {
    pub bitboards: [u64; 12],
    pub occupancies: [u64; 3],

    // side to move
    pub side: usize, // 0 - W, 1 - B, 2 - Default - none

    // en passant square
    pub enpassant: usize, // Number of square

    // castling rights
    pub castle: usize

    /*
    binary encoding
    0001    1  white king can castle to the king side
    0010    2  white king can castle to the queen side
    0100    4  black king can castle to the king side
    1000    8  black king can castle to the queen side
    */
}

pub struct Move{
    pub source_square: u8,
    pub target_square: u8,
    pub piece: Piece,
    pub promoted_piece: Piece,
    pub capture: bool,
    pub enpassant: bool,
    pub castling: bool,
    pub double_push: bool
}

impl Default for Move {
    fn default() -> Move {
        Move{
            source_square: 65,
            target_square: 65,
            piece: Piece::P,
            promoted_piece: Piece::P,
            capture: false,
            enpassant: false,
            castling: false,
            double_push: false
        }

    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Move {{ {}-{}, piece: {:?}, P={:?}{}{}{}{} }}",
            SQUARE_TO_COORDINATES[self.source_square as usize],
            SQUARE_TO_COORDINATES[self.target_square as usize],
            self.piece,
            self.promoted_piece,
            if self.capture { "+" } else { "" },
            if self.enpassant { "EP" } else { "" },
            if self.castling { "O-O" } else { "" },
            if self.double_push { "Double Push" } else { "" },
        )
    }
}

// board squares
pub enum Sq {
    a8, b8, c8, d8, e8, f8, g8, h8,
    a7, b7, c7, d7, e7, f7, g7, h7,
    a6, b6, c6, d6, e6, f6, g6, h6,
    a5, b5, c5, d5, e5, f5, g5, h5,
    a4, b4, c4, d4, e4, f4, g4, h4,
    a3, b3, c3, d3, e3, f3, g3, h3,
    a2, b2, c2, d2, e2, f2, g2, h2,
    a1, b1, c1, d1, e1, f1, g1, h1, no_sq = 64
}

// encode pieces
#[derive(Debug)]
pub enum Piece { P = 0, N = 1, B = 2, R = 3, Q = 4, K = 5, p = 6, n = 7, b = 8, r = 9, q = 10, k = 11}

pub fn pieceTousize(piece: Piece) -> usize{
    match piece {
        Piece::P => 0,
        Piece::N => 1,
        Piece::B => 2,
        Piece::R => 3,
        Piece::Q => 4,
        Piece::K => 5,
        Piece::p => 6,
        Piece::n => 7,
        Piece::b => 8,
        Piece::r => 9,
        Piece::q => 10,
        Piece::k => 11
    }
}

pub enum Castle { wk = 1, wq = 2, bk = 4, bq = 8 }

impl BitAnd<Castle> for usize {
    type Output = usize;

    fn bitand(self, rhs: Castle) -> usize {
        self & rhs as usize
    }
}

pub enum Side {white = 0,black = 1,both = 2}


pub const SQUARE_TO_COORDINATES: [&str; 64] = [
    "a8", "b8", "c8", "d8", "e8", "f8", "g8", "h8",
    "a7", "b7", "c7", "d7", "e7", "f7", "g7", "h7",
    "a6", "b6", "c6", "d6", "e6", "f6", "g6", "h6",
    "a5", "b5", "c5", "d5", "e5", "f5", "g5", "h5",
    "a4", "b4", "c4", "d4", "e4", "f4", "g4", "h4",
    "a3", "b3", "c3", "d3", "e3", "f3", "g3", "h3",
    "a2", "b2", "c2", "d2", "e2", "f2", "g2", "h2",
    "a1", "b1", "c1", "d1", "e1", "f1", "g1", "h1",
];

const ASCII_PIECES: [u8; 12] = [80, 78, 66, 82, 81, 75, 112, 110, 98, 114, 113, 107];

// convert ASCII character pieces to encoded constants
const char_pieces: [(char, i32); 12] = [
('P', 0),
('N', 1),
('B', 2),
('R', 3),
('Q', 4),
('K', 5),
('p', 6),
('n', 7),
('b', 8),
('r', 9),
('q', 10),
('k', 11),
];


/**********************************\
==================================

          Bit manipulations

==================================
\**********************************/

pub fn set_bit(bitboard: &mut u64, square: usize) {
    *bitboard |= 1u64 << square;
}

pub fn get_bit(bitboard: u64, square: usize) -> bool {
    (bitboard & (1u64 << square)) != 0
}

pub fn pop_bit(bitboard: &mut u64, square: usize) { *bitboard &= !(1u64 << square); }


/***************************\

               IO

\**************************/

pub fn print_bitboard(bitboard: u64) {
    println!();

    // loop over board ranks
    for rank in (0..8).rev() {
        // loop over board files
        for file in 0..8 {
            // convert file & rank into square index
            let square = rank * 8 + file;

            // print ranks
            if file == 0 {
                print!("  {} ", 8 - rank);
            }

            // print bit state (either 1 or 0)
            let bit_state = if get_bit(bitboard, square) { 1 } else { 0 };
            print!(" {}", bit_state);
        }

        // print new line every rank
        println!();
    }

    // print board files
    println!("\n     a b c d e f g h\n");

    // print bitboard as unsigned decimal number
    println!("     Bitboard: {}", bitboard);
}

// print board
pub fn print_board(board: &BoardPosition)
{
    // print offset
    println!();

    // loop over board ranks
    for rank in 0..8
    {
        // loop over board files
        for file in 0..8
        {
        // init square
            let square = rank * 8 + file;

            // print ranks
            if file == 0 {
                print!("  {} ", 8 - rank);
            }

            // define piece variable
            let mut piece = 12 as usize;

            // loop over all piece bitboards
            for bb_piece in 0..12
            {
                if get_bit(board.bitboards[bb_piece], square) {
                    piece = bb_piece;
                }
            }

            if piece == 12
            {
                print!(" .");
            }
            else {
                print!(" {}", char::from(ASCII_PIECES[piece]));
            }
        }

    // print new line every rank
    println!();
    }

    // print board files
    println!("\n     a b c d e f g h\n\n");

    match board.side {
        0 => println!("White"),
        1 => println!("Black"),
        _ => println!("No side"),
    }

    match board.enpassant {
        64 => println!("Enpassant not available"),
        _ => println!("Enpassant: {}", SQUARE_TO_COORDINATES[board.enpassant]),
    }


    // print castling rights

    if board.castle & Castle::wk != 0
    {
        print!("K");
    }
    if board.castle & Castle::wq != 0
    {
        print!("Q");
    }
    if board.castle & Castle::bk != 0
    {
        print!("k");
    }
    if board.castle & Castle::bq != 0
    {
        print!("q");
    }
    println!();
}
//Count bits - x.countones()
//LS1b - trailing zeros !! - invalid = 64, not -1

/******************************************\

                   FEN STUFF

\******************************************/

// FEN debug positions
pub const empty_board: &str = "8/8/8/8/8/8/8/8 w - - ";
pub const start_position: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";
pub const tricky_position: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
pub const killer_position: &str = "rnbqkb1r/pp1p1pPp/8/2p1pP2/1P1P4/3P3P/P1P1P3/RNBQKBNR w KQkq e6 0 1";
pub const cmk_position: &str = "r2q1rk1/ppp2ppp/2n1bn2/2b1p3/3pP3/3P1NPP/PPP1NPB1/R1BQ1RK1 b - - 0 9 ";

pub fn parse_fen(fen: &str) -> BoardPosition {

    let mut board_position = BoardPosition {
        bitboards: [0; 12],
        occupancies: [0; 3],
        side: 2,
        enpassant: 64,
        castle: 0,
    };

    let mut fen_chars = fen.chars();
    let mut rank = 0;
    let mut file = 0;

    while let Some(ch) = fen_chars.next() {
        if ch.is_ascii_alphabetic() {
            let piece = match ch {
                'P' => 0,
                'N' => 1,
                'B' => 2,
                'R' => 3,
                'Q' => 4,
                'K' => 5,
                'p' => 6,
                'n' => 7,
                'b' => 8,
                'r' => 9,
                'q' => 10,
                'k' => 11,
                _ => continue,
            };
            let square = rank * 8 + file;
            board_position.bitboards[piece] |= 1u64 << square;
            file += 1;
        } else if ch.is_digit(10) {
            let offset = ch.to_digit(10).unwrap() as usize;
            file += offset;
        } else if ch == '/' {
            rank += 1;
            file = 0;
        } else if ch == ' ' {
            break;
        }
    }

    if let Some(ch) = fen_chars.next() {
        board_position.side = match ch {
            'w' => 0,
            'b' => 1,
            _ => 2,
        };
        fen_chars.next();
    }

    while let Some(ch) = fen_chars.next() {
        if ch == ' ' {
            break;
        }
        match ch {
            'K' => board_position.castle |= 1,
            'Q' => board_position.castle |= 2,
            'k' => board_position.castle |= 4,
            'q' => board_position.castle |= 8,
            _ => continue,
        }
    }

    if let Some(ch) = fen_chars.next() {
        if ch != '-' {
            let file = match ch {
                'a'..='h' => (ch as u8 - b'a') as usize,
                _ => {
                    // Handle the case when the file is invalid
                    // You can choose to return an error, set a default value, or handle it in another way
                    // For now, let's set it to 0
                    0
                }
            };
            let rank = match fen_chars.next() {
                Some(rank_ch @ '1'..='8') => 8 - (rank_ch as u8 - b'0') as usize,
                _ => {
                    // Handle the case when the rank is invalid
                    // You can choose to return an error, set a default value, or handle it in another way
                    // For now, let's set it to 0
                    0
                }
            };

            board_position.enpassant = rank * 8 + file;
        }
    }


    for piece in 0..=5 {
        board_position.occupancies[0] |= board_position.bitboards[piece];
    }

    for piece in 6..=11 {
        board_position.occupancies[1] |= board_position.bitboards[piece];
    }

    board_position.occupancies[2] = board_position.occupancies[0] | board_position.occupancies[1];

    board_position
}