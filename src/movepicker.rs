use crate::movegen::move_gen::generate_moves;
use crate::primitives::board::BoardPosition;
use crate::primitives::consts::{FIRST_KILLER_BONUS, SECOND_KILLER_BONUS};
use crate::primitives::shared::Move;
use crate::search_objs::search_state::SearchState;

#[derive(Copy, Clone, Eq, PartialEq, PartialOrd)]
pub enum Stage {
    HashMove,
    Movegen,
    Rest
    //GenerateNoisy,
    //GoodNoisy,
    //Quiet,
    //BadNoisy,
}


pub struct MoveEntry {
    mv: Move,
    score: i32
}

pub struct MovePicker {
    list: Vec<MoveEntry>,
    tt_move: Move,
    stage: Stage,
    //bad_noisy: Vec<Move>,
    //bad_noisy_idx: usize,
    //noisy_count: usize,
}


impl MovePicker {
    pub const fn new(tt_move: Move) -> Self {
        Self {
            list: vec![],
            tt_move,
            stage:  Stage::HashMove,
        }
    }

    //pub fn next<NODE: NodeType>(&mut self, board_position: &BoardPosition, search_state: &SearchState, quiescence: bool) -> Option<(Move, BoardPosition)> {
    pub fn next(&mut self, board_position: &BoardPosition, search_state: &SearchState, quiescence: bool) -> Option<(Move, BoardPosition)> {
        
        if self.stage == Stage::HashMove {

            self.stage = Stage::Movegen;

            if !self.tt_move.is_null() {

                let new_board= board_position.make_move(self.tt_move);
                
                if new_board.is_some() {
                    return Some((self.tt_move, new_board.unwrap()));
                }
            }
        }

        if self.stage == Stage::Movegen {
            self.list = generate_moves(board_position, quiescence).iter().map(|mv| MoveEntry{mv: *mv, score: 0}).collect();
            self.score_moves(board_position, search_state);
            self.stage = Stage::Rest;
        }
        
        if self.stage == Stage::Rest {

            while !self.list.is_empty() {
                let entry = self.get_best_entry();

                // if !td.board.see(entry.mv, threshold) {
                //     self.bad_noisy.push(entry.mv);
                //     continue;
                // }

                // if NODE::ROOT {
                //     self.score_noisy(td);
                // }

                //self.noisy_count += 1;

                let new_board= board_position.make_move(entry.mv);
                    
                if new_board.is_some() {
                    return Some((entry.mv, new_board.unwrap()));
                }
            }

        }

        //println!("No more moves. Returning None");
        None
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
            if search_state.killer_moves[0][search_state.ply] == mv {
                return FIRST_KILLER_BONUS;
            }
            if search_state.killer_moves[1][search_state.ply] == mv {
                return SECOND_KILLER_BONUS;
            }
        }

        // History heuristic
        search_state.history_moves[board_position.side][mv.get_source_square() as usize][mv.get_target_square() as usize]
    }
}