use serde::{Deserialize, Serialize};

use crate::{shared::Piece};


#[derive(Debug, Clone, Serialize, Deserialize, Hash, PartialEq, Eq)]
pub struct GraphPattern {
    pub attacks: Vec<PositionEdge>,
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