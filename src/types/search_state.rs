use coarsetime::{Duration, Instant};

use crate::types::board::BoardPosition;
use crate::types::shared::{Move, Piece};
use crate::types::consts::{FIRST_KILLER_BONUS, MAX_HISTORY, MIN_DEPTH, MVV_LVA, SECOND_KILLER_BONUS};
use crate::types::tt::{RepetitionTable, TTEntry, TTFlag, TranspositionTable, score_to_tt};

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    pub max_depth: usize,
    pub seldepth: usize,
    killer_moves: [[Move; 256]; 2],
    //only public for test purposes
    pub history_moves: [[[i32; 64]; 64]; 2],
    //pub capt_history_moves: [[[i32; 64]; 12]; 12], // target, own, captured
    tt: TranspositionTable,
    pub rep_table: RepetitionTable,
    pub nodes: u64,
    deadline: Instant, // change to more universal stop_condition struct
    pub search_start: Instant,
    should_quit: bool,
    pub ply: usize,
}

impl SearchState {
    pub fn new() -> Self {
        Self {
            max_depth: 0,
            seldepth: 0,
            killer_moves: [[Move::create_null(); 256]; 2],
            history_moves: [[[0; 64]; 64]; 2],
            //capt_history_moves: [[[0; 64]; 12]; 12],
            tt: TranspositionTable::new(),
            rep_table: RepetitionTable::new(),
            nodes: 0,
            deadline: Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
            search_start: Instant::now(),
            should_quit: false,
            ply: 0,
        }
    }

    // This function was moved here to preserve TT between nodes
    pub fn change_position(&mut self) {
        self.max_depth = 0;
        self.seldepth = 0;
        self.killer_moves = [[Move::create_null(); 256]; 2];
        self.rep_table.clear();
        self.nodes = 0;
        self.deadline = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
        self.should_quit = false;
        self.ply = 0;
    }

    pub fn clear_persistent_data(&mut self) {
        self.tt.clear();
        self.history_moves = [[[0;64]; 64]; 2];
        //self.capt_history_moves = [[[0; 64]; 12]; 12];
    }

    pub fn reset_for_new_iteration(&mut self, depth: usize) {
        self.max_depth = depth;
        self.seldepth = depth;
        self.tt.increment_age();
    }

    pub fn make_move(&mut self, board_hash: u64) {
        self.rep_table.push(board_hash); 
        self.ply += 1;
    }

    pub fn take_back(&mut self) {
        //take back manages the hash        
        self.rep_table.pop();
        self.ply -= 1;

    }

    #[inline(always)]
    fn get_mvv_lva(victim: Piece, attacker: Piece) -> i32 {
        MVV_LVA[victim as usize % 6 + attacker as usize % 6 * 6]
    }

    fn get_victim(&self, mv: Move) -> Piece {
        mv.get_taken_piece()
    }

    pub fn get_move_score(&self, board_position: &BoardPosition, mv: Move) -> i32 {
        if mv.is_capture() {
            let victim = self.get_victim(mv);
            let mvv = Self::get_mvv_lva(victim, board_position.get_piece(mv));
            
            return mvv;
            //return mvv + 
            //    self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][self.get_piece(mv) as usize][mv.get_target_square() as usize];
        }

        // Killer moves
        if self.ply < 256 {
            if self.killer_moves[0][self.ply] == mv {
                return FIRST_KILLER_BONUS;
            }
            if self.killer_moves[1][self.ply] == mv {
                return SECOND_KILLER_BONUS;
            }
        }

        // History heuristic
        self.history_moves[board_position.side][mv.get_source_square() as usize][mv.get_target_square() as usize]
    }

    pub fn update_killer_move(&mut self, mv: Move) {
        if self.ply < 256 {
            self.killer_moves[1][self.ply] = self.killer_moves[0][self.ply];
            self.killer_moves[0][self.ply] = mv;
        }
    }

    pub fn update_history(&mut self, board_position: &BoardPosition, mv: Move, bonus: i32) {
        let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        let piece = board_position.get_piece(mv) as usize;
        let source = mv.get_source_square() as usize;
        let target = mv.get_target_square() as usize;
        let side = board_position.side;
        if piece < 12 && target < 64 {
            let history_val = self.history_moves[side][source][target];           
            self.history_moves[side][source][target] += clamped_bonus - history_val * clamped_bonus.abs() / MAX_HISTORY //second bonus should be abs
            //if mv.is_capture() {
            //    let history_val = self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][piece][target];
            //    self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][piece][target] += clamped_bonus - history_val * clamped_bonus / MAX_HISTORY;
            //} else {
            //}
            //self.history_moves[piece][target] += bonus;

        }
    }

    // pub fn get_stats(&self) -> (u64, u64, f64) {
    //     let fill_pct = self.tt.fill_percentage();
    //     (self.nodes_searched, self.tt_hits, fill_pct)
    // }

    pub fn is_trifold_repetition(&self, hash: u64) -> bool {
        self.rep_table.is_draw(hash)
    }

    pub fn is_twofold_repetition(&self, hash: u64) -> bool {
        self.rep_table.has_occurred(hash)
    }

    #[inline(always)]
    pub fn probe_tt(&self, hash: u64) -> Option<&TTEntry> {
        self.tt.probe(hash)
    }

    // add static eval
    #[inline(always)]
    pub fn store_tt(
        &mut self,
        depth: u8,
        score: i32,
        flag: TTFlag,
        best_move: Move,
        hash: u64
    ) {
        self.tt.store(
            hash,
            depth,
            score_to_tt(score, self.ply),
            flag,
            best_move, // or .into()
        );
    }

    #[inline(always)]
    pub fn tt_move(&mut self, hash: u64) -> Option<Move> {
        self.tt
            .probe(hash)
            .and_then(|e| {
                if !e.best_move.is_null() {
                    Some(e.best_move)
                } else {
                    None
                }
            })
    }

    pub fn passed_deadline(&self) -> bool {
        Instant::now() > self.deadline
    }

    pub fn should_quit(&mut self, depth: usize) -> bool {
        if self.should_quit {
            return true;
        }

        if self.max_depth > MIN_DEPTH && self.passed_deadline() {
            self.should_quit = true;
            return true;
        }

        false
    }

    pub fn set_deadline(&mut self, deadline: Instant) {
        self.deadline = deadline;
        self.should_quit = false;
    }

}

impl Default for SearchState {
    fn default() -> Self {
        Self::new()
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use crate::gui::{parse_position_command, parse_ucinewgame};
use crate::search::search; 
    use crate::types::board::BoardPosition;
use crate::types::shared::{START_POSITION};
    use crate::types::search_state::{SearchState};

    #[test]
    fn test_clearing_persistent_data_correctly() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new();
                let mut board_position = BoardPosition::new(START_POSITION);
                let empty_history = [[[0; 64]; 64]; 2];
                search(&board_position, &mut search_state, Some(4), None);
                assert_ne!(search_state.history_moves, empty_history);

                board_position = parse_position_command(&mut search_state, "position startpos moves e2e4 e7e5");
                assert_ne!(search_state.history_moves, empty_history);

                search(&board_position, &mut search_state, Some(4), None);
                assert_ne!(search_state.history_moves, empty_history);

                parse_ucinewgame(&mut search_state);
                board_position = parse_position_command(&mut search_state,"position kiwipete");
                assert_eq!(search_state.history_moves, empty_history);

                search(&board_position, &mut search_state, Some(4), None);
                assert_ne!(search_state.history_moves, empty_history);

            })
            .unwrap();
        handler.join().unwrap();



    }
}
