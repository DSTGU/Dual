use arrayvec::ArrayVec;

/// Threefold repetition detector
/// Stores a history of position hashes
#[derive(Debug)]
pub struct MoveStack {
    position_command_hashes: Vec<u64>,
    search_position_info: ArrayVec<PositionInfo, 513>
}

impl MoveStack {
    pub fn new() -> Self {
        Self {
            position_command_hashes: Vec::with_capacity(256),
            search_position_info: ArrayVec::new_const()
        }
    }

    /// Clear the repetition table
    pub fn clear(&mut self) {
        self.position_command_hashes.clear();
        self.search_position_info.clear();
    }

    // For position X moves <>
    #[inline(always)]
    pub fn prefill(&mut self, hash: u64) {
        self.position_command_hashes.push(hash);
    }

    /// Push a position onto the history
    #[inline(always)]
    pub fn push(&mut self, hash: u64, static_eval: i32) {
        self.search_position_info.push(PositionInfo { hash: hash, static_eval: static_eval });
    }

    /// Pop the last position from history
    #[inline(always)]
    pub fn pop(&mut self) -> PositionInfo {
        self.search_position_info.pop().expect("can't unmake a move that's not there")
    }

    /// Check if the current position is a draw by repetition
    /// (appears at least 2 times in the history, for a total of 3
    /// including the current position)
    #[inline]
    pub fn is_draw(&self, hash: u64) -> bool {
        let mut count = 0;

        let _start = self.position_command_hashes.len() % 2;
        for &h in self.position_command_hashes.iter() {
            if h == hash {
                count += 1;
                if count >= 2 {
                    // Current occurrence + 2 previous = 3 total
                    return true;
                }
            }
        }

        for &h in self.search_position_info.iter().map(|pos: &PositionInfo| &pos.hash) {
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
    pub fn has_occurred_in_search(&self, hash: u64) -> bool {
        for &h in self.search_position_info.iter().map(|pos| &pos.hash) {
            if h == hash {
                // Current occurrence + 2 previous = 3 total
                return true;
            }
        }
        false
    }

    pub fn is_improving(&self, static_eval: i32) -> bool {
        let second_last = self.search_position_info.iter().rev().nth(1);

        if let Some(sl_pos) = second_last {
            if static_eval > sl_pos.static_eval {
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

#[derive(Debug)]
pub struct PositionInfo {
    hash: u64,
    static_eval: i32,
}