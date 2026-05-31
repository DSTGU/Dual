use std::{borrow::Borrow, collections::{HashSet}, fs, hash::{Hash, Hasher}, path::{Path, PathBuf}, sync::RwLock};

use once_cell::sync::Lazy;
use serde::{Serialize, Deserialize};

use crate::{evaluate::pattern_evaluate, morph::graphpattern::GraphPattern, shared::{KIWIPETE, Piece, START_POSITION}, types::{board::BoardPosition, config::EngineConfig}};

pub const DB_PATH: &str = "./database.json"; 
pub const ALPHA : f32 = 0.01;
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


    pub fn print_info(&self) {
        println!("Path: {}", self.path.as_os_str().to_str().unwrap_or("Empty"));
        println!("Entries: {}", self.db.patterns.len());

        let wdlvec: Vec<f32> = self.db.patterns.iter().map(|p| p.wdl).collect();
        let avg =  wdlvec.iter().sum::<f32>() / wdlvec.len() as f32;

        println!("Avg WDL: {}", avg);

        let start_pos = BoardPosition::new(START_POSITION);
        println!("startposition static dbeval (wdl): {}, cp conversion: {}", self.db.evaluate(&start_pos), pattern_evaluate(&start_pos));
        let scandi1 = BoardPosition::new("rnbqkbnr/ppp1pppp/8/3P4/8/8/PPPP1PPP/RNBQKBNR b KQkq - 0 2");
        println!("Scandi 1... static dbeval (wdl): {}, cp conversion: {}", self.db.evaluate(&scandi1), pattern_evaluate(&scandi1));
        let scandi2 = BoardPosition::new("rnb1kbnr/ppp1pppp/8/3q4/8/8/PPPP1PPP/RNBQKBNR w KQkq - 0 3");
        println!("Scandi 2. static dbeval (wdl): {}, cp conversion: {}", self.db.evaluate(&scandi2), pattern_evaluate(&scandi2));
        let kiwipete_pos = BoardPosition::new(KIWIPETE);
        println!("kiwipete static dbeval (wdl): {}, cp conversion: {}", self.db.evaluate(&kiwipete_pos), pattern_evaluate(&kiwipete_pos));

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

        let patterns = board.extract_patterns();

        for pattern in &patterns {
            let db_data = self.patterns.get(pattern);
            
            if db_data.is_none() {
                continue;
            }

            let db_pattern = db_data.unwrap();

            let w = db_pattern.wdl;
            let ex = db_pattern.weight; // technically non compliant

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

    pub fn save(&self, config: &EngineConfig) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(config.get_path(), json)?;
        Ok(())
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct Pattern {
    pub data: PatternData,
    pub wdl: f32,
    pub weight: f32,
    pub uses: i32,
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
    Graph(GraphPattern),
    Material(MaterialPattern),
}

pub trait MatchesPosition {
    fn applies(&self, position: &BoardPosition) -> bool;
}

impl Pattern {
    pub fn weight(&self) -> f32 {
        self.weight
    }

    // pub fn applies(&self, board: &BoardPosition) -> bool {
    //     match &self.data {
    //         Pattern::Graph(p) => p.applies(board),
    //         PatternData::Material(p) => p.applies(board),
    //     }
    // }
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