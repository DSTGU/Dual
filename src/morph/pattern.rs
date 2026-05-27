use std::{borrow::Borrow, collections::{HashMap, HashSet}, fs, hash::{Hash, Hasher}, path::{Path, PathBuf}, sync::RwLock};

use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use crate::{shared::Piece, types::board::BoardPosition};

pub const DB_PATH: &str = "./database.json"; 
pub const ALPHA : f32 = 0.08;
pub const BETA : f32 = 1.0;

pub struct DatabaseState {
    pub path: PathBuf,
    pub db: PatternDatabase,
}

impl DatabaseState {
    pub fn switch_database<P: AsRef<Path>>(&mut self, path: P) -> anyhow::Result<()> {
        let path = path.as_ref();

        let new_db = PatternDatabase::load_from_path(path);

        //let mut state = DATABASE.write().unwrap();

        self.path = path.to_path_buf();
        self.db = new_db;

        Ok(())
    }
}

pub static DATABASE: Lazy<RwLock<DatabaseState>> = Lazy::new(|| {
    RwLock::new(DatabaseState {
        path: PathBuf::from(DB_PATH),
        db: PatternDatabase::load_from_path(DB_PATH),
    })
});


// // Global runtime-managed database
// pub static DATABASE: Lazy<RwLock<PatternDatabase>> =
//     Lazy::new(|| RwLock::new(PatternDatabase::load_or_create()));


#[derive(Debug, Serialize, Deserialize)]
pub struct PatternDatabase {
    pub patterns: HashSet<Pattern>,
}

impl Default for PatternDatabase {
    fn default() -> Self {
        Self {
            patterns: HashSet::new(),
        }
    }
}

impl PatternDatabase {
    pub fn evaluate(
        &self,
        board: &BoardPosition,
    ) -> f32 {
        let mut numerator = 0.0;
        let mut denominator = 0.0;

        for pattern in &self.patterns {
            if !pattern.applies(board) {
                continue;
            }

            let w = pattern.wdl;
            let ex = pattern.weight; // technically non compliant

            numerator += w * ex;
            denominator += ex;
        }

        if denominator == 0.0 {
            0.5
        } else {
            numerator / denominator
        }
    }

    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Self {
        let path = path.as_ref();

        if path.exists() {
            let content = fs::read_to_string(path)
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


#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub data: PatternData,
    pub wdl: f32,
    pub weight: f32,
}

impl Borrow<PatternData> for Pattern {
    fn borrow(&self) -> &PatternData {
        &self.data
    }
}

impl Hash for Pattern {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data.hash(state);
    }
}


impl PartialEq for Pattern {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}

impl Eq for Pattern {}


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub enum PatternData {
    //Graph(GraphPattern),
    Material(MaterialPattern),
}

pub trait MatchesPosition {
    fn applies(&self, position: &BoardPosition) -> bool;
}

impl Pattern {
    pub fn weight(&self) -> f32 {
        self.weight
    }

    pub fn applies(&self, board: &BoardPosition) -> bool {
        match &self.data {
            //Pattern::Graph(p) => p.applies(board),
            PatternData::Material(p) => p.applies(board),
        }
    }
}

pub enum TypeOfPattern {
    Material,
    Attack,
}



#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MaterialPattern {
    pub pawns: i8,
    pub knights: i8,
    pub bishops: i8,
    pub rooks: i8,
    pub queens: i8,
}

impl MaterialPattern {

    pub fn applies(&self, board_position: &BoardPosition) -> bool {
        //TODO

        if board_position.side == 0 {
            if board_position.bitboards[Piece::P as usize].count_ones() as i32 - board_position.bitboards[Piece::p as usize].count_ones() as i32 != self.pawns as i32 {
                return false;
            }
            if board_position.bitboards[Piece::N as usize].count_ones() as i32 - board_position.bitboards[Piece::n as usize].count_ones() as i32 != self.knights as i32 {
                return false;
            }
            if board_position.bitboards[Piece::B as usize].count_ones() as i32 - board_position.bitboards[Piece::b as usize].count_ones() as i32 != self.bishops as i32 {
                return false;
            }
            if board_position.bitboards[Piece::R as usize].count_ones() as i32 - board_position.bitboards[Piece::r as usize].count_ones() as i32 != self.rooks as i32 {
                return false;
            }
            if board_position.bitboards[Piece::Q as usize].count_ones() as i32 - board_position.bitboards[Piece::q as usize].count_ones() as i32 != self.queens as i32 {
                return false;
            }

            return true;
        } else {
            if board_position.bitboards[Piece::p as usize].count_ones() as i32 - board_position.bitboards[Piece::P as usize].count_ones() as i32 != self.pawns as i32 {
                return false;
            }
            if board_position.bitboards[Piece::n as usize].count_ones() as i32 - board_position.bitboards[Piece::N as usize].count_ones() as i32 != self.knights as i32 {
                return false;
            }
            if board_position.bitboards[Piece::b as usize].count_ones() as i32 - board_position.bitboards[Piece::B as usize].count_ones() as i32 != self.bishops as i32 {
                return false;
            }
            if board_position.bitboards[Piece::r as usize].count_ones() as i32 - board_position.bitboards[Piece::R as usize].count_ones() as i32 != self.rooks as i32 {
                return false;
            }
            if board_position.bitboards[Piece::q as usize].count_ones() as i32 - board_position.bitboards[Piece::Q as usize].count_ones() as i32 != self.queens as i32 {
                return false;
            }

            return true;
        }
    }

    pub fn extract_pattern(board_position: &BoardPosition) -> MaterialPattern {   
        if board_position.side == 0 {
            MaterialPattern {
                pawns: board_position.bitboards[Piece::P as usize].count_ones() as i8 - board_position.bitboards[Piece::p as usize].count_ones() as i8,
                knights: board_position.bitboards[Piece::N as usize].count_ones() as i8 - board_position.bitboards[Piece::n as usize].count_ones() as i8,
                bishops: board_position.bitboards[Piece::B as usize].count_ones() as i8 - board_position.bitboards[Piece::b as usize].count_ones() as i8,
                rooks: board_position.bitboards[Piece::R as usize].count_ones() as i8 - board_position.bitboards[Piece::r as usize].count_ones() as i8,
                queens: board_position.bitboards[Piece::Q as usize].count_ones() as i8 - board_position.bitboards[Piece::q as usize].count_ones() as i8
            }
    
        } else {
            MaterialPattern {
                pawns: board_position.bitboards[Piece::p as usize].count_ones() as i8 - board_position.bitboards[Piece::P as usize].count_ones() as i8,
                knights: board_position.bitboards[Piece::n as usize].count_ones() as i8 - board_position.bitboards[Piece::N as usize].count_ones() as i8,
                bishops: board_position.bitboards[Piece::b as usize].count_ones() as i8 - board_position.bitboards[Piece::B as usize].count_ones() as i8,
                rooks: board_position.bitboards[Piece::r as usize].count_ones() as i8 - board_position.bitboards[Piece::R as usize].count_ones() as i8,
                queens: board_position.bitboards[Piece::q as usize].count_ones() as i8 - board_position.bitboards[Piece::Q as usize].count_ones() as i8
            }
        }
    }
}