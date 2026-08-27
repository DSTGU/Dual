use coarsetime::{Instant};

use crate::primitives::board::BoardPosition;
use crate::tunable::{max_history, tm_alloc_div, tm_hard_percent, tm_inc_scale, tm_soft_div};
use crate::primitives::shared::{Color, Move, Piece};
use crate::primitives::consts::MVV_LVA;
use crate::search_objs::config::EngineConfig;
use crate::search_objs::move_stack::MoveStack;
use crate::search_objs::pv_table::PrincipalVariationTable;
use crate::search_objs::search_state::Reporting::UCI;
use crate::search_objs::tt::{TTEntry, TTFlag, TranspositionTable, score_to_tt};
use crate::evaluation::network_state::NetworkState;

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    pub max_depth: usize, // Of the search iteration, not in general
    pub seldepth: usize,
    pub killer_moves: Box<[Move; 256]>,
    //only public for test purposes
    pub history_moves: Box<[[[i16; 64]; 64]; 2]>,
    //pub capt_history_moves: [[[i32; 64]; 12]; 12], // target, own, captured
    tt: TranspositionTable,
    pub move_stack: Box<MoveStack>,
    pub nodes: u64,
    pub stop_condition: StopCondition,
    should_quit: bool,
    pub ply: usize,
    pub network_state: NetworkState,
    pub pv_table: Box<PrincipalVariationTable>,
    pub engine_config: EngineConfig,
    pub reporting: Reporting,
    //pub search_stage: SearchStage,
}

impl SearchState {
    pub fn new(config: &EngineConfig) -> Self {
        Self {
            max_depth: 0,
            seldepth: 0,
            killer_moves: unsafe { Box::new_zeroed().assume_init() },
            history_moves: unsafe { Box::new_zeroed().assume_init() },
            //capt_history_moves: [[[0; 64]; 12]; 12],
            tt: TranspositionTable::new(config.hash),
            move_stack: Box::new(MoveStack::new()),
            nodes: 0,
            stop_condition: StopCondition::default(),
            //deadline: Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
            should_quit: false,
            ply: 0,
            network_state: NetworkState::default(),
            pv_table: Box::new(PrincipalVariationTable::default()),
            engine_config: config.clone(),
            reporting: UCI,
            //search_stage: Meaningless
        }
    }

    // position X
    pub fn clear_data(&mut self) {
        self.max_depth = 0;
        self.seldepth = 0;
        // Clear killer moves without creating a large stack temporary
        for m in self.killer_moves.iter_mut() {
            *m = Move::create_null();
        }
        self.move_stack.clear();
        self.nodes = 0;
        self.pv_table.clear(0);
        self.stop_condition = StopCondition::default();
        self.stop_condition.soft_nodecount = self.engine_config.soft_nodes;
        self.should_quit = false;
        self.ply = 0;
        self.tt.increment_age();
    }

    //ucinewgame
    pub fn clear_persistent_data(&mut self) {
        self.tt.clear();
        // Zero history on heap without large stack temp
        // Allocate fresh zeroed box (old box dropped)
        self.history_moves = unsafe { Box::new_zeroed().assume_init() };
        //self.capt_history_moves = [[[0; 64]; 12]; 12];
    }

    // ID
    pub fn reset_for_new_iteration(&mut self, depth: usize) {
        self.max_depth = depth;
        self.seldepth = depth;
    }

    // make move during position command parsing
    pub fn prefill_position_info(&mut self, hash: u64) {
        self.move_stack.prefill(hash); 
    }

    pub fn make_move(&mut self, mv: Move, board_position: &BoardPosition, static_eval: i32) {
        self.move_stack.push(board_position.hash, static_eval); 
        self.ply += 1;
        self.network_state.apply_move(mv, board_position);
    }

    pub fn take_back(&mut self) {
        self.move_stack.pop();
        self.ply -= 1;
        self.network_state.undo_move();
    }

    #[inline(always)]
    pub fn get_mvv_lva(victim: Piece, attacker: Piece) -> i32 {
        MVV_LVA[victim as usize % 6 + attacker as usize % 6 * 6]
    }

    pub fn update_killer_move(&mut self, mv: Move) {
        if self.ply < 256 {
            self.killer_moves[self.ply] = mv;
        }
    }

    pub fn update_history(&mut self, board_position: &BoardPosition, mv: Move, bonus: i32) {
        let mh = max_history();
        let clamped_bonus = bonus.clamp(-mh, mh) as i32;
        let piece = board_position.get_piece(mv) as usize;
        let source = mv.get_source_square();
        let target = mv.get_target_square();
        let side = board_position.side;
        if piece < 12 && target < 64 {
            let history_val = self.get_quiet_history(side, mv);           
            self.history_moves[side][source as usize][target as usize] += (clamped_bonus - history_val as i32 * clamped_bonus.abs() / mh) as i16 //second bonus should be abs
            //if mv.is_capture() {
            //    let history_val = self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][piece][target];
            //    self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][piece][target] += clamped_bonus - history_val * clamped_bonus / MAX_HISTORY;
            //} else {
            //}
            //self.history_moves[piece][target] += bonus;

        }
    }

    pub fn get_quiet_history(&self, side: Color, mv: Move) -> i16 {
        self.history_moves[side][mv.get_source_square() as usize][mv.get_target_square() as usize]
    }


    // pub fn get_stats(&self) -> (u64, u64, f64) {
    //     let fill_pct = self.tt.fill_percentage();
    //     (self.nodes_searched, self.tt_hits, fill_pct)
    // }

    pub fn is_trifold_repetition(&self, hash: u64) -> bool {
        self.move_stack.is_draw(hash)
    }

    pub fn has_occured_in_search(&self, hash: u64) -> bool {
        self.move_stack.has_occurred_in_search(hash)
    }

    #[inline(always)]
    pub fn probe_tt(&self, hash: u64) -> Option<&TTEntry> {
        if self.engine_config.hash == 0 {
            return None
        }
        self.tt.probe(hash)
    }

    // add static eval
    #[inline(always)]
    pub fn store_tt(
        &mut self,
        depth: u8,
        score: i32,
        eval: i32,
        flag: TTFlag,
        best_move: Move,
        hash: u64
    ) {
        if self.engine_config.hash == 0 {
            return;
        }

        self.tt.store(
            hash,
            depth,
            score_to_tt(score, self.ply),
            eval,
            flag,
            best_move, // or .into()
        );
    }
}


pub struct StopCondition {
    pub movetime_deadline: Option<u64>,
    pub our_time_ms: Option<u64>,
    pub our_inc_ms: Option<u64>,
    pub depth: Option<usize>,
    pub hard_nodecount: Option<u64>,
    pub soft_nodecount: Option<u64>,
    pub started_search: Instant,
    drop_everything_and_quit: bool 
}

impl Default for StopCondition {
    fn default() -> Self {
        StopCondition { movetime_deadline: None,
            our_time_ms: None,
            our_inc_ms: None,
            depth: None, 
            hard_nodecount: None, 
            soft_nodecount: None, 
            started_search: Instant::now(),
            drop_everything_and_quit: false 
        }
    }
}

impl StopCondition {
    fn passed_deadline(&self) -> bool {
        let elapsed = self.started_search.elapsed().as_millis();
        
        if let Some(movetime_deadline) = self.movetime_deadline {
            if elapsed > movetime_deadline {
                return true;
            }
        }

        if let Some(our_time) = self.our_time_ms {
            if elapsed >= our_time * tm_hard_percent() as u64 / 100 {
                return true;
            }
            
            let our_inc = if let Some(our_inc) = self.our_inc_ms {our_inc} else { 0 };
            let inc_part = our_inc * tm_inc_scale() as u64 / 100;
            let allocation = our_time / tm_alloc_div() as u64 + inc_part; 

            if elapsed > allocation {
                return true;
            }
        }

        false
    }
    
    pub fn should_soft_quit(&self, depth: usize, nodes: u64) -> bool {
        if let Some(max_depth) = self.depth {
            if max_depth == depth {
                return true;
            }
        }

        if let Some(max_nodes) = self.soft_nodecount {
            if nodes >= max_nodes {
                return true;
            }
        }

        let elapsed = self.started_search.elapsed().as_millis();

        if let Some(our_time) = self.our_time_ms {
            if elapsed >= our_time * tm_hard_percent() as u64 / 100 {
                return true;
            }
            
            let our_inc = if let Some(our_inc) = self.our_inc_ms {our_inc} else { 0 };
            let inc_part = our_inc * tm_inc_scale() as u64 / 100;
            let allocation = (our_time / tm_alloc_div() as u64 + inc_part) / tm_soft_div() as u64; 

            if elapsed > allocation {
                return true;
            }
        }

        false
    }

    pub fn should_hard_quit(&mut self, nodes: u64) -> bool {
        
        if self.drop_everything_and_quit {
            return true;
        }

        if self.passed_deadline() && nodes % 1024 == 0 {
            self.drop_everything_and_quit = true;
            return true;
        }

        if let Some(nodelimit) = self.hard_nodecount {
            if nodes > nodelimit { 
                self.drop_everything_and_quit = true;
                return true;
            }
        }

        false
    }

    pub fn reset(&mut self) {
        self.drop_everything_and_quit = false;
        self.started_search = Instant::now();
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Reporting {
    UCI,
    Quiet
}

// #[derive(Debug, PartialEq, Clone, Copy)]
// pub enum SearchStage {
//     Meaningless,
//     Partial,
//     Full,
// }

#[cfg(test)]
mod tests {
    use crate::gui::{parse_position_command, parse_ucinewgame};
    use crate::search::search; 
    use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::{SearchState};

    #[test]
    fn test_clearing_persistent_data_correctly() {
        let mut search_state = SearchState::new(&EngineConfig::thin());
                let mut board_position = parse_position_command(&mut search_state, "position startpos");
                search_state.stop_condition.depth = Some(4);
                let empty_history: [[[i16; 64]; 64]; 2] = [[[0; 64]; 64]; 2];
                search(&board_position, &mut search_state);
                assert_ne!(*search_state.history_moves, empty_history);

                board_position = parse_position_command(&mut search_state, "position startpos moves e2e4 e7e5");
                search_state.stop_condition.depth = Some(4);
                assert_ne!(*search_state.history_moves, empty_history);

                search(&board_position, &mut search_state);
                assert_ne!(*search_state.history_moves, empty_history);

                parse_ucinewgame(&mut search_state);
                board_position = parse_position_command(&mut search_state,"position kiwipete");
                search_state.stop_condition.depth = Some(8);
                assert_eq!(*search_state.history_moves, empty_history);

                search(&board_position, &mut search_state);
                assert_ne!(*search_state.history_moves, empty_history);

    }

    #[test]
    fn test_clear_data_applies_soft_nodes() {
        // Unlimited by default.
                let mut search_state = SearchState::new(&EngineConfig::thin());
                assert_eq!(search_state.stop_condition.soft_nodecount, None);
                search_state.clear_data();
                assert_eq!(search_state.stop_condition.soft_nodecount, None);

                // A configured soft node limit is applied on every clear_data.
                let mut config = EngineConfig::thin();
                config.soft_nodes = Some(12345);
                let mut search_state = SearchState::new(&config);
                search_state.clear_data();
                assert_eq!(search_state.stop_condition.soft_nodecount, Some(12345));

                // A per-search override survives until the next clear_data,
                // which restores the configured value.
                search_state.stop_condition.soft_nodecount = Some(7);
                search_state.clear_data();
                assert_eq!(search_state.stop_condition.soft_nodecount, Some(12345));

                // Setting the option to 0 maps to None (no limit).
                config.soft_nodes = None;
                let mut search_state = SearchState::new(&config);
                search_state.clear_data();
                assert_eq!(search_state.stop_condition.soft_nodecount, None);
    }
}
