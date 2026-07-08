use crate::move_gen::{CASTLING_RIGHTS, is_square_attacked};
use crate::nnue::{Accumulator, HIDDEN_SIZE, NNUE, Network, feature_index};
use crate::types::shared::Color::{Black, White};
use crate::types::shared::{ASCII_PIECES, Castle, Color, KING_INDEX, Move, MoveSuccess, Piece, SQUARE_TO_COORDINATES, get_bit, pop_bit, set_bit};
use crate::types::tt::{compute_hash, get_zobrist_keys};

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

    pub accumulators: [Accumulator; 2],
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
            accumulators: [Accumulator{ vals: [0; HIDDEN_SIZE]}; 2],
        };

        board_position.parse_fen(fen);
        board_position.hash = compute_hash(&board_position);
        board_position.refresh_nnue(&NNUE);

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
        // fen_chars.next(); // skip space

        // let mut word = String::new();

        // while let Some(c) = fen_chars.next() {
        //     if c.is_whitespace() {
        //         break;
        //     }

        //     word.push(c);
        // }

        // self.fifty_mr = word.parse().unwrap_or(0);

        for piece in 0..=5 {
            self.occupancies[0] |= self.bitboards[piece];
        }

        for piece in 6..=11 {
            self.occupancies[1] |= self.bitboards[piece];
        }

        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];
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

        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq);
        self.accumulators[0].remove_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square);
        self.accumulators[1].remove_feature(black_feature, &NNUE);
    }

    #[inline(always)]
    pub fn add_piece(&mut self, square: usize, piece: Piece, update_hash: bool) {
        if update_hash {
            self.hash ^= get_zobrist_keys().piece_keys[piece as usize][square];
        }

        self.mailbox[square] = piece;
        set_bit(&mut self.occupancies[piece.get_side()], square);
        set_bit(&mut self.bitboards[piece as usize], square);

        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq);
        self.accumulators[0].add_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square);
        self.accumulators[1].add_feature(black_feature, &NNUE);
    }

    #[inline(always)]
    pub fn find_capture_at_square(&self, square: usize) -> Piece {
        self.mailbox[square]
    }

    /// Apply `move_to_make` to `board` and return the resulting position, or
    /// `None` if the move leaves the king in check.
    pub fn make_move(&mut self, move_to_make: Move) -> MoveSuccess {

        let keys = get_zobrist_keys();
        let old_hash = self.hash;

        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        
        let piece = self.mailbox[source];
        let is_capture = move_to_make.is_capture();
        let is_enpassant = move_to_make.is_enpassant();
        let is_castling = move_to_make.get_castling();
        let is_double_push = move_to_make.get_double_pawn_push();
        let promoted = move_to_make.is_promotion();

        // //handle 50mr
        // if is_capture || piece == Piece::P || piece == Piece::p {
        //     self.fifty_mr += 1;
        // } else {
        //     self.fifty_mr = 0;
        // }

        // Handle captures: 
        if is_capture && !is_enpassant {
            self.remove_piece(target, self.mailbox[target], true);
        }

        self.remove_piece(source, piece, true);
        self.add_piece(target, piece, true);

        // Handle promotion: replace the pawn with the promoted piece.
        if promoted {
            self.remove_piece(target, piece, true);
            self.add_piece(target, move_to_make.get_promoted_piece(self.side), true);
        }

        // Handle en passant: remove the captured pawn (which is on a different
        // square from the target).
        if is_enpassant {
            let ep_sq = if self.side == White {
                target + 8
            } else {
                target - 8
            };

            let pawn = if self.side == White {
                Piece::p
            } else {
                Piece::P
            };

            self.remove_piece(ep_sq, pawn, true);
        }

        if self.enpassant != 0 {
            self.hash ^= keys.enpassant_keys[(self.enpassant % 8) as usize];
        }

        // Reset en passant square; set it again if this was a double pawn push.
        self.enpassant = 0;

        if is_double_push {
            self.enpassant = if self.side == White {
                target as u8 + 8
            } else {
                target as u8 - 8
            };
        }

        if self.enpassant != 0 {
            self.hash ^= keys.enpassant_keys[(self.enpassant % 8) as usize];
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

            self.remove_piece(rook_from, rook_piece, true);
            self.add_piece(rook_to, rook_piece, true);
        }

        for i in 0..4 {
            if self.castle & (1 << i) != 0 {
                self.hash ^= keys.castling_keys[i];
            }
        }

        // Update castling rights.
        self.castle &= CASTLING_RIGHTS[source] as usize;
        self.castle &= CASTLING_RIGHTS[target] as usize;

        for i in 0..4 {
            if self.castle & (1 << i) != 0 {
                self.hash ^= keys.castling_keys[i];
            }
        }

        // Recompute occupancies.
        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];

        // Find the king of the side that just moved to check for legality.
        let king_sq =
            self.bitboards[KING_INDEX[self.side]].trailing_zeros() as u8;

        if is_square_attacked(king_sq, self) {
            self.side = self.side.invert();
            self.take_back(move_to_make, old_hash);    
            return MoveSuccess::Attacked;
        }

        // Flip side for the returned position.
        self.side = self.side.invert();
        self.hash ^= keys.side_key;

        MoveSuccess::Success
    }


    /// Apply `move_to_make` to `board` and return the resulting position, or
    /// `None` if the move leaves the king in check.
    pub fn take_back(&mut self, move_to_make: Move, old_hash: u64) -> MoveSuccess {

        self.side = self.side.invert();
        let source = move_to_make.get_source_square() as usize;
        let target = move_to_make.get_target_square() as usize;
        let piece = self.mailbox[target];
        let is_capture = move_to_make.is_capture();
        let is_enpassant = move_to_make.is_enpassant();
        let is_castling: bool = move_to_make.get_castling();
        let promoted = move_to_make.get_promoted_piece(self.side);
        let taken_piece = move_to_make.get_taken_piece();

        if move_to_make.is_promotion() {
            self.remove_piece(target, promoted, false);
            let pawn = if self.side == White {Piece::P} else {Piece::p};
            self.add_piece(source,  pawn, false);
        } else {
            // Normal move
            self.remove_piece(target, piece, false);
            self.add_piece(source, piece, false);
        }

        if is_capture && !is_enpassant {
            self.add_piece(target, taken_piece, false);
        }

        // Handle en passant: remove the captured pawn (which is on a different
        // square from the target).
        if is_enpassant {
            if (piece as usize) < 6 {
                // White pawn captured a black pawn on the rank below target.
                self.add_piece(target+8, Piece::p, false);

            } else {
                // Black pawn captured a white pawn on the rank above target.
                self.add_piece(target-8, Piece::P, false);
            }
        }

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

            self.add_piece(rook_from, rook_piece, false);
            self.remove_piece(rook_to, rook_piece, false);            
        }
            // Update castling rights.
        self.castle = move_to_make.get_old_castle();

        // Recompute occupancies.
        self.occupancies[2] = self.occupancies[0] | self.occupancies[1];

        self.hash = old_hash;

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

    pub fn refresh_nnue(&mut self, net: &Network) {
        self.accumulators[0] = Accumulator::new(net);
        self.accumulators[1] = Accumulator::new(net);

        for square in 0..64 {
            //let nnue_square = square ^ 7;
            
            let flipped_sq = square ^ 56;
            let piece = self.mailbox[square];

            if piece == Piece::NONE {
                continue;
            }

            // mirrored perspective for white (flip the top 3 bits)
            let feature = feature_index(piece, flipped_sq);

            self.accumulators[0].add_feature(feature, net);

            let black_feature =
                feature_index(piece.flip_color(), square);

            self.accumulators[1].add_feature(black_feature, net);
        }
    }
}
