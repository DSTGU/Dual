/// Threefold repetition detector
/// Stores a history of position hashes
#[derive(Debug)]
pub struct MoveStack {
    hashes: Vec<u64>,
}

impl MoveStack {
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
}

impl Default for MoveStack {
    fn default() -> Self {
        Self::new()
    }
}