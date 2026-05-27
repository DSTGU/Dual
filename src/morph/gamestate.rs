use std::collections::HashSet;

use crate::{morph::pattern::{ALPHA, DATABASE, Pattern}, shared::MATE_SCORE, types::board::BoardPosition};

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

    pub fn result_f32(&self) -> f32 {
        match self {
            WDL::Win => 1.0,
            WDL::Draw => 0.5,
            WDL::Loss => 0.0,
        }
    }
}

impl GameHistory {
    pub fn save_patterns(&mut self) {

        if self.positions.is_empty() {
            return;
        }

        let game_end_eval = self.positions.last().unwrap().1;

        if game_end_eval.abs() > 10000 && game_end_eval.abs() < MATE_SCORE - 100 {
            return;
        }

        let mut db = DATABASE.write().unwrap();

        let result = WDL::from_eval(game_end_eval);

        let mut duplicate_patterns = HashSet::new(); 

        for position_tuple in self.positions.iter().rev() {
            let board = &position_tuple.0;

            let board_patterns = board.extract_patterns();
            
            for pattern in board_patterns {
                if duplicate_patterns.contains(&pattern) {
                   continue; 
                } else {

                    if let Some(mut existing_pattern) = db.patterns.take(&pattern) {
                        // update behavior
                        existing_pattern.wdl =
                            (1.0 - ALPHA) * existing_pattern.wdl + ALPHA * result.result_f32();

                        db.patterns.insert(existing_pattern);
                    } else {
                        // add behavior
                        let pattern = Pattern {
                            wdl: 0.0,
                            data: pattern.clone(),
                            weight: 1.0,
                        };

                        db.patterns.insert(pattern);
                    }
                    
                    duplicate_patterns.insert(pattern);
                }

            } 
        }

        match db.save() {
            Err(error) => println!("Error, {}", error.backtrace()),
            Ok(_) => {
                println!("Saved {} patterns", duplicate_patterns.len());
                self.positions = vec![];
            }
        };
        
    }
}