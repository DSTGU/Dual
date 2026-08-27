use crate::movegen::attacks::get_piece_attacks;
use crate::movegen::move_gen::{CASTLING_RIGHTS, is_square_attacked};
use crate::primitives::shared::Color::{Black, White};
use crate::primitives::shared::{ASCII_PIECES, Castle, Color, KING_INDEX, Move, Piece, SQUARE_TO_COORDINATES, get_bit, pop_bit, set_bit};
use crate::primitives::hash::{compute_hash, get_zobrist_keys};

#[allow(non_camel_case_types)]
#[allow(unused_variables)]
#[allow(non_upper_case_globals)]
#[derive(Clone, Debug, PartialEq)]
pub struct BoardPosition {
    pub bitboards: [u64; 12],
    pub occupancies: [u64; 3],
    pub mailbox: [Piece; 64],

    // side to move
    pub side: Color,

    // en passant square
    pub enpassant: u8, // Number of square

    // castling rights
    pub castle: usize,

    pub hash: u64,

    pub fifty_mr: u8,
}
    /*
    binary encoding
    0001    1  white king can castle to the king side
    0010    2  white king can castle to the queen side
    0100    4  black king can castle to the king side
    1000    8  black king can castle to the queen side
    */

impl BoardPosition {

    pub fn new(fen: &str) -> BoardPosition {
        let mut board_position = BoardPosition {
            bitboards: [0; 12],
            occupancies: [0; 3],
            mailbox: [Piece::NONE; 64],
            side: White,
            enpassant: 0,
            castle: 0,
            hash: 0,
            fifty_mr: 0
        };

        board_position.parse_fen(fen);
        board_position.hash = compute_hash(&board_position);

        board_position
    }

    pub fn parse_fen(&mut self, fen: &str) {

        self.bitboards = [0; 12];
        self.occupancies = [0; 3];
        self.mailbox = [Piece::NONE; 64];
        self.side = White;
        self.enpassant = 0;
        self.castle = 0;

        let mut fen_chars = fen.chars();
        let mut rank = 0;
        let mut file = 0;

        while let Some(ch) = fen_chars.next() {
            if ch.is_ascii_alphabetic() {
                let piece = match ch {
                    'P' => Piece::P,
                    'N' => Piece::N,
                    'B' => Piece::B,
                    'R' => Piece::R,
                    'Q' => Piece::Q,
                    'K' => Piece::K,
                    'p' => Piece::p,
                    'n' => Piece::n,
                    'b' => Piece::b,
                    'r' => Piece::r,
                    'q' => Piece::q,
                    'k' => Piece::k,
                    _ => continue,
                };
                let square = rank * 8 + file;
                    self.add_piece(square, piece, false);
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
            self.side = match ch {
                'w' => White,
                'b' => Black,
                _ => White,
            };
            fen_chars.next();
        }

        while let Some(ch) = fen_chars.next() {
            if ch == ' ' {
                break;
            }
            match ch {
                'K' => self.castle |= 1,
                'Q' => self.castle |= 2,
                'k' => self.castle |= 4,
                'q' => self.castle |= 8,
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

                self.enpassant = (rank * 8 + file) as u8;
            }
        }

        // Keeping this comment for better times, when the state regarding previous move is gonna be kept on a stack, not deduced
        fen_chars.next(); // skip space

        let mut word = String::new();

        while let Some(c) = fen_chars.next() {
            if c.is_whitespace() {
                break;
            }

            word.push(c);
        }

        self.fifty_mr = word.parse().unwrap_or(0);

        for piece in 0..=5 {
            self.occupancies[0] |= self.bitboards[piece];
        }

        for piece in 6..=11 {
            self.occupancies[1] |= self.bitboards[piece];
        }

        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];
    }

    //Only works before move
    pub fn get_piece(&self, mv: Move) -> Piece {
        self.mailbox[mv.get_source_square() as usize]
    }

    pub fn has_pieces(&self) -> bool {
        self.bitboards[Piece::B as usize] > 0 ||
        self.bitboards[Piece::R as usize] > 0 ||
        self.bitboards[Piece::N as usize] > 0 ||
        self.bitboards[Piece::Q as usize] > 0 ||
        self.bitboards[Piece::b as usize] > 0 ||
        self.bitboards[Piece::r as usize] > 0 ||
        self.bitboards[Piece::n as usize] > 0 ||
        self.bitboards[Piece::q as usize] > 0
    }

    pub fn make_null_move(&self) -> BoardPosition {
        let mut new_board=  self.clone();
        
        new_board.side = self.side.invert();
        new_board.hash ^= get_zobrist_keys().side_key;

        if new_board.enpassant != 0 {
            new_board.hash ^= get_zobrist_keys().enpassant_keys[(new_board.enpassant % 8) as usize];
            new_board.enpassant = 0;
        }

        new_board
    }

    pub fn is_king_attacked(&self) -> bool {
        is_square_attacked(self.bitboards[6*self.side as usize+5].trailing_zeros() as u8, self)
    }

    #[inline(always)]
    pub fn remove_piece(&mut self, square: usize, piece: Piece, update_hash: bool) {
        debug_assert!(self.mailbox[square] == piece);
        debug_assert!(get_bit(self.bitboards[piece as usize], square));

        if update_hash {
            self.hash ^= get_zobrist_keys().piece_keys[piece as usize][square];
        }

        self.mailbox[square] = Piece::NONE;
        pop_bit(&mut self.occupancies[piece.get_side()], square);
        pop_bit(&mut self.bitboards[piece as usize], square);
    }

    #[inline(always)]
    pub fn add_piece(&mut self, square: usize, piece: Piece, update_hash: bool) {
        if update_hash {
            self.hash ^= get_zobrist_keys().piece_keys[piece as usize][square];
        }

        self.mailbox[square] = piece;
        set_bit(&mut self.occupancies[piece.get_side()], square);
        set_bit(&mut self.bitboards[piece as usize], square);
    }

    #[inline(always)]
    pub fn get_victim(&self, mv: Move) -> Piece {
        if mv.is_enpassant() {
            return self.mailbox[mv.get_source_square() as usize].flip_color();
        }
            
        self.mailbox[mv.get_target_square() as usize]
    }

    /// Apply `move_to_make` to `board` and return the resulting position, or
    /// `None` if the move leaves the king in check.
    pub fn make_move(&self, move_to_make: Move) -> Option<BoardPosition> {

        let mut new_board = self.clone();

        let keys = get_zobrist_keys();

        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        
        let piece = self.mailbox[source];
        let is_capture = move_to_make.is_capture();
        let is_enpassant = move_to_make.is_enpassant();
        let is_castling = move_to_make.get_castling();
        let is_double_push = move_to_make.get_double_pawn_push();
        let promoted = move_to_make.is_promotion();

        // //handle 50mr
        if is_capture || piece == Piece::P || piece == Piece::p {
            new_board.fifty_mr = 0;
        } else {
            new_board.fifty_mr += 1;
        }

        // Handle captures: 
        if is_capture && !is_enpassant {
            new_board.remove_piece(target, new_board.mailbox[target], true);
        }

        new_board.remove_piece(source, piece, true);
        new_board.add_piece(target, piece, true);

        // Handle promotion: replace the pawn with the promoted piece.
        if promoted {
            new_board.remove_piece(target, piece, true);
            new_board.add_piece(target, move_to_make.get_promoted_piece(new_board.side), true);
        }

        // Handle en passant: remove the captured pawn (which is on a different
        // square from the target).
        if is_enpassant {
            let ep_sq = if new_board.side == White {
                target + 8
            } else {
                target - 8
            };

            let pawn = if new_board.side == White {
                Piece::p
            } else {
                Piece::P
            };

            new_board.remove_piece(ep_sq, pawn, true);
        }

        if new_board.enpassant != 0 {
            new_board.hash ^= keys.enpassant_keys[(new_board.enpassant % 8) as usize];
        }

        // Reset en passant square; set it again if this was a double pawn push.
        new_board.enpassant = 0;

        if is_double_push {
            new_board.enpassant = if new_board.side == White {
                target as u8 + 8
            } else {
                target as u8 - 8
            };
        }

        if new_board.enpassant != 0 {
            new_board.hash ^= keys.enpassant_keys[(new_board.enpassant % 8) as usize];
        }

        // Handle castling: move the rook.
        if is_castling {
            let (rook_piece, rook_from, rook_to) = match target {
                62 => (Piece::R, 63, 61),
                58 => (Piece::R, 56, 59),
                6  => (Piece::r, 7, 5),
                2  => (Piece::r, 0, 3),
                _ => unsafe { std::hint::unreachable_unchecked() }
            };

            new_board.remove_piece(rook_from, rook_piece, true);
            new_board.add_piece(rook_to, rook_piece, true);
        }

        for i in 0..4 {
            if new_board.castle & (1 << i) != 0 {
                new_board.hash ^= keys.castling_keys[i];
            }
        }

        // Update castling rights.
        new_board.castle &= CASTLING_RIGHTS[source] as usize;
        new_board.castle &= CASTLING_RIGHTS[target] as usize;

        for i in 0..4 {
            if new_board.castle & (1 << i) != 0 {
                new_board.hash ^= keys.castling_keys[i];
            }
        }

        // Recompute occupancies.
        new_board.occupancies[2] = new_board.occupancies[0] | new_board.occupancies[1];

        // Find the king of the side that just moved to check for legality.
        let king_sq =
            new_board.bitboards[KING_INDEX[new_board.side]].trailing_zeros() as u8;

        if is_square_attacked(king_sq, &new_board) {
            return None;
        }

        // Flip side for the returned position.
        new_board.side = new_board.side.invert();
        new_board.hash ^= keys.side_key;

        Some(new_board)
    }

    // Is legal but not checking if is actually legal, but if it can be tried:
    // moves on a legal trajectory of a piece which is placed on the source square
    // pub fn is_safe_to_try(&self, mv: Move) -> bool {
    //     let piece = self.mailbox[mv.get_source_square() as usize];

    //     if piece == Piece::NONE || piece.get_side() != self.side {
    //         return false;
    //     }

    //     // Note - get_piece_attacks checks for if a piece CAN BE AN ATTACKER OF A GIVEN SQUARE
    //     // Therefore source and target swapped
    //     if get_piece_attacks(self, mv.get_target_square(), piece) & 1 << mv.get_source_square() == 0  {
    //         return false;
    //     }

    //     true
    // }

    // 99.9% correct
    pub fn can_make_move(&self, mv: Move) -> bool {
        let piece = self.mailbox[mv.get_source_square() as usize];
        let source = mv.get_source_square();
        let target = mv.get_target_square();

        if piece == Piece::NONE || piece.get_side() != self.side {
            return false;
        }

        if (piece == Piece::P || piece == Piece::p) && !mv.is_capture() {
            return source % 8 == target % 8;
        }

        if mv.get_castling() {
            return piece == Piece::K || piece == Piece::k;
        }

        // Note - get_piece_attacks checks for if a piece CAN BE AN ATTACKER OF A GIVEN SQUARE
        // Therefore source and target swapped
        if get_piece_attacks(self, mv.get_target_square(), piece) & (1 << mv.get_source_square()) == 0  {
            return false;
        }

        true
    }

    // print board
    pub fn format_board(&self) -> String
    {
        let mut output = "\n".to_owned();

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
                    output += &format!("  {} ", 8 - rank);
                }

                // define piece variable
                let mut piece = 12;

                // loop over all piece bitboards
                for bb_piece in 0..12
                {
                    if get_bit(self.bitboards[bb_piece], square) {
                        piece = bb_piece;
                    }
                }

                if piece == 12
                {
                    output += " .";
                }
                else {
                    output += &format!(" {}", char::from(ASCII_PIECES[piece]));
                }
            }

        // print new line every rank
            output += "\n";
        }

        // print board files
        output += "\n     a b c d e f g h\n\n";

        match self.side {
            White => output += "White\n",
            Black => output += "Black\n",
        }

        match self.enpassant {
            0 => output += "Enpassant not available\n",
            _ =>  output += &format!("Enpassant: {}\n", SQUARE_TO_COORDINATES[self.enpassant as usize]),
        }


        // print castling rights

        if self.castle & Castle::Wk != 0
        {
            output += "K";
        }
        if self.castle & Castle::Wq != 0
        {
            output += "Q";
        }
        if self.castle & Castle::Bk != 0
        {
            output += "k";
        }
        if self.castle & Castle::Bq != 0
        {
            output += "q";
        }
        output += "\n";

        output
    }

    pub fn print_board(&self) {
        println!("{}", self.format_board());
    }

    /// Serialize the position to a FEN string. `fullmove` is the fullmove
    /// counter (the 6th FEN field, which the engine otherwise ignores).
    pub fn to_fen(&self, fullmove: usize) -> String {
        let mut fen = String::with_capacity(96);

        // Piece placement, rank 8 down to rank 1.
        for rank in 0..8 {
            let mut empty = 0;
            for file in 0..8 {
                let square = rank * 8 + file;
                let piece = self.mailbox[square];
                if piece == Piece::NONE {
                    empty += 1;
                } else {
                    if empty > 0 {
                        fen.push(char::from_digit(empty as u32, 10).unwrap());
                        empty = 0;
                    }
                    fen.push(char::from(ASCII_PIECES[piece as usize]));
                }
            }
            if empty > 0 {
                fen.push(char::from_digit(empty as u32, 10).unwrap());
            }
            if rank < 7 {
                fen.push('/');
            }
        }

        // Side to move.
        fen.push(' ');
        fen.push(if self.side == White { 'w' } else { 'b' });

        // Castling rights.
        fen.push(' ');
        if self.castle & Castle::Wk != 0 {
            fen.push('K');
        }
        if self.castle & Castle::Wq != 0 {
            fen.push('Q');
        }
        if self.castle & Castle::Bk != 0 {
            fen.push('k');
        }
        if self.castle & Castle::Bq != 0 {
            fen.push('q');
        }
        if self.castle == 0 {
            fen.push('-');
        }

        // En passant target square.
        fen.push(' ');
        if self.enpassant != 0 {
            fen.push_str(SQUARE_TO_COORDINATES[self.enpassant as usize]);
        } else {
            fen.push('-');
        }

        // Halfmove clock and fullmove number.
        fen.push(' ');
        fen.push_str(&self.fifty_mr.to_string());
        fen.push(' ');
        fen.push_str(&fullmove.to_string());

        fen
    }

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::primitives::shared::MoveCode;

    #[test]
    fn test_to_fen_round_trip() {
            let fens = [
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
                "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
                "r3k2r/8/8/8/8/8/8/R3K2R b KQkq - 0 1",
            ];

            for fen in fens {
                let board = BoardPosition::new(fen);
                let out = board.to_fen(1);
                assert_eq!(out, fen, "FEN string mismatch for {}", fen);
                assert_eq!(
                    BoardPosition::new(&out),
                    board,
                    "position mismatch for {}",
                    fen
                );
            }
    }

    #[test]
    fn test_to_fen_en_passant_and_fullmove() {
            // After 1. e4 d5 the en passant target is d6.
            let fen = "rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2";
            let board = BoardPosition::new(fen);
            assert_eq!(board.enpassant, 19); // d6
            assert_eq!(board.to_fen(2), fen);
    }

    #[test]
    fn test_to_fen_castled_king_side() {
            // Castling removes the corresponding rights. After O-O the king moved,
            // so White loses both of its rights; Black keeps both.
            let board =
                BoardPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1");
            let castle_king = Move::create(60, 62, MoveCode::KingCastle);
            let after = board.make_move(castle_king).unwrap();
            assert!(after.castle & Castle::Wk as usize == 0);
            assert!(after.castle & Castle::Wq as usize == 0);
            assert!(after.castle & Castle::Bk as usize != 0);
            assert!(after.castle & Castle::Bq as usize != 0);
            assert_eq!(after.castle, Castle::Bk as usize | Castle::Bq as usize);
    }
}

