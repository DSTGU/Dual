use arrayvec::ArrayVec;

use crate::movegen::move_gen::{NoisyMovegen, QuietMovegen, generate_moves};
use crate::primitives::board::BoardPosition;
use crate::primitives::consts::{FIRST_KILLER_BONUS};
use crate::primitives::shared::Move;
use crate::search_objs::search_state::SearchState;
use crate::search_objs::see::{see_a_move_threshold};
use crate::tunable::mp_see_threshold;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum Stage {
    HashMove,
    Movegen,
    Noisy,
    Quiet,
    //GenerateNoisy,
    //GoodNoisy,
    //Quiet,
    BadNoisy,
}


pub struct MoveEntry {
    pub mv: Move,
    pub score: i32
}

pub struct MovePicker {
    list: ArrayVec<MoveEntry, 256>,
    tt_move: Move,
    stage: Stage,
    bad_noisy: ArrayVec<Move, 16>,
    bad_noisy_idx: usize,
    skip_quiets: bool
    //noisy_count: usize,
}


impl MovePicker {
    pub const fn new(tt_move: Move) -> Self {
        Self {
            list: ArrayVec::new_const(),
            tt_move,
            stage:  Stage::HashMove,
            bad_noisy: ArrayVec::new_const(),
            bad_noisy_idx: 0,
            skip_quiets: false
        }
    }

    //pub fn next<NODE: NodeType>(&mut self, board_position: &BoardPosition, search_state: &SearchState, quiescence: bool) -> Option<(Move, BoardPosition)> {
    pub fn next(&mut self, board_position: &BoardPosition, search_state: &SearchState, quiescence: bool) -> Option<(Move, BoardPosition)> {
        
        if self.stage == Stage::HashMove {

            self.stage = Stage::Movegen;

            if !self.tt_move.is_null() && board_position.can_make_move(self.tt_move){

                let new_board= board_position.make_move(self.tt_move);
                
                if let Some(new_board) = new_board {
                    return Some((self.tt_move, new_board));
                }
            }
        }

        if self.stage == Stage::Movegen {
            //TODO: switch
            generate_moves::<NoisyMovegen>(board_position, &mut self.list);
            self.score_moves(board_position, search_state);
            self.stage = Stage::Noisy;
        }
        
        if self.stage == Stage::Noisy {

            while !self.list.is_empty() {
                let entry = self.get_best_entry();

                // if NODE::ROOT {
                //     self.score_noisy(td);
                // }

                //self.noisy_count += 1;

                let new_board= board_position.make_move(entry.mv);
                    
                if let Some(new_board) = new_board {
                    if !see_a_move_threshold(board_position, entry.mv, &new_board, mp_see_threshold()) {
                        self.bad_noisy.push(entry.mv);
                        continue;
                    }

                    return Some((entry.mv, new_board));
                }
            }

            if quiescence {
                return None;
                // Currently no need to check bad noisy in quiescence (they are always pruned)
                //self.stage = Stage::BadNoisy;
            } else if self.skip_quiets {
                self.stage = Stage::BadNoisy;
            } else {
                generate_moves::<QuietMovegen>(board_position, &mut self.list);
                self.score_moves(board_position, search_state);
                self.stage = Stage::Quiet;
            }
        }

        if self.stage == Stage::Quiet {

            if self.skip_quiets {
                self.stage = Stage::BadNoisy;
            }

            while !self.list.is_empty() {
                let entry = self.get_best_entry();

                let new_board= board_position.make_move(entry.mv);
                    
                if let Some(new_board) = new_board {
                    return Some((entry.mv, new_board));
                }
            }

            self.stage = Stage::BadNoisy;
        }

        if self.stage == Stage::BadNoisy {
            while self.bad_noisy_idx < self.bad_noisy.len() {

                let mv = self.bad_noisy[self.bad_noisy_idx];
                let new_board= board_position.make_move(mv);
                
                self.bad_noisy_idx += 1;
                if let Some(new_board) = new_board {
                    return Some((mv, new_board));
                }
            }
        }

        //println!("No more moves. Returning None");
        None
    }

    pub fn skip_quiets(&mut self) {
        self.skip_quiets = true;
    }

    fn get_best_entry(&mut self) -> MoveEntry {
        let mut best_index = 0;
        let mut best_score = i32::MIN;

        for (index, entry) in self.list.iter().enumerate() {
            if entry.score >= best_score {
                best_index = index;
                best_score = entry.score;
            }
        }
        self.list.remove(best_index)
    }

    fn score_moves(&mut self, board_position: &BoardPosition, search_state: &SearchState) {
        for entry in self.list.iter_mut() {
            let mv = entry.mv;
            entry.score = Self::get_move_score(board_position, search_state, mv);
        }
    }

    pub fn get_move_score(board_position: &BoardPosition, search_state: &SearchState, mv: Move) -> i32 {
        if mv.is_capture() {
            let victim = board_position.get_victim(mv);
            let mvv = SearchState::get_mvv_lva(victim, board_position.get_piece(mv));
            
            return mvv;
            //return mvv + 
            //    self.capt_history_moves[self.board_position.mailbox[mv.get_target_square() as usize] as usize][self.get_piece(mv) as usize][mv.get_target_square() as usize];
        }

        if search_state.ply < 256 {
            if search_state.killer_moves[search_state.ply] == mv {
                return FIRST_KILLER_BONUS;
            }
        }

        // History heuristic
        search_state.get_quiet_history(board_position.side, mv) as i32
    }
}