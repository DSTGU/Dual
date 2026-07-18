use coarsetime::{Instant};

use crate::primitives::board::BoardPosition;
use crate::primitives::shared::{Move, Piece};
use crate::primitives::consts::{FIRST_KILLER_BONUS, MAX_HISTORY, MVV_LVA, SECOND_KILLER_BONUS};
use crate::search_objs::config::EngineConfig;
use crate::search_objs::move_stack::MoveStack;
use crate::search_objs::tt::{TTEntry, TTFlag, TranspositionTable, score_to_tt};
use crate::evaluation::network_state::NetworkState;

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    pub max_depth: usize, // Of the search iteration, not in general
    pub seldepth: usize,
    killer_moves: [[Move; 256]; 2],
    //only public for test purposes
    pub history_moves: [[[i32; 64]; 64]; 2],
    //pub capt_history_moves: [[[i32; 64]; 12]; 12], // target, own, captured
    tt: TranspositionTable,
    pub rep_table: MoveStack,
    pub nodes: u64,
    pub stop_condition: StopCondition,
    should_quit: bool,
    pub ply: usize,
    pub network_state: NetworkState,
    pub engine_config: EngineConfig
}

impl SearchState {
    pub fn new(config: &EngineConfig) -> Self {
        Self {
            max_depth: 0,
            seldepth: 0,
            killer_moves: [[Move::create_null(); 256]; 2],
            history_moves: [[[0; 64]; 64]; 2],
            //capt_history_moves: [[[0; 64]; 12]; 12],
            tt: TranspositionTable::new(config.hash),
            rep_table: MoveStack::new(),
            nodes: 0,
            stop_condition: StopCondition::default(),
            //deadline: Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
            should_quit: false,
            ply: 0,
            network_state: NetworkState::default(),
            engine_config: config.clone()
        }
    }

    // This function was moved here to preserve TT between nodes
    pub fn clear_data(&mut self) {
        self.max_depth = 0;
        self.seldepth = 0;
        self.killer_moves = [[Move::create_null(); 256]; 2];
        self.rep_table.clear();
        self.nodes = 0;
        self.stop_condition = StopCondition::default();
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

    pub fn make_move(&mut self, mv: Move, board_position: &BoardPosition) {
        self.rep_table.push(board_position.hash); 
        self.ply += 1;
        self.network_state.apply_move(mv, board_position);
    }

    pub fn take_back(&mut self) {
        //take back manages the hash        
        self.rep_table.pop();
        self.ply -= 1;
        self.network_state.undo_move();
    }

    #[inline(always)]
    fn get_mvv_lva(victim: Piece, attacker: Piece) -> i32 {
        MVV_LVA[victim as usize % 6 + attacker as usize % 6 * 6]
    }

    pub fn get_move_score(&self, board_position: &BoardPosition, mv: Move) -> i32 {
        if mv.is_capture() {
            let victim = board_position.get_victim(mv);
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
            flag,
            best_move, // or .into()
        );
    }

    #[inline(always)]
    pub fn tt_move(&mut self, hash: u64) -> Option<Move> {
        if self.engine_config.hash == 0 {
            return None;
        }

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

}


pub struct StopCondition {
    pub movetime_deadline: Option<u64>,
    pub our_time_ms: Option<u64>,
    pub our_inc_ms: Option<u64>,
    pub depth: Option<usize>,
    _hard_nodecount: Option<u64>,
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
            _hard_nodecount: None, 
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
            if elapsed >= our_time * 3 / 4 {
                return true;
            }
            
            let our_time_plusinc = if let Some(our_inc) = self.our_inc_ms { our_time/17 + our_inc} else { our_time/17 }; 

            if elapsed > our_time_plusinc {
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
            if max_nodes == nodes {
                return true;
            }
        }

        let elapsed = self.started_search.elapsed().as_millis();

        if let Some(our_time) = self.our_time_ms {
            if elapsed >= our_time * 3 / 4 {
                return true;
            }
            
            let our_time_plusinc = if let Some(our_inc) = self.our_inc_ms { our_time/17 + our_inc } else { our_time/17 } / 3; 

            if elapsed > our_time_plusinc {
                return true;
            }
        }

        false
    }

    pub fn should_hard_quit(&mut self, _nodes: u64) -> bool {
        
        if self.drop_everything_and_quit {
            return true;
        }



        if self.passed_deadline() {
            self.drop_everything_and_quit = true;
            return true;
        }

        false
    }

    pub fn reset(&mut self) {
        self.drop_everything_and_quit = false;
        self.started_search = Instant::now();
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use crate::gui::{parse_position_command, parse_ucinewgame};
    use crate::search::search; 
    use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::{SearchState};

    #[test]
    fn test_clearing_persistent_data_correctly() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(&EngineConfig::thin());
                let mut board_position = parse_position_command(&mut search_state, "position startpos");
                search_state.stop_condition.depth = Some(4);
                let empty_history = [[[0; 64]; 64]; 2];
                search(&board_position, &mut search_state);
                assert_ne!(search_state.history_moves, empty_history);

                board_position = parse_position_command(&mut search_state, "position startpos moves e2e4 e7e5");
                search_state.stop_condition.depth = Some(4);
                assert_ne!(search_state.history_moves, empty_history);

                search(&board_position, &mut search_state);
                assert_ne!(search_state.history_moves, empty_history);

                parse_ucinewgame(&mut search_state);
                board_position = parse_position_command(&mut search_state,"position kiwipete");
                search_state.stop_condition.depth = Some(6);
                assert_eq!(search_state.history_moves, empty_history);

                search(&board_position, &mut search_state);
                assert_ne!(search_state.history_moves, empty_history);

            })
            .unwrap();
        handler.join().unwrap();

    }
}
