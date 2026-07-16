use crate::{nnue::{Accumulator, NNUE, Network, feature_index}, types::{board::BoardPosition, shared::{Color, Move, Piece}}};



pub struct NetworkState {
    pub accumulators: [Accumulator; 2]
}

impl Default for NetworkState {
    fn default() -> Self {
        NetworkState { accumulators: [Accumulator::new(&NNUE); 2]}
    }
}


impl NetworkState {

    pub fn remove_feature(&mut self, piece: Piece, square: u8) {
        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq as usize);
        self.accumulators[0].remove_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square as usize);
        self.accumulators[1].remove_feature(black_feature, &NNUE);
    }


    pub fn add_feature(&mut self, piece: Piece, square: u8) {
        let flipped_sq = square ^ 56;

        // mirrored perspective for black (flip the top 3 bits)
        let feature = feature_index(piece, flipped_sq as usize);
        self.accumulators[0].add_feature(feature, &NNUE);

        let black_feature =
            feature_index(piece.flip_color(), square as usize);
        self.accumulators[1].add_feature(black_feature, &NNUE);
    }

    pub fn start_board(&mut self, board_position: &BoardPosition, net: &Network) {
        self.accumulators[0] = Accumulator::new(net);
        self.accumulators[1] = Accumulator::new(net);

        for square in 0..64 {
            //let nnue_square = square ^ 7;
            
            let flipped_sq = square ^ 56;
            let piece = board_position.mailbox[square];

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

    pub fn apply_move(&mut self, mv: Move, board_position: &BoardPosition) {
        if mv.get_castling() {

        }

        if mv.is_capture() {

        }

        
    }

    pub fn evaluate(&self, stm: Color) -> i32 {
        NNUE.evaluate(
            &self.accumulators[stm],
            &self.accumulators[stm.invert()],
        )
    }
}