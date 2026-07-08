use crate::types::shared::Piece;

pub const HIDDEN_SIZE: usize = 64;
const SCALE: i32 = 400;
const QA: i16 = 255;
const QB: i16 = 64;

pub static NNUE: Network =
    unsafe { std::mem::transmute(*include_bytes!("../nets/quantised-64.bin")) };

#[inline]
/// Square Clipped ReLU - Activation Function.
/// Note that this takes the i16s in the accumulator to i32s.
/// Range is 0.0 .. 1.0 (in other words, 0 to QA*QA quantized).
fn screlu(x: i16) -> i32 {
    let y = i32::from(x).clamp(0, i32::from(QA));
    y * y
}

/// This is the quantised format that bullet outputs.
#[repr(C)]
pub struct Network {
    /// Column-Major `HIDDEN_SIZE x 768` matrix.
    /// Values have quantization of QA.
    feature_weights: [Accumulator; 768],
    /// Vector with dimension `HIDDEN_SIZE`.
    /// Values have quantization of QA.
    feature_bias: Accumulator,
    /// Column-Major `1 x (2 * HIDDEN_SIZE)`
    /// matrix, we use it like this to make the
    /// code nicer in `Network::evaluate`.
    /// Values have quantization of QB.
    output_weights: [i16; 2 * HIDDEN_SIZE],
    /// Scalar output bias.
    /// Value has quantization of QA * QB.
    output_bias: i16,
}

impl Network {
    /// Calculates the output of the network, starting from the already
    /// calculated hidden layer (done efficiently during makemoves).
    pub fn evaluate(&self, us: &Accumulator, them: &Accumulator) -> i32 {
        // Initialise output.
        let mut output = 0;

        // Side-To-Move Accumulator -> Output.
        for (&input, &weight) in us.vals.iter().zip(&self.output_weights[..HIDDEN_SIZE]) {
            output += screlu(input) * i32::from(weight);
        }

        // Not-Side-To-Move Accumulator -> Output.
        for (&input, &weight) in them.vals.iter().zip(&self.output_weights[HIDDEN_SIZE..]) {
            output += screlu(input) * i32::from(weight);
        }

        // Reduce quantization from QA * QA * QB to QA * QB.
        output /= i32::from(QA);

        // Add bias.
        output += i32::from(self.output_bias);

        // Apply eval scale.
        output *= SCALE;

        // Remove quantisation altogether.
        output /= i32::from(QA) * i32::from(QB);

        output
    }
}

/// A column of the feature-weights matrix.
/// Note the `align(64)`.
#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(C, align(64))]
pub struct Accumulator {
    pub vals: [i16; HIDDEN_SIZE],
}

impl Accumulator {
    /// Initialised with bias so we can just efficiently
    /// operate on it afterwards.
    pub fn new(net: &Network) -> Self {
        net.feature_bias
    }

    /// Add a feature to an accumulator.
    pub fn add_feature(&mut self, feature_idx: usize, net: &Network) {
        for (i, d) in self.vals.iter_mut().zip(&net.feature_weights[feature_idx].vals) {
            *i += *d
        }
    }

    /// Remove a feature from an accumulator.
    pub fn remove_feature(&mut self, feature_idx: usize, net: &Network) {
        for (i, d) in self.vals.iter_mut().zip(&net.feature_weights[feature_idx].vals) {
            *i -= *d
        }
    }
}

pub fn feature_index(piece: Piece, square: usize) -> usize {
    debug_assert!(piece != Piece::NONE);

    piece as usize * 64 + square
}


// #[cfg(test)]
// mod tests {
//     use crate::{nnue::{Accumulator, Network, screlu}, shared::{KIWIPETE, START_POSITION}, types::board::BoardPosition};

    
// pub static NNUE: Network =
//     unsafe { std::mem::transmute(*include_bytes!("../nets/beans.bin")) };

//     #[test]
//     pub fn test_nnue_loading() {
//         println!("{}", std::mem::size_of::<Network>()); 
//         println!("{}", include_bytes!("../nets/rival-256x2.bin").len());
//         assert_eq!(std::mem::size_of::<Network>(), include_bytes!("../nets/rival-256x2.bin").len());
        

//         println!("{:?}", NNUE.feature_bias);
//         println!("{:?}", NNUE.output_bias);
//         println!("{:?}", NNUE.output_weights);

//     }

//         #[test]
//     pub fn test_beans_loading() {  
//         assert_eq!(NNUE.feature_bias.vals[0..16], [176 as i16, 33, 18, 47, 9, 64, 104, -24, 161, 85, 58, 180, 23, 57, 6, 36]);
//         assert_eq!(NNUE.output_bias, 825);
        
//         let mut start_pos : BoardPosition = BoardPosition::new(START_POSITION);
//         start_pos.refresh_nnue(&NNUE);

//         assert_eq!([-1233 as i16, 106, 168, -515, 401, 268, 5, 134, 565, 564, -26, 233, -346, 253, 131, 237], start_pos.accumulators[0].vals[0..16]);
//         assert_eq!([-1233 as i16, 106, 168, -515, 401, 268, 5, 134, 565, 564, -26, 233, -346, 253, 131, 237], start_pos.accumulators[1].vals[0..16]);

//         println!("-------------------------------");
//         let mut kiwipete : BoardPosition = BoardPosition::new(KIWIPETE);
//         kiwipete.refresh_nnue(&NNUE);
    
//         assert_eq!([-1326 as i16, 140, 57, -500, 539, 265, -180, 81, 574, 576, 42, 271, -260, 286, -52, 287], kiwipete.accumulators[0].vals[0..16]);
//         assert_eq!([-1296 as i16, 138, 83, -485, 511, 229, 7, 97, 575, 565, 2, -174, -279, 285, 153, 303], kiwipete.accumulators[1].vals[0..16]);

//     }

//     #[test]
//     pub fn test_empty_acc() {
//         let acc = Accumulator::new(&NNUE);

//         let ev = NNUE.evaluate(&acc, &acc);
//         println!("{}", ev);
//         assert!(ev < 100);
//         assert!(ev > 0);
//     }

//     #[test]
//     pub fn test_validate_screlu() {
//         assert_eq!(0, screlu(-1));    // 0
//         assert_eq!(0, screlu(0));     // 0
//         assert_eq!(1, screlu(1));     // 1
//         assert_eq!(65025, screlu(255));   // 65025
//         assert_eq!(65025, screlu(300));   // 65025
//     }
// }


// beans
// Test positions
// startpos: rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1
// kiwipete: r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1
// Network parameters
// HL=64
// activation=SCReLU
// QA=255, QB=64
// Scale=400
// File IO
// First 16 HL biases: 176 33 18 47 9 64 104 -24 161 85 58 180 23 57 6 36
// Output neuron bias: 825
// Active indices
// startpos (both perspectives): 192 65 130 259 324 133 70 199 8 9 10 11 12 13 14 15 432 433 434 435 436 437 438 439 632 505 570 699 764 573 510 639
// kiwipete (white): 192 324 199 8 9 10 139 140 13 14 15 82 277 407 409 28 35 100 552 489 428 493 430 432 434 435 692 437 566 632 764 639
// kiwipete (black): 632 764 639 432 433 434 563 564 437 438 439 490 685 47 33 420 411 476 144 81 20 85 22 8 10 11 268 13 142 192 324 199
// First 16 accumulator values (pre-activation)
// startpos (both perspectives): -1233 106 168 -515 401 268 5 134 565 564 -26 233 -346 253 131 237
// kiwipete (white): -1326 140 57 -500 539 265 -180 81 574 576 42 271 -260 286 -52 287
// kiwipete (black): -1296 138 83 -485 511 229 7 97 575 565 2 -174 -279 285 153 303
// Unscaled eval without output bias
// startpos: 608404
// kiwipete: -1423747
// Final evaluation
// startpos: 78
// kiwipete: -116