use crate::primitives::{consts::{MAX_PLY_PLUS_ONE}, shared::Move};


#[derive(Clone)]
pub struct PrincipalVariationTable {
    pub table: Box<[[Move; MAX_PLY_PLUS_ONE]]>,
    pub len: [usize; MAX_PLY_PLUS_ONE],
}

impl PrincipalVariationTable {
    pub const fn clear(&mut self, ply: usize) {
        self.len[ply] = 0;
    }

    pub fn update(&mut self, ply: usize, mv: Move) {
        self.table[ply][0] = mv;
        self.len[ply] = self.len[ply + 1] + 1;

        for i in 0..self.len[ply + 1] {
            self.table[ply][i + 1] = self.table[ply + 1][i];
        }
    }
}

impl Default for PrincipalVariationTable {
    fn default() -> Self {
        Self {
            table: vec![[Move::create_null(); MAX_PLY_PLUS_ONE]; MAX_PLY_PLUS_ONE].into_boxed_slice(),
            len: [0; MAX_PLY_PLUS_ONE],
        }
    }
}
