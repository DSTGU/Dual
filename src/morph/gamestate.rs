use crate::{shared::MATE_SCORE, types::board::BoardPosition};




pub struct GameHistory {
    pub positions: Vec<(BoardPosition, i32)>
}

pub enum WDL {
    Win, Draw, Loss
}

impl WDL {
    pub fn from_eval(eval: i32) -> WDL {
        if eval.abs() < 10000 {
           return WDL::Draw; 
        }
        if eval > 0 {
            return WDL::Win;
        }

        return WDL::Loss;
    }
}

impl GameHistory {
    pub fn save_patterns(&self) {

        if self.positions.is_empty() {
            return;
        }

        let game_end_eval = self.positions.last().unwrap().1;

        if game_end_eval.abs() > 10000 && game_end_eval.abs() < MATE_SCORE - 100 {
            return;
        }

        let result = WDL::from_eval(game_end_eval);

        for position_tuple in self.positions.iter().rev() {
            let board = &position_tuple.0;

               
        }
    }
}