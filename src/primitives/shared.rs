use std::{fmt, mem};
use std::ops::{BitAnd, Index, IndexMut};

use crate::primitives::shared::Color::White;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
#[repr(u8)]
pub enum Color {
    White = 0,
    Black = 1,
}

impl Color {
    pub fn invert(self) -> Color {
        unsafe { std::mem::transmute( 1 - self as u8) }
    }

    pub const fn new(value: u8) -> Self {
        debug_assert!(value < 2);
        unsafe { std::mem::transmute(value) }
    }
}

impl<T> Index<Color> for [T] {
    type Output = T;

    fn index(&self, index: Color) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Color> for [T] {
    fn index_mut(&mut self, index: Color) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

#[derive(Debug)]
pub struct SearchAnswer {
    //pub search_state: SearchState,
    pub move_list: Vec<Option<Move>>,
    pub node_count: i32,
    pub eval: i32
}

#[derive(Clone, Copy, PartialEq)]
pub enum MoveCode {
    QuietMove = 0,
    DoublePush = 1,
    KingCastle = 2,
    QueenCastle = 3,
    Capture = 4,
    EnPassant = 5,
    KnightPromotion = 8,
    BishopPromotion = 9,
    RookPromotion = 10,
    QueenPromotion = 11,
    KnightPromotionCapture = 12,
    BishopPromotionCapture = 13,
    RookPromotionCapture = 14,
    QueenPromotionCapture = 15,
}


#[derive(Clone, Copy, PartialEq)]
pub struct Move(u16);

impl Move {
    pub fn create(
        source_square: u8,
        target_square: u8,
        move_code : MoveCode
    ) -> Move {
        let value: u16 =
            (source_square as u16) // 6 bits
            | ((target_square as u16) << 6) // 6 bits
            | ((move_code as u16) << 12); // 4 bits
        Move(value)
    }

    pub const fn create_null() -> Move {
        Move(0)
    }

    pub const fn is_null(self) -> bool {
        self.0 == 0
    }

    pub fn get_source_square(self) -> u8 {
        (self.0 & 0x3f) as u8
    }

    pub fn get_target_square(self) -> u8 {
        (self.0 >> 6) as u8 & 0x3f
    }

    pub fn get_move_code(self) -> MoveCode {
        unsafe { mem::transmute(((self.0 >> 12) & 0xf) as u8 ) }
    }

    pub fn get_promoted_piece_idx(self, side: Color) -> u8 {
        if !self.is_promotion() {
            return 12; // Piece::None index
        }        

        let promo_idx = ((self.0 >> 12) & 0x3) as u8;
        6 * (side as u8) + promo_idx + 1   
    }

    pub fn get_promoted_piece(self, side: Color) -> Piece {
        Piece::new(self.get_promoted_piece_idx(side) as usize)
    }

    pub fn is_capture(self) -> bool {
        (self.0 >> 14) & 1 != 0
    }

    pub fn is_promotion(self) -> bool {
        (self.0 >> 15) & 1 != 0
    }

    pub fn is_enpassant(self) -> bool {
        MoveCode::EnPassant == self.get_move_code()
    }

    pub fn is_quiet(self) -> bool {
        MoveCode::QuietMove == self.get_move_code()
    }
 
    pub fn get_castling(self) -> bool {
        matches!(
            self.get_move_code(),
            MoveCode::KingCastle | MoveCode::QueenCastle
        )
    }

    pub fn get_double_pawn_push(self) -> bool {
        MoveCode::DoublePush == self.get_move_code()
    }
 
}

pub fn move_to_alg(mv: &Move) -> String {
    match mv.get_promoted_piece_idx(White) { // white
        4 => format!("{}{}q", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        //10 => format!("{}{}q", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        1 => format!("{}{}n", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        //7 => format!("{}{}n", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        3 => format!("{}{}r", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        //9 => format!("{}{}r", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        2 => format!("{}{}b", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        //8 => format!("{}{}b", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize]),
        _ => format!("{}{}", SQUARE_TO_COORDINATES[mv.get_source_square() as usize], SQUARE_TO_COORDINATES[mv.get_target_square() as usize])
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Move {{ {}({})-{}({}): P={:?}{}{}{} }}",
            SQUARE_TO_COORDINATES[self.get_source_square() as usize],
            self.get_source_square(),
            SQUARE_TO_COORDINATES[self.get_target_square() as usize],
            self.get_target_square(),
            self.get_promoted_piece(White),
            if self.is_enpassant() { " EP" } else { "" },
            if self.get_castling() { " O-O" } else { "" },
            if self.get_double_pawn_push() { " dblPP" } else { "" },
        )
    }
}


// encode pieces
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
#[allow(non_camel_case_types)]
#[repr(u8)]
pub enum Piece { P = 0, N = 1, B = 2, R = 3, Q = 4, K = 5, p = 6, n = 7, b = 8, r = 9, q = 10, k = 11, NONE = 12}

impl Piece {
    pub fn new(idx: usize) -> Piece {
        match idx {
            0 => Piece::P,
            1 => Piece::N,
            2 => Piece::B,
            3 => Piece::R,
            4 => Piece::Q,
            5 => Piece::K,
            6 => Piece::p,
            7 => Piece::n,
            8 => Piece::b,
            9 => Piece::r,
            10 => Piece::q,
            11 => Piece::k,
            _ => Piece::NONE,
        }
    }

    pub const fn get_side(self) -> Color {
        Color::new(self as u8 / 6)
    }

    pub fn flip_color(self) -> Piece {
        debug_assert!(self != Piece::NONE);
        Piece::new((self as usize + 6) % 12)
    }
}

impl<T> Index<Piece> for [T] {
    type Output = T;

    fn index(&self, index: Piece) -> &Self::Output {
        &self[index as usize]
    }
}

impl<T> IndexMut<Piece> for [T] {
    fn index_mut(&mut self, index: Piece) -> &mut Self::Output {
        &mut self[index as usize]
    }
}

pub enum Castle { Wk = 1, Wq = 2, Bk = 4, Bq = 8 }

impl BitAnd<Castle> for usize {
    type Output = usize;

    fn bitand(self, rhs: Castle) -> usize {
        self & rhs as usize
    }
}

// pub enum Side {white = 0,black = 1,both = 2}


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

pub fn coordinates_to_squares(coordinatestr: &str) -> u8 {
    let mut val  = 0;
    let mut chars = coordinatestr.chars();
    let first = chars.next().unwrap();
    let second = chars.next().unwrap();
    
    if first.is_ascii_alphabetic() && second.is_digit(10) {
        match first {
            'a' => val += 0,
            'b' => val += 1,
            'c' => val += 2,
            'd' => val += 3,
            'e' => val += 4,
            'f' => val += 5,
            'g' => val += 6,
            'h' => val += 7,
            _ => return 65
        }
        val = val + 56 - ((second.to_digit(10).unwrap() as i32 - 1) * 8);

    }
    
    val as u8
}

pub const ASCII_PIECES: [u8; 12] = [80, 78, 66, 82, 81, 75, 112, 110, 98, 114, 113, 107];

// convert ASCII character pieces to encoded constants
// const CHAR_PIECES: [(char, i32); 12] = [
// ('P', 0),
// ('N', 1),
// ('B', 2),
// ('R', 3),
// ('Q', 4),
// ('K', 5),
// ('p', 6),
// ('n', 7),
// ('b', 8),
// ('r', 9),
// ('q', 10),
// ('k', 11),
// ];


/**********************************\
          Bit manipulations
\**********************************/

#[inline(always)]
pub fn set_bit(bitboard: &mut u64, square: usize) {
    *bitboard |= 1u64 << square;
}

#[inline(always)]
pub fn get_bit(bitboard: u64, square: usize) -> bool {
    (bitboard & (1u64 << square)) != 0
}

#[inline(always)]
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
    println!("{}", bitboard);
}

//Count bits - x.countones()
//LS1b - trailing zeros !! - invalid = 64, not -1

/******************************************\
                   FEN STUFF
\******************************************/

// FEN debug positions
pub const START_POSITION: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 ";
pub const KIWIPETE: &str = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1 ";
pub const ENDGAME_PERFT: &str = "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1 ";

// Piece index offsets: white pieces are 0..6, black pieces are 6..12.
// For a given side, the king bitboard index is:
//   side 0 (white) -> Piece::K = 5
//   side 1 (black) -> Piece::k = 11
pub const KING_INDEX: [usize; 2] = [Piece::K as usize, Piece::k as usize];


#[cfg(test)]
mod tests {
    use crate::primitives::shared::{Color::White, Move, MoveCode, Piece};

    #[test]
    fn move_constructor_test_promotion() {
        let source = 11;
        let target = 27;
        let promoted = Piece::Q;
        let capture = 1;
        let enpassant = 0;
        let castling = 0;
        let double_pawn_push = 0;


        let move_to_test = Move::create(source, target, MoveCode::QueenPromotionCapture);
        assert_eq!(move_to_test.get_source_square(), source);
        assert_eq!(move_to_test.get_target_square(), target);
        assert_eq!(move_to_test.get_promoted_piece(White), promoted);
        assert_eq!(move_to_test.is_capture(), capture != 0);
        assert_eq!(move_to_test.is_enpassant(), enpassant != 0);
        assert_eq!(move_to_test.get_castling(), castling != 0);
        assert_eq!(move_to_test.get_double_pawn_push(), double_pawn_push != 0);
    }

    #[test]
    fn move_constructor_test_castling() {
        let source = 4;
        let target = 6;
        let promoted = Piece::NONE;
        let capture = 0;
        let enpassant = 0;
        let castling = 1;
        let double_pawn_push = 0;


        let move_to_test = Move::create(source, target, MoveCode::KingCastle);
        assert_eq!(move_to_test.get_source_square(), source);
        assert_eq!(move_to_test.get_target_square(), target);
        assert_eq!(move_to_test.get_promoted_piece(White), promoted);
        assert_eq!(move_to_test.is_capture(), capture != 0);
        assert_eq!(move_to_test.is_enpassant(), enpassant != 0);
        assert_eq!(move_to_test.get_castling(), castling != 0);
        assert_eq!(move_to_test.get_double_pawn_push(), double_pawn_push != 0);
    }

    #[test]
    fn move_constructor_test_enpassant() {
        let source = 24;
        let target = 16;
        let promoted = Piece::NONE;
        let capture = 1;
        let enpassant = 1;
        let castling = 0;
        let double_pawn_push = 0;

        let move_to_test = Move::create(source, target, MoveCode::EnPassant);
        assert_eq!(move_to_test.get_source_square(), source);
        assert_eq!(move_to_test.get_target_square(), target);
        assert_eq!(move_to_test.get_promoted_piece(White), promoted);
        assert_eq!(move_to_test.is_capture(), capture != 0);
        assert_eq!(move_to_test.is_enpassant(), enpassant != 0);
        assert_eq!(move_to_test.get_castling(), castling != 0);
        assert_eq!(move_to_test.get_double_pawn_push(), double_pawn_push != 0);
    }
}
