use coarsetime::{Duration, Instant};

use crate::gui::parse_move;
use crate::types::board::BoardPosition;
use crate::move_gen::{is_square_attacked};
use crate::shared::{FIRST_KILLER_BONUS, KIWIPETE, MATE_THRESHOLD, MAX_HISTORY, MVV_LVA, Move, MoveSuccess, PV_MOVE_BONUS, Piece, SECOND_KILLER_BONUS, START_POSITION};
use crate::types::tt::{RepetitionTable, TTEntry, TTFlag, TranspositionTable, compute_hash, get_zobrist_keys, score_to_tt};

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    pub board_position: BoardPosition,
    pub max_depth: usize,
    pub seldepth: usize,
    killer_moves: [[Move; 256]; 2],
    //only public for test purposes
    pub history_moves: [[[i32; 64]; 64]; 2],
    //pub capt_history_moves: [[[i32; 64]; 12]; 12], // target, own, captured
    prev_iter_best_move: Move,
    tt: TranspositionTable,
    rep_table: RepetitionTable,
    nodes_searched: u64,
    deadline: Instant,
    should_quit: bool,
    pub ply: usize,
}

impl SearchState {
    pub fn new(fen : &str) -> Self {
        let board_position = BoardPosition::new(fen);
        let search_state = Self {
            board_position: board_position,
            max_depth: 0,
            seldepth: 0,
            killer_moves: [[Move::create_null(); 256]; 2],
            history_moves: [[[0; 64]; 64]; 2],
            //capt_history_moves: [[[0; 64]; 12]; 12],
            prev_iter_best_move: Move::create_null(),
            tt: TranspositionTable::new(),
            rep_table: RepetitionTable::new(),
            nodes_searched: 0,
            deadline: Instant::now().checked_add(Duration::from_secs(1)).unwrap(),
            should_quit: false,
            ply: 0,
        };

        search_state
    }

    // This function was moved here to preserve TT between nodes
    pub fn change_position(&mut self, fen: &str) {
        let board_position = BoardPosition::new(fen);
        self.board_position = board_position;
        self.max_depth = 0;
        self.seldepth = 0;
        self.killer_moves = [[Move::create_null(); 256]; 2];
        self.prev_iter_best_move = Move::create_null();
        self.rep_table.clear();
        self.nodes_searched = 0;
        self.deadline = Instant::now().checked_add(Duration::from_secs(1)).unwrap();
        self.should_quit = false;
        self.ply = 0;
    }

    pub fn clear_persistent_data(&mut self) {
        self.tt.clear();
        self.history_moves = [[[0;64]; 64]; 2];
        //self.capt_history_moves = [[[0; 64]; 12]; 12];
    }

    pub fn parse_position_command(&mut self, command: &str) {
        let words : Vec<&str> = command.trim().split(" ").collect();

        if words.len() < 2 {
            self.change_position(START_POSITION);
            self.clear_persistent_data();
            return;
        }

        let hash = self.board_position.hash;
        let mut hash_in_node = false;

        match words[1] {
            "fen" => {
                self.change_position(&command[13..]);
                if hash == self.board_position.hash {
                    hash_in_node = true;
                }
                if words.len() > 8 {
                    for &i in words[9..].iter() {
                        let mov = parse_move(&self.board_position, i);
                        if let Some(x) = mov {
                            self.make_move(x);
                        }
                        if hash == self.board_position.hash {
                            hash_in_node = true;
                        }
                    }
                }
            },
            "startpos" => {
                self.change_position(START_POSITION);
                if hash == self.board_position.hash {
                    hash_in_node = true;
                }
                for &i in words[2..].iter() {
                    let mov = parse_move(&self.board_position, i);
                    if let Some(x) = mov {
                        self.make_move(x);
                    }
                    if hash == self.board_position.hash {
                        hash_in_node = true;
                    }
                }
            },
            "kiwipete" => {
                self.change_position(KIWIPETE);
                if hash == self.board_position.hash {
                    hash_in_node = true;
                }
                for &i in words[2..].iter() {
                    let mov = parse_move(&self.board_position, i);
                    if let Some(x) = mov {
                        self.make_move(x);
                    }
                    if hash == self.board_position.hash {
                        hash_in_node = true;
                    }
                }

            },
            _ => self.change_position(START_POSITION),
        }

        if !hash_in_node {
            self.clear_persistent_data();
        }

        self.ply = 0;
    }

    pub fn reset_for_new_iteration(&mut self, depth: usize, previter_bestmove: Move) {
        self.max_depth = depth;
        self.seldepth = depth;
        self.prev_iter_best_move = previter_bestmove;
        self.tt.increment_age();
        self.nodes_searched = 0;
    }

    pub fn make_move(&mut self, move_to_make: Move) -> MoveSuccess {
        self.rep_table.push(self.board_position.hash); 
        let result = self.board_position.make_move(move_to_make);
        if result == MoveSuccess::Attacked {
            self.rep_table.pop();
        } else {
            self.ply += 1;
        }

        result
    }

    pub fn make_null_move(&mut self) {
        self.board_position.side = 1-self.board_position.side;
        self.board_position.hash ^= get_zobrist_keys().side_key;

        if self.board_position.enpassant != 0 {
            self.board_position.hash ^= get_zobrist_keys().enpassant_keys[(self.board_position.enpassant % 8) as usize];
            self.board_position.enpassant = 0;
        }
    }

    pub fn take_back(&mut self, move_to_take_back: Move) {
        //take back manages the hash        
        self.board_position.take_back(move_to_take_back, self.rep_table.pop());
        self.ply -= 1;
        debug_assert!(compute_hash(&self.board_position) == self.board_position.hash)
    }

    pub fn take_back_null_move(&mut self, old_ep_square: u8) {
        self.board_position.side = 1-self.board_position.side;
        self.board_position.hash ^= get_zobrist_keys().side_key;

        if old_ep_square != 0 {
            self.board_position.hash ^= get_zobrist_keys().enpassant_keys[(old_ep_square % 8) as usize];
            self.board_position.enpassant = old_ep_square;
        }
    }

    #[inline(always)]
    fn get_mvv_lva(victim: Piece, attacker: Piece) -> i32 {
        MVV_LVA[victim as usize % 6 + attacker as usize % 6 * 6]
    }

    fn get_victim(&self, mv: Move) -> Piece {
        mv.get_taken_piece()
    }

    //Only works before move
    fn get_piece(&self, mv: Move) -> Piece {
        self.board_position.mailbox[mv.get_source_square() as usize]
    }

    pub fn get_move_score(&self, mv: Move) -> i32 {
        // PV move from previous iteration gets highest priority

        if self.ply == 0 && mv == self.prev_iter_best_move {
            return PV_MOVE_BONUS;
        }

        if mv.is_capture() {
            let victim = self.get_victim(mv);
            let mvv = Self::get_mvv_lva(victim, self.get_piece(mv));
            
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
        self.history_moves[self.board_position.side][mv.get_source_square() as usize][mv.get_target_square() as usize]
    }

    pub fn update_killer_move(&mut self, mv: Move) {
        if self.ply < 256 {
            self.killer_moves[1][self.ply] = self.killer_moves[0][self.ply];
            self.killer_moves[0][self.ply] = mv;
        }
    }

    pub fn update_history(&mut self, mv: Move, bonus: i32) {
        let clamped_bonus = bonus.clamp(-MAX_HISTORY, MAX_HISTORY);
        let piece = self.get_piece(mv) as usize;
        let source = mv.get_source_square() as usize;
        let target = mv.get_target_square() as usize;
        let side = self.board_position.side;
        if piece < 12 && target < 64 {
            let history_val = self.history_moves[side][piece][target];
            //let bonus = depth * depth;
            
            self.history_moves[self.board_position.side][source][target] += clamped_bonus - history_val * clamped_bonus / MAX_HISTORY //second bonus should be abs
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

    pub fn is_trifold_repetition(&self) -> bool {
        self.rep_table.is_draw(self.board_position.hash)
    }

    pub fn is_twofold_repetition(&self) -> bool {
        self.rep_table.has_occurred(self.board_position.hash)
    }

    pub fn has_pieces(&self) -> bool {
        self.board_position.bitboards[Piece::B as usize] > 0 ||
        self.board_position.bitboards[Piece::R as usize] > 0 ||
        self.board_position.bitboards[Piece::N as usize] > 0 ||
        self.board_position.bitboards[Piece::Q as usize] > 0 ||
        self.board_position.bitboards[Piece::b as usize] > 0 ||
        self.board_position.bitboards[Piece::r as usize] > 0 ||
        self.board_position.bitboards[Piece::n as usize] > 0 ||
        self.board_position.bitboards[Piece::q as usize] > 0
    }

    pub fn is_king_attacked(&self) -> bool {
        is_square_attacked(self.board_position.bitboards[6*self.board_position.side+5].trailing_zeros() as u8, &self.board_position)
    }

    #[inline(always)]
    pub fn probe_tt(&self) -> Option<&TTEntry> {
        self.tt.probe(self.board_position.hash)
    }

    // add static eval
    #[inline(always)]
    pub fn store_tt(
        &mut self,
        depth: u8,
        score: i32,
        flag: TTFlag,
        best_move: Move,
    ) {
        self.tt.store(
            self.board_position.hash,
            depth,
            score_to_tt(score, self.ply),
            flag,
            best_move, // or .into()
        );
    }

    #[inline(always)]
    pub fn tt_move(&mut self) -> Option<Move> {
        self.tt
            .probe(self.board_position.hash)
            .and_then(|e| {
                if !e.best_move.is_null() {
                    Some(e.best_move)
                } else {
                    None
                }
            })
    }

    pub fn passed_deadline(&self) -> bool {
        return Instant::now() > self.deadline;
    }

    pub fn should_quit(&mut self) -> bool {
        if self.should_quit {
            return true;
        }

        if (self.nodes_searched & 0x3fff) == 0 {
            if self.passed_deadline() {
                self.should_quit = true;
                return true;
            }
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
        Self::new(START_POSITION)
    }
}


#[cfg(test)]
mod tests {
    use std::thread;
    use crate::{search::search, shared::{START_POSITION}, types::search_state::{SearchState}};

    #[test]
    fn assert_clearing_persistent_data_correctly() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(START_POSITION);
                let empty_history = [[[0; 64]; 64]; 2];
                search(&mut search_state, Some(4), None);
                assert_ne!(search_state.history_moves, empty_history);

                search_state.parse_position_command("position startpos moves e2e4 e7e5");
                assert_ne!(search_state.history_moves, empty_history);

                search(&mut search_state, Some(4), None);
                assert_ne!(search_state.history_moves, empty_history);

                search_state.parse_position_command("position kiwipete");
                assert_eq!(search_state.history_moves, empty_history);

                search(&mut search_state, Some(6), None);
                assert_ne!(search_state.history_moves, empty_history);

            })
            .unwrap();
        handler.join().unwrap();



    }
}
