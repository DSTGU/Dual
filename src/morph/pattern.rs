use std::{fs, path::Path, sync::RwLock};

use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use crate::{shared::Piece, types::board::BoardPosition};

pub const DB_PATH: &str = "./database.json"; 

#[derive(Debug, Serialize, Deserialize)]
pub struct PatternDatabase {
    pub patterns: Vec<Pattern>,
}

impl Default for PatternDatabase {
    fn default() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }
}

impl PatternDatabase {
    pub fn evaluate(
        &self,
        board: &BoardPosition,
        beta: f32,
    ) -> f32 {
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for pattern in &self.patterns {
            if !pattern.applies(board) {
                continue;
            }

            let w = pattern.weight();
            let ex = (w - 0.5).abs().powf(beta);

            numerator += w * ex;
            denominator += ex;
        }

        if denominator == 0.0 {
            0.5
        } else {
            numerator / denominator
        }
    }

    fn load_or_create() -> Self {
        if Path::new(DB_PATH).exists() {
            let content = fs::read_to_string(DB_PATH)
                .expect("failed to read database file");

            serde_json::from_str(&content)
                .expect("failed to parse database")
        } else {
            Self::default()
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(DB_PATH, json)?;
        Ok(())
    }
}

// Global runtime-managed database
pub static DATABASE: Lazy<RwLock<PatternDatabase>> =
    Lazy::new(|| RwLock::new(PatternDatabase::load_or_create()));

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Pattern {
    //Graph(GraphPattern),
    Material(MaterialPattern),
}

pub trait MatchesPosition {
    fn applies(&self, position: &BoardPosition) -> bool;
}

impl Pattern {
    pub fn weight(&self) -> f32 {
        match self {
            //Pattern::Graph(p) => p.weight,
            Pattern::Material(p) => p.weight,
        }
    }

    pub fn applies(&self, board: &BoardPosition) -> bool {
        match self {
            //Pattern::Graph(p) => p.applies(board),
            Pattern::Material(p) => p.applies(board),
        }
    }
}

pub enum TypeOfPattern {
    Material,
    Attack,
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialVector {
    pub pawns: i8,
    pub knights: i8,
    pub bishops: i8,
    pub rooks: i8,
    pub queens: i8,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MaterialPattern {
    pub material: MaterialVector,
    pub weight: f32,
}

impl MaterialPattern {

    pub fn applies(&self, board_position: &BoardPosition) -> bool {
        //TODO

        if board_position.side == 0 {
            if board_position.bitboards[Piece::P as usize].count_ones() as i32 - board_position.bitboards[Piece::p as usize].count_ones() as i32 != self.material.pawns as i32 {
                return false;
            }
            if board_position.bitboards[Piece::N as usize].count_ones() as i32 - board_position.bitboards[Piece::n as usize].count_ones() as i32 != self.material.knights as i32 {
                return false;
            }
            if board_position.bitboards[Piece::B as usize].count_ones() as i32 - board_position.bitboards[Piece::b as usize].count_ones() as i32 != self.material.bishops as i32 {
                return false;
            }
            if board_position.bitboards[Piece::R as usize].count_ones() as i32 - board_position.bitboards[Piece::r as usize].count_ones() as i32 != self.material.rooks as i32 {
                return false;
            }
            if board_position.bitboards[Piece::Q as usize].count_ones() as i32 - board_position.bitboards[Piece::q as usize].count_ones() as i32 != self.material.queens as i32 {
                return false;
            }

            return true;
        } else {
            if board_position.bitboards[Piece::p as usize].count_ones() as i32 - board_position.bitboards[Piece::P as usize].count_ones() as i32 != self.material.pawns as i32 {
                return false;
            }
            if board_position.bitboards[Piece::n as usize].count_ones() as i32 - board_position.bitboards[Piece::N as usize].count_ones() as i32 != self.material.knights as i32 {
                return false;
            }
            if board_position.bitboards[Piece::b as usize].count_ones() as i32 - board_position.bitboards[Piece::B as usize].count_ones() as i32 != self.material.bishops as i32 {
                return false;
            }
            if board_position.bitboards[Piece::r as usize].count_ones() as i32 - board_position.bitboards[Piece::R as usize].count_ones() as i32 != self.material.rooks as i32 {
                return false;
            }
            if board_position.bitboards[Piece::q as usize].count_ones() as i32 - board_position.bitboards[Piece::Q as usize].count_ones() as i32 != self.material.queens as i32 {
                return false;
            }

            return true;
        }
    }
}

impl MaterialVector {

    pub fn extract_pattern(board_position: &BoardPosition) -> MaterialVector {   
        if board_position.side == 0 {
            MaterialVector {
                pawns: board_position.bitboards[Piece::P as usize].count_ones() as i8 - board_position.bitboards[Piece::p as usize].count_ones() as i8,
                knights: board_position.bitboards[Piece::N as usize].count_ones() as i8 - board_position.bitboards[Piece::n as usize].count_ones() as i8,
                bishops: board_position.bitboards[Piece::B as usize].count_ones() as i8 - board_position.bitboards[Piece::b as usize].count_ones() as i8,
                rooks: board_position.bitboards[Piece::R as usize].count_ones() as i8 - board_position.bitboards[Piece::r as usize].count_ones() as i8,
                queens: board_position.bitboards[Piece::Q as usize].count_ones() as i8 - board_position.bitboards[Piece::q as usize].count_ones() as i8
            }
    
        } else {
            MaterialVector {
                pawns: board_position.bitboards[Piece::p as usize].count_ones() as i8 - board_position.bitboards[Piece::P as usize].count_ones() as i8,
                knights: board_position.bitboards[Piece::n as usize].count_ones() as i8 - board_position.bitboards[Piece::N as usize].count_ones() as i8,
                bishops: board_position.bitboards[Piece::b as usize].count_ones() as i8 - board_position.bitboards[Piece::B as usize].count_ones() as i8,
                rooks: board_position.bitboards[Piece::r as usize].count_ones() as i8 - board_position.bitboards[Piece::R as usize].count_ones() as i8,
                queens: board_position.bitboards[Piece::q as usize].count_ones() as i8 - board_position.bitboards[Piece::Q as usize].count_ones() as i8
            }
        }
    }
}