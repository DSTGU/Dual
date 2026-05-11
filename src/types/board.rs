use crate::move_gen::{CASTLING_RIGHTS, is_square_attacked};
use crate::shared::{ASCII_PIECES, Castle, KING_INDEX, Move, MoveSuccess, Piece, SQUARE_TO_COORDINATES, get_bit, pop_bit, set_bit};
use crate::types::tt::compute_hash;

#[allow(non_camel_case_types)]
#[allow(unused_variables)]
#[allow(non_upper_case_globals)]
#[derive(Clone, Debug, PartialEq)]
pub struct BoardPosition {
    pub bitboards: [u64; 12],
    pub occupancies: [u64; 3],
    pub mailbox: [Piece; 64],

    // side to move
    pub side: usize, // 0 - W, 1 - B, 2 - Default - none

    // en passant square
    pub enpassant: u8, // Number of square

    // castling rights
    pub castle: usize,

    pub hash: u64
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
            side: 2,
            enpassant: 0,
            castle: 0,
            hash: 0
        };

        board_position.parse_fen(fen);
        board_position.hash = compute_hash(&board_position);

        board_position
    }

    pub fn parse_fen(&mut self, fen: &str) {

        self.bitboards = [0; 12];
        self.occupancies = [0; 3];
        self.mailbox = [Piece::NONE; 64];
        self.side = 2;
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
                    self.add_piece(square, piece);
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


        for piece in 0..=5 {
            self.occupancies[0] |= self.bitboards[piece];
        }

        for piece in 6..=11 {
            self.occupancies[1] |= self.bitboards[piece];
        }

        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];
    }


    pub fn remove_piece(&mut self, square: usize, piece: Piece) {
        debug_assert!(self.mailbox[square] == piece);
        debug_assert!(get_bit(self.bitboards[piece as usize], square));

        self.mailbox[square] = Piece::NONE;
        pop_bit(&mut self.occupancies[piece.get_side()], square);
        pop_bit(&mut self.bitboards[piece as usize], square);
    }

    pub fn add_piece(&mut self, square: usize, piece: Piece) {
        self.mailbox[square] = piece;
        set_bit(&mut self.occupancies[piece.get_side()], square);
        set_bit(&mut self.bitboards[piece as usize], square);
    }

    #[inline(always)]
    pub fn find_capture_at_square(&self, square: usize) -> Piece {
        self.mailbox[square]
    }

    /// Apply `move_to_make` to `board` and return the resulting position, or
    /// `None` if the move leaves the king in check.
    pub fn make_move(&mut self, move_to_make: Move) -> MoveSuccess {
        // println!("{:?}, direction {:?}", move_to_make, move_direction);

        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        
        let piece = self.mailbox[source];
        let is_capture = move_to_make.is_capture();
        let is_enpassant = move_to_make.is_enpassant();
        let is_castling = move_to_make.get_castling();
        let is_double_push = move_to_make.get_double_pawn_push();
        let promoted = move_to_make.is_promotion();

        // Handle captures: 
        if is_capture && !is_enpassant {
            self.remove_piece(target, self.mailbox[target]);
        }

        self.remove_piece(source, piece);
        self.add_piece(target, piece);

        // Handle promotion: replace the pawn with the promoted piece.
        if promoted {
            self.remove_piece(target, piece);
            self.add_piece(target, move_to_make.get_promoted_piece(self.side != 0));
        }

        // Handle en passant: remove the captured pawn (which is on a different
        // square from the target).
        if is_enpassant {
            let ep_sq = if self.side == 0 {
                target + 8
            } else {
                target - 8
            };

            let pawn = if self.side == 0 {
                Piece::p
            } else {
                Piece::P
            };

            self.remove_piece(ep_sq, pawn);
        }

        // Reset en passant square; set it again if this was a double pawn push.
        self.enpassant = 0;

        if is_double_push {
            self.enpassant = if self.side == 0 {
                target as u8 + 8
            } else {
                target as u8 - 8
            };
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

            self.remove_piece(rook_from, rook_piece);
            self.add_piece(rook_to, rook_piece);
        }

        // Update castling rights.
        self.castle &= CASTLING_RIGHTS[source] as usize;
        self.castle &= CASTLING_RIGHTS[target] as usize;

        // Recompute occupancies.
        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];

        // Find the king of the side that just moved to check for legality.
        let king_sq =
            self.bitboards[KING_INDEX[self.side]].trailing_zeros() as u8;

        if is_square_attacked(king_sq, &self) {
            self.side = 1 - self.side;
            self.take_back(move_to_make);    
            return MoveSuccess::Attacked;
        }

        // Flip side for the returned position.
        self.side = 1 - self.side;
        MoveSuccess::Success
    }


    /// Apply `move_to_make` to `board` and return the resulting position, or
    /// `None` if the move leaves the king in check.
    pub fn take_back(&mut self, move_to_make: Move) -> MoveSuccess {

        self.side = 1 - self.side;
        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        let piece = self.mailbox[target];
        let is_capture = move_to_make.is_capture();
        let is_enpassant = move_to_make.is_enpassant();
        let is_castling: bool = move_to_make.get_castling();
        let promoted = move_to_make.get_promoted_piece(self.side != 0);
        let taken_piece = move_to_make.get_taken_piece();

        // if source == 27 && target == 19 || self.mailbox[59] == Piece::K {
        //     println!("Move to take back:");
        //     println!("{:?}", move_to_make);
        //     self.print_board();
        // }

        if move_to_make.is_promotion() {
            self.remove_piece(target, promoted);
            let pawn = if self.side == 0 {Piece::P} else {Piece::p};
            self.add_piece(source,  pawn);
        } else {
            // Normal move
            self.remove_piece(target, piece);
            self.add_piece(source, piece);
        }

        if is_capture && !is_enpassant {
            self.add_piece(target, taken_piece);
        }

        // Handle en passant: remove the captured pawn (which is on a different
        // square from the target).
        if is_enpassant {
            if (piece as usize) < 6 {
                // White pawn captured a black pawn on the rank below target.
                self.add_piece(target+8, Piece::p);

            } else {
                // Black pawn captured a white pawn on the rank above target.
                self.add_piece(target-8, Piece::P);
            }
        }

        // Reset en passant square
        self.enpassant = move_to_make.get_old_ep_square();

        // Handle castling: move the rook.
        if is_castling {
            let (rook_piece, rook_from, rook_to) = match target {
                62 => (Piece::R, 63, 61),
                58 => (Piece::R, 56, 59),
                6  => (Piece::r, 7, 5),
                2  => (Piece::r, 0, 3),
                _ => unsafe { std::hint::unreachable_unchecked() }
            };

            self.add_piece(rook_from, rook_piece);
            self.remove_piece(rook_to, rook_piece);            
        }
            // Update castling rights.
        self.castle = move_to_make.get_old_castle();

        // Recompute occupancies.
        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];

        MoveSuccess::Success
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
                    output = output + &format!("  {} ", 8 - rank);
                }

                // define piece variable
                let mut piece = 12 as usize;

                // loop over all piece bitboards
                for bb_piece in 0..12
                {
                    if get_bit(self.bitboards[bb_piece], square) {
                        piece = bb_piece;
                    }
                }

                if piece == 12
                {
                    output = output + " .";
                }
                else {
                    output = output + &format!(" {}", char::from(ASCII_PIECES[piece]));
                }
            }

        // print new line every rank
            output = output + "\n";
        }

        // print board files
        output = output + "\n     a b c d e f g h\n\n";

        match self.side {
            0 => output = output + "White\n",
            1 => output = output + "Black\n",
            _ => output = output + "No side\n",
        }

        match self.enpassant {
            0 => output = output + "Enpassant not available\n",
            _ =>  output = output + &format!("Enpassant: {}\n", SQUARE_TO_COORDINATES[self.enpassant as usize]),
        }


        // print castling rights

        if self.castle & Castle::Wk != 0
        {
            output = output + "K";
        }
        if self.castle & Castle::Wq != 0
        {
            output = output + "Q";
        }
        if self.castle & Castle::Bk != 0
        {
            output = output + "k";
        }
        if self.castle & Castle::Bq != 0
        {
            output = output + "q";
        }
        output = output + "\n";

        output
    }

    pub fn print_board(&self) {
        println!("{}", self.format_board());
    }

}
