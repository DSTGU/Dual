use serde::{Deserialize, Serialize};

use crate::{attacks::get_piece_attacks, shared::Piece, types::board::BoardPosition};


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphPattern {
    //pub graph: PositionGraph,

    /// learned value in [0, 1]
    pub weight: f32,

    /// optional metadata
    pub uses: u32,
    pub variance: f32,
}


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum EdgeKind {
    DirectAttack,
    IndirectAttack,
    DiscoveredAttack,
    Defends,
}

pub fn extract_patterns(board: &BoardPosition) -> Vec<GraphPattern> {
    let mut patterns = Vec::new();

    for from_sq in 0..64 {
        let attacker = board.mailbox[from_sq];

        if attacker == Piece::NONE {
            continue;
        }

        let attacks = get_piece_attacks(board, from_sq);

        let mut bb = attacks;

        while bb != 0 {
            let to_sq = bb.trailing_zeros() as usize;

            let target = board.mailbox[to_sq];

            if target != Piece::NONE {
                // patterns.push(PatternKey {
                //     atoms: vec![
                //         PatternAtom {
                //             attacker,
                //             relation: RelationKind::Attacks,
                //             target,
                //         }
                //     ]
                // });
            }

            bb &= bb - 1;
        }
    }

    patterns
}