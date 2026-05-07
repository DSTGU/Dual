use crate::move_gen::is_square_attacked;
use crate::shared::{BoardPosition, FIRST_KILLER_BONUS, MVV_LVA, Move, PV_MOVE_BONUS, SECOND_KILLER_BONUS, START_POSITION, get_bit, parse_fen};
use crate::tt::{RepetitionTable, TranspositionTable, compute_hash};

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    board_position: BoardPosition,
    pub max_depth: usize,
    killer_moves: [[u32; 256]; 2],
    history_moves: [[usize; 64]; 12],
    prev_iter_best_move: u32,
    tt: TranspositionTable,
    rep_table: RepetitionTable,
    nodes_searched: u64,
    tt_hits: u64,
}

impl SearchState {
    pub fn new(board_position: BoardPosition) -> Self {
        let mut search_state = Self {
            board_position: board_position,
            max_depth: 0,
            killer_moves: [[0; 256]; 2],
            history_moves: [[0; 64]; 12],
            prev_iter_best_move: 0,
            tt: TranspositionTable::new(),
            rep_table: RepetitionTable::new(),
            nodes_searched: 0,
            tt_hits: 0,
        };

        search_state.rep_table.push_position(&board_position);

        search_state
    }

    // pub fn reset(&mut self, depth: usize) {
    //     self.board_position = None;
    //     self.max_depth = depth;
    //     self.killer_moves = [[0; 256]; 2];
    //     self.history_moves = [[0; 64]; 12];
    //     self.prev_iter_best_move = 0;
    //     self.tt.clear();
    //     self.rep_table.clear();
    //     self.nodes_searched = 0;
    //     self.tt_hits = 0;
    // }

    // pub fn reset_for_new_search(&mut self, depth: usize, board_position: BoardPosition) {
    //     self.board_position = board_position;
    //     self.max_depth = depth;
    //     self.prev_iter_best_move = 0;
    //     self.tt.increment_age();
    //     self.rep_table.clear();
    //     self.nodes_searched = 0;
    //     // Don't clear TT - keep entries from previous searches
    // }

    // pub fn reset_for_new_search_with_moves(&mut self, depth: usize, board_position: BoardPosition, moves: Vec<Move>) {
    //     self.board_position = board_position;
    //     self.max_depth = depth;
    //     self.prev_iter_best_move = 0;
    //     self.tt.increment_age();
    //     self.rep_table.clear();
    //     self.nodes_searched = 0;
    // }

    pub fn reset_for_new_search(&mut self, depth: usize) {
        self.max_depth = depth;
        self.prev_iter_best_move = 0;
        self.tt.increment_age();
        self.killer_moves = [[0; 256]; 2];
        self.history_moves = [[0; 64]; 12];
        self.nodes_searched = 0;
    }

    pub fn make_move_for_state(&mut self, board_position: BoardPosition) {
        self.board_position = board_position;
        self.rep_table.push_position(&board_position);
        // TODO: Implement TT
    }

    pub fn take_back_for_state(&mut self, board_position: BoardPosition) {
        self.board_position = board_position;
        self.rep_table.pop();
        // TODO: Implement TT
    }

    #[inline(always)]
    fn get_mvv_lva(victim: usize, attacker: usize) -> usize {
        MVV_LVA[victim % 6 + attacker % 6 * 6]
    }

    fn get_victim(&self, mv: &Move) -> usize {
        let opponent_side = self.board_position.side ^ 1;
        let start_idx = opponent_side * 6;

        for i in start_idx..start_idx + 6 {
            if get_bit(self.board_position.bitboards[i], mv.get_target_square() as usize) {
                return i;
            }
        }
        0
    }

    pub fn get_move_score(&self, mv: &Move, ply: usize) -> usize {
        // PV move from previous iteration gets highest priority
        if ply == 0 && mv.mv == self.prev_iter_best_move {
            return PV_MOVE_BONUS;
        }

        if mv.get_capture() {
            let victim = self.get_victim(mv);
            return Self::get_mvv_lva(victim, mv.get_piece() as usize);
        }

        // Killer moves
        if self.killer_moves[0][ply] == mv.mv {
            return FIRST_KILLER_BONUS;
        }
        if self.killer_moves[1][ply] == mv.mv {
            return SECOND_KILLER_BONUS;
        }

        // History heuristic
        self.history_moves[mv.get_piece() as usize][mv.get_target_square() as usize]
    }

    pub fn update_killer_move(&mut self, mv: &Move, ply: usize) {
        let idx = self.max_depth.saturating_sub(ply);
        if idx < 256 {
            self.killer_moves[1][idx] = self.killer_moves[0][idx];
            self.killer_moves[0][idx] = mv.mv;
        }
    }

    pub fn update_history(&mut self, mv: &Move, depth: usize) {
        let piece = mv.get_piece() as usize;
        let target = mv.get_target_square() as usize;
        if piece < 12 && target < 64 {
            self.history_moves[piece][target] += depth;
        }
    }

    // pub fn get_stats(&self) -> (u64, u64, f64) {
    //     let fill_pct = self.tt.fill_percentage();
    //     (self.nodes_searched, self.tt_hits, fill_pct)
    // }

    pub fn get_board_position(&self) -> BoardPosition {
        self.board_position
    }

    pub fn is_trifold_repetition(&self) -> bool {
        self.rep_table.is_draw(compute_hash(&self.get_board_position()))
    }

    pub fn is_king_attacked(&self) -> bool {
        is_square_attacked(self.get_board_position().bitboards[6*self.get_board_position().side+5].trailing_zeros() as usize, &self.get_board_position())
    }

}

impl Default for SearchState {
    fn default() -> Self {
        Self::new(parse_fen(START_POSITION))
    }
}