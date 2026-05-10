use crate::move_gen::{CASTLING_RIGHTS, is_square_attacked};
use crate::shared::{ASCII_PIECES, Castle, KING_INDEX, Move, MoveSuccess, Piece, SQUARE_TO_COORDINATES, get_bit, pop_bit, set_bit};

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
    pub enpassant: usize, // Number of square

    // castling rights
    pub castle: usize
}
    /*
    binary encoding
    0001    1  white king can castle to the king side
    0010    2  white king can castle to the queen side
    0100    4  black king can castle to the king side
    1000    8  black king can castle to the queen side
    */

impl BoardPosition {

    pub fn remove_piece(&mut self, square: usize, piece: Piece) {
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
        let is_capture = move_to_make.get_capture();
        let is_enpassant = move_to_make.get_enpassant();
        let is_castling = move_to_make.get_castling();
        let is_double_push = move_to_make.get_double_pawn_push();
        let promoted = move_to_make.get_promoted_piece();

        if piece == Piece::NONE || (move_to_make.get_taken_piece() == Piece::NONE && is_capture && !is_enpassant) {
            self.print_board();
            println!("Will oob eventually");
        }

        // Handle captures: 
        if is_capture && !is_enpassant {
            self.remove_piece(target, self.mailbox[target]);
        }

        // Move the piece: clear source, set target
        self.remove_piece(source, piece);
        self.add_piece(target, piece);

        // Handle promotion: replace the pawn with the promoted piece.
        if promoted != Piece::NONE {
            self.remove_piece(target, piece);
            self.add_piece(target, promoted);
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
        self.enpassant = 64;

        if is_double_push {
            self.enpassant = if self.side == 0 {
                target + 8
            } else {
                target - 8
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
            self.bitboards[KING_INDEX[self.side]].trailing_zeros() as usize;

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
        let piece = move_to_make.get_piece();
        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        let is_capture = move_to_make.get_capture();
        let is_enpassant = move_to_make.get_enpassant();
        let is_castling = move_to_make.get_castling();
        let promoted = move_to_make.get_promoted_piece();
        let taken_piece = move_to_make.get_taken_piece();

        if promoted != Piece::NONE {
            self.remove_piece(target, promoted);
            self.add_piece(source, piece);
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
        self.enpassant = move_to_make.get_old_ep_square() as usize;

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
        //board.occupancies[0] = board.bitboards[0..6].iter().fold(0, |acc, &b| acc | b);
        //board.occupancies[1] = board.bitboards[6..12].iter().fold(0, |acc, &b| acc | b);
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
            64 => output = output + "Enpassant not available\n",
            65 => output = output + "Enpassant not available\n",
            _ =>  output = output + &format!("Enpassant: {}\n", SQUARE_TO_COORDINATES[self.enpassant]),
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
