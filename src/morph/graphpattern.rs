use serde::{Deserialize, Serialize};

use crate::{attacks::get_piece_attacks, shared::Piece, types::board::BoardPosition};


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct GraphPattern {
    //pub graph: PositionGraph,
    pub attacks: Vec<PositionEdge>,
    // optional metadata
    //pub variance: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct PositionEdge {
    pub kind: EdgeKind,
    pub attacker: Piece,
    pub victim: Piece
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    DirectAttack,
    IndirectAttack,
    DiscoveredAttack,
    Defends,
}