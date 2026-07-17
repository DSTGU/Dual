use std::sync::OnceLock;

use crate::primitives::{board::BoardPosition, shared::Color::Black};

/// Zobrist hash keys for position hashing
/// Generated once and cached using OnceLock
pub struct ZobristKeys {
    /// Keys for each piece on each square [piece][square]
    pub piece_keys: [[u64; 64]; 12],
    /// Key for side to move
    pub side_key: u64,
    /// Keys for castling rights [4 rights]
    pub castling_keys: [u64; 4],
    /// Keys for en passant file [8 files]
    pub enpassant_keys: [u64; 8],
}

impl ZobristKeys {
    /// Generate deterministic pseudo-random keys using xorshift
    fn generate() -> Self {
        let mut seed: u64 = 0xF0E1D2C3B4A59687;

        fn xorshift64(state: &mut u64) -> u64 {
            *state ^= *state << 13;
            *state ^= *state >> 7;
            *state ^= *state << 17;
            *state
        }

        let mut piece_keys = [[0u64; 64]; 12];
        for piece in 0..12 {
            for square in 0..64 {
                piece_keys[piece][square] = xorshift64(&mut seed);
            }
        }

        let side_key = xorshift64(&mut seed);

        let mut castling_keys = [0u64; 4];
        for i in 0..4 {
            castling_keys[i] = xorshift64(&mut seed);
        }

        let mut enpassant_keys = [0u64; 8];
        for i in 0..8 {
            enpassant_keys[i] = xorshift64(&mut seed);
        }

        Self {
            piece_keys,
            side_key,
            castling_keys,
            enpassant_keys,
        }
    }
}

/// Global Zobrist keys - initialized once on first access
static ZOBRIST_KEYS: OnceLock<ZobristKeys> = OnceLock::new();

/// Get the global Zobrist keys, initializing if necessary
#[inline(always)]
pub fn get_zobrist_keys() -> &'static ZobristKeys {
    ZOBRIST_KEYS.get_or_init(ZobristKeys::generate)
}

/// Compute the Zobrist hash for a board position
/// This is a full re-computation - for incremental updates during search,
/// use update_hash functions
pub fn compute_hash(board: &BoardPosition) -> u64 {
    let keys = get_zobrist_keys();
    let mut hash: u64 = 0;

    // Hash pieces
    for piece in 0..12 {
        let mut bb = board.bitboards[piece];
        while bb != 0 {
            let sq = bb.trailing_zeros() as usize;
            hash ^= keys.piece_keys[piece][sq];
            bb &= bb - 1; // Clear LSB
        }
    }

    // Hash side to move
    if board.side == Black {
        hash ^= keys.side_key;
    }

    // Hash castling rights
    if board.castle & 1 != 0 {
        hash ^= keys.castling_keys[0];
    }
    if board.castle & 2 != 0 {
        hash ^= keys.castling_keys[1];
    }
    if board.castle & 4 != 0 {
        hash ^= keys.castling_keys[2];
    }
    if board.castle & 8 != 0 {
        hash ^= keys.castling_keys[3];
    }

    // Hash en passant
    if board.enpassant != 0 {
        let file = board.enpassant % 8;
        hash ^= keys.enpassant_keys[file as usize];
    }

    hash
}