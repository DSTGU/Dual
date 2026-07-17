use crate::evaluation::nnue::{Accumulator, NNUE, Network, feature_index};
use crate::primitives::board::{BoardPosition};
use crate::primitives::shared::{Color::{self, White}, Move, Piece};

pub struct NetworkState {
    pub accumulators: Vec<[Accumulator; 2]>,
    //pub feature_list: Vec<Feature>
}

impl Default for NetworkState {
    fn default() -> Self {
        NetworkState { 
            accumulators: vec![[Accumulator::new(&NNUE); 2]],
            //feature_list: vec![]
        }
    }
}

impl NetworkState {

    pub fn remove_feature(&mut self, piece: Piece, square: u8, accumulators: &mut [Accumulator;2]) {
        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq as usize);
        accumulators[0].remove_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square as usize);
        accumulators[1].remove_feature(black_feature, &NNUE);
    }


    pub fn add_feature(&mut self, piece: Piece, square: u8, accumulators: &mut [Accumulator;2]) {
        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq as usize);
        accumulators[0].add_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square as usize);
        accumulators[1].add_feature(black_feature, &NNUE);
    }

    pub fn start_board(&mut self, board_position: &BoardPosition, net: &Network) {
        let mut accumulators = [Accumulator::new(net); 2];

        for square in 0..64 {
            //let nnue_square = square ^ 7;
            
            let flipped_sq = square ^ 56;
            let piece = board_position.mailbox[square];

            if piece == Piece::NONE {
                continue;
            }

            // mirrored perspective for white (flip the top 3 bits)
            let feature = feature_index(piece, flipped_sq);

            accumulators[0].add_feature(feature, net);

            let black_feature =
                feature_index(piece.flip_color(), square);

            accumulators[1].add_feature(black_feature, net);
        }

        self.accumulators.clear();
        self.accumulators.push(accumulators);
    }

    // Board state pre move
    pub fn apply_move(&mut self, mv: Move, board_position: &BoardPosition) {
        let piece = board_position.get_piece(mv);
        let new_piece = if mv.get_promoted_piece(board_position.side) == Piece::NONE { piece } else {mv.get_promoted_piece(board_position.side)};
        let mut accumulators = self.accumulators.last().unwrap().clone();

        self.remove_feature(piece, mv.get_source_square(), &mut accumulators);
        self.add_feature(new_piece, mv.get_target_square(), &mut accumulators);

        if mv.is_capture() {
            if mv.is_enpassant() {
                let ep_sq = if board_position.side == White {
                    mv.get_target_square() + 8
                } else {
                    mv.get_target_square() - 8
                };

                self.remove_feature(board_position.get_victim(mv), ep_sq, &mut accumulators);
                //self.feature_list.push(AddSubSub(new_piece, target, piece, source, board_position.get_victim(mv), ep_sq));
            } else {
                self.remove_feature(board_position.get_victim(mv), mv.get_target_square(), &mut accumulators);
                //self.feature_list.push(AddSubSub(new_piece, target, piece, source, board_position.get_victim(mv), target));
            }

        } else if mv.get_castling() {
        // White kingside (O-O): king e1->g1, rook h1->f1
            let (rook_piece, rook_from, rook_to) = match mv.get_target_square() {
                62 => (Piece::R, 63, 61),
                58 => (Piece::R, 56, 59),
                6  => (Piece::r, 7, 5),
                2  => (Piece::r, 0, 3),
                _ => unsafe { std::hint::unreachable_unchecked() }
            };

            self.add_feature(rook_piece, rook_to, &mut accumulators);
            self.remove_feature(rook_piece, rook_from, &mut accumulators);
            //self.feature_list.push(AddSubAddSub(new_piece, target, piece, source, rook_piece, rook_to, rook_piece, rook_from));

        } else {
            //self.feature_list.push(AddSub(new_piece, target, piece, source));
        }

        self.accumulators.push(accumulators);
    }

    pub fn undo_move(&mut self) {
        self.accumulators.pop();
    }

    pub fn evaluate(&self, stm: Color) -> i32 {
        let accumulators= self.accumulators.last().unwrap();
        
        NNUE.evaluate(
            &accumulators[stm],
            &accumulators[stm.invert()],
        )
    }
}

// pub enum Feature {
//     AddSub(Piece, u8, Piece, u8),
//     AddSubSub(Piece, u8, Piece, u8, Piece, u8),
//     AddSubAddSub(Piece, u8, Piece, u8, Piece, u8, Piece, u8)
// }

// impl Feature {

    
// }