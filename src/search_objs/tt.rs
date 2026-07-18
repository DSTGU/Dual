//! Transposition table implementation with Zobrist hashing
//!
//! This module provides:
//! - Zobrist hash key generation for board positions
//! - Transposition table for storing search results
//! - Threefold repetition detection

use crate::primitives::shared::Move;
use crate::primitives::consts::MATE_THRESHOLD;

/// Size of the transposition table (number of entries)
/// Using a power of 2 allows for fast modulo with bitwise AND
// pub const TT_SIZE: usize = 1 << 20; // ~1 million entries, ~24MB

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
    tt_size: usize
}

impl TranspositionTable {
    pub fn new(hash_size: usize) -> Self {
        let nr_entries = (1024*1024*hash_size/size_of::<TTEntry>()) as u64;
        let nr_entries_pow2 = 1 << (64 - nr_entries.leading_zeros() - 1);

        println!("{} - {}", nr_entries, nr_entries_pow2);

        Self {
            entries: vec![TTEntry::empty(); nr_entries_pow2],
            age: 0,
            tt_size: nr_entries_pow2
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
    fn index(&self, hash: u64) -> usize {
        (hash as usize) & (self.tt_size - 1)
    }

    /// Probe the transposition table
    #[inline]
    pub fn probe(&self, hash: u64) -> Option<&TTEntry> {
        let idx = self.index(hash);
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
        let idx = self.index(hash);
        let entry = &mut self.entries[idx];

        if entry.hash == 0 || !entry.matches(hash) && depth as i32 - entry.depth as i32 + (self.age.wrapping_sub(entry.age) as i32 * 3) > 0 {
            *entry = TTEntry {
                hash,
                depth,
                score,
                flag,
                best_move,
                age: self.age,
            };
        } else if entry.matches(hash) && matches_replacement_strength(depth, flag) >= matches_replacement_strength(entry.depth, entry.flag) {
            let mv = if best_move.is_null() { entry.best_move } else {best_move};

            *entry = TTEntry {
                hash,
                depth,
                score,
                flag,
                best_move: mv,
                age: self.age,
            };
        }
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

#[inline(always)]
pub fn score_from_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_THRESHOLD {
        score - ply as i32
    } else if score <= -MATE_THRESHOLD {
        score + ply as i32
    } else {
        score
    }
}

#[inline(always)]
pub fn score_to_tt(score: i32, ply: usize) -> i32 {
    if score >= MATE_THRESHOLD {
        score + ply as i32
    } else if score <= -MATE_THRESHOLD {
        score - ply as i32
    } else {
        score
    }
}