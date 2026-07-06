//! Transposition table implementation with Zobrist hashing
//!
//! This module provides:
//! - Zobrist hash key generation for board positions
//! - Transposition table for storing search results
//! - Threefold repetition detection

use std::sync::OnceLock;

use crate::{shared::{Move}, types::board::BoardPosition};

/// Size of the transposition table (number of entries)
/// Using a power of 2 allows for fast modulo with bitwise AND
pub const TT_SIZE: usize = 1 << 20; // ~1 million entries, ~24MB

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
    if board.side == 1 {
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

/// Transposition table entry types
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TTFlag {
    Exact, // Score is exact
    Alpha, // Score is upper bound (fail low)
    Beta,  // Score is lower bound (fail high)
}

/// Transposition table entry
#[derive(Clone, Copy)]
pub struct TTEntry {
    pub hash: u64,      // Full hash for verification
    pub score: i32,     // Evaluated score
    pub best_move: Move, // Best move found (if any)
    pub depth: u8,     // Search depth
    pub age: u8,        // Search age for replacement
    pub flag: TTFlag,   // Type of score
}

impl TTEntry {
    pub const fn empty() -> Self {
        Self {
            hash: 0,
            depth: 0,
            score: 0,
            flag: TTFlag::Exact,
            best_move: Move::create_null(),
            age: 0,
        }
    }

    /// Check if this entry is valid for the given hash
    #[inline(always)]
    pub fn matches(&self, hash: u64) -> bool {
        self.hash == hash
    }
}

/// Transposition table using fixed-size array
/// Using a simple direct indexing scheme for speed
pub struct TranspositionTable {
    entries: Vec<TTEntry>,
    age: u8,
}

impl TranspositionTable {
    pub fn new() -> Self {
        Self {
            entries: vec![TTEntry::empty(); TT_SIZE],
            age: 0
        }
    }

    /// Clear the transposition table
    pub fn clear(&mut self) {
        for entry in &mut self.entries {
            *entry = TTEntry::empty();
        }
    }

    /// Increment the search age
    pub fn increment_age(&mut self) {
        self.age = self.age.wrapping_add(1);
    }

    /// Get index into the table from hash
    #[inline(always)]
    fn index(hash: u64) -> usize {
        (hash as usize) & (TT_SIZE - 1)
    }

    /// Probe the transposition table
    #[inline]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = Self::index(hash);
        let entry = &self.entries[idx];

        if entry.matches(hash) {
            Some(entry)
        } else {
            None
        }
    }

    /// Store an entry in the transposition table
    #[inline]
    pub fn store(&mut self, hash: u64, depth: u8, score: i32, flag: TTFlag, best_move: Move) {
        let idx = Self::index(hash);
        let entry = &mut self.entries[idx];

        // Replacement strategy:
        // 1. Always replace if entry is empty or from different position
        // 2. Replace if new search is deeper
        // 3. Replace if same depth but from older search

        if entry.hash == 0
            || entry.matches(hash) && matches_replacement_strength(depth, flag) >= matches_replacement_strength(entry.depth, entry.flag) // || (flag == TTFlag::Exact && entry.flag != TTFlag::Exact)
            || !entry.matches(hash) && depth as i32 - entry.depth as i32 + (self.age.wrapping_sub(entry.age) as i32 * 3) > 0 {
            *entry = TTEntry {
                hash,
                depth,
                score,
                flag,
                best_move,
                age: self.age,
            };
        }
    }

    /// Calculate hash fill percentage
    pub fn fill_percentage(&self) -> f64 {
        let used = self.entries.iter().filter(|e| e.hash != 0).count();
        (used as f64 / TT_SIZE as f64) * 100.0
    }
}

    #[inline]
    pub fn matches_replacement_strength(depth: u8, flag: TTFlag) -> u8 {
        depth + if flag == TTFlag::Exact {
            1
        } else {
            0
        }
    }

impl Default for TranspositionTable {
    fn default() -> Self {
        Self::new()
    }
}


/// Threefold repetition detector
/// Stores a history of position hashes
#[derive(Debug)]
pub struct RepetitionTable {
    hashes: Vec<u64>,
}

impl RepetitionTable {
    pub fn new() -> Self {
        Self {
            hashes: Vec::with_capacity(256),
        }
    }

    /// Clear the repetition table
    pub fn clear(&mut self) {
        self.hashes.clear();
    }

    /// Push a hash onto the history
    #[inline(always)]
    pub fn push(&mut self, hash: u64) {
        self.hashes.push(hash);
    }

    /// Pop the last hash from history
    #[inline(always)]
    pub fn pop(&mut self) -> u64 {
        self.hashes.pop().expect("can't unmake a move that's not there")
    }

    /// Check if the current position is a draw by repetition
    /// (appears at least 2 times in the history, for a total of 3
    /// including the current position)
    #[inline]
    pub fn is_draw(&self, hash: u64) -> bool {
        let mut count = 0;
        // Only check positions where the same side is to move.
        // hashes[i] stores the hash of the position before move i was made,
        // so hashes[i] has the same side to move as the current position
        // when i and self.hashes.len() have the same parity.
        let start = self.hashes.len() % 2;
        for &h in self.hashes.iter().skip(start).step_by(2) {
            if h == hash {
                count += 1;
                if count >= 2 {
                    // Current occurrence + 2 previous = 3 total
                    return true;
                }
            }
        }
        false
    }

    /// Check if position has occurred at least once before
    /// (for detecting twofold repetition)
    pub fn has_occurred(&self, hash: u64) -> bool {
        let start = self.hashes.len() % 2;
        for &h in self.hashes.iter().skip(start).step_by(2) {
            if h == hash {
                return true;
            }
        }
        false
    }

    /// Get the number of times a hash has occurred
    pub fn count_occurrences(&self, hash: u64) -> usize {
        self.hashes.iter().filter(|&&h| h == hash).count()
    }

    /// Get the current ply (half-move count)
    pub fn ply(&self) -> usize {
        self.hashes.len()
    }
}

impl Default for RepetitionTable {
    fn default() -> Self {
        Self::new()
    }
}