use crate::gui::parse_move;
use crate::types::board::BoardPosition;
use crate::move_gen::{is_square_attacked};
use crate::shared::{FIRST_KILLER_BONUS, KIWIPETE, MVV_LVA, Move, MoveSuccess, PV_MOVE_BONUS, Piece, SECOND_KILLER_BONUS, START_POSITION};
use crate::types::tt::{RepetitionTable, TTEntry, TTFlag, TranspositionTable, compute_hash};

/// Search state structure - encapsulates all search-related state
pub struct SearchState {
    pub board_position: BoardPosition,
    pub max_depth: usize,
    pub seldepth: usize,
    killer_moves: [[Move; 256]; 2],
    history_moves: [[usize; 64]; 12],
    prev_iter_best_move: Move,
    tt: TranspositionTable,
    rep_table: RepetitionTable,
    nodes_searched: u64,
    tt_hits: u64,
}

impl SearchState {
    pub fn new(fen : &str) -> Self {
        let board_position = BoardPosition::new(fen);
        let search_state = Self {
            board_position: board_position,
            max_depth: 0,
            seldepth: 0,
            killer_moves: [[Move::create_null(); 256]; 2],
            history_moves: [[0; 64]; 12],
            prev_iter_best_move: Move::create_null(),
            tt: TranspositionTable::new(),
            rep_table: RepetitionTable::new(),
            nodes_searched: 0,
            tt_hits: 0,
        };

        search_state
    }

    // This function was moved here to preserve TT between nodes
    pub fn change_position(&mut self, fen: &str) {
        let board_position = BoardPosition::new(fen);
        self.board_position = board_position;
        self.max_depth = 0;
        self.seldepth = 0;
        self.killer_moves = [[Move::create_null(); 256]; 2];
        self.history_moves = [[0; 64]; 12];
        self.prev_iter_best_move = Move::create_null();
        self.rep_table.clear();
        self.nodes_searched = 0;
    }


    pub fn parse_position_command(&mut self, command: &str) {
        let words : Vec<&str> = command.trim().split(" ").collect();

        if words.len() < 2 {
            self.change_position(START_POSITION);
            return;
        }

        match words[1] {
            "fen" => {
                self.change_position(&command[13..]);
                if words.len() > 8 {
                    for &i in words[9..].iter() {
                        let mov = parse_move(&self.board_position, i);
                        if let Some(x) = mov {
                            self.make_move(x);
                        }
                    }
                }
            },
            "startpos" => {
                self.change_position(START_POSITION);
                for &i in words[2..].iter() {
                    let mov = parse_move(&self.board_position, i);
                    if let Some(x) = mov {
                        self.make_move(x);
                    }
                }
            },
            "kiwipete" => {
                self.change_position(KIWIPETE);
                for &i in words[2..].iter() {
                    let mov = parse_move(&self.board_position, i);
                    if let Some(x) = mov {
                        self.make_move(x);
                    }
                }
            },
            _ => self.change_position(START_POSITION),
        }

    }

    pub fn reset_for_new_search(&mut self, depth: usize, previter_bestmove: Move) {
        self.max_depth = depth;
        self.seldepth = depth;
        self.prev_iter_best_move = previter_bestmove;
        self.tt.increment_age();
        self.killer_moves = [[Move::create_null(); 256]; 2];
        self.history_moves = [[0; 64]; 12];
        self.nodes_searched = 0;
    }

    pub fn make_move(&mut self, move_to_make: Move) -> MoveSuccess {
        self.rep_table.push(self.board_position.hash); 
        let result = self.board_position.make_move(move_to_make);
        if result == MoveSuccess::Attacked {
            self.rep_table.pop();
        }

        result
    }

    pub fn take_back(&mut self, move_to_take_back: Move) {
        //take back manages the hash        
        self.board_position.take_back(move_to_take_back, self.rep_table.pop());
        
        debug_assert!(compute_hash(&self.board_position) == self.board_position.hash)
    }

    #[inline(always)]
    fn get_mvv_lva(victim: Piece, attacker: Piece) -> usize {
        MVV_LVA[victim as usize % 6 + attacker as usize % 6 * 6]
    }

    fn get_victim(&self, mv: Move) -> Piece {
        mv.get_taken_piece()
    }

    //Only works before move
    fn get_piece(&self, mv: Move) -> Piece {
        self.board_position.mailbox[mv.get_source_square() as usize]
    }

    pub fn get_move_score(&self, mv: Move, ply: usize) -> usize {
        // PV move from previous iteration gets highest priority
        
        if ply == 0 && mv == self.prev_iter_best_move {
            return PV_MOVE_BONUS;
        }

        if mv.is_capture() {
            let victim = self.get_victim(mv);
            return Self::get_mvv_lva(victim, self.get_piece(mv));
        }

        // Killer moves
        if self.killer_moves[0][ply] == mv {
            return FIRST_KILLER_BONUS;
        }
        if self.killer_moves[1][ply] == mv {
            return SECOND_KILLER_BONUS;
        }

        // History heuristic
        self.history_moves[self.get_piece(mv) as usize][mv.get_target_square() as usize]
    }

    pub fn update_killer_move(&mut self, mv: Move, ply: usize) {
        let idx = self.max_depth.saturating_sub(ply);
        if idx < 256 {
            self.killer_moves[1][idx] = self.killer_moves[0][idx];
            self.killer_moves[0][idx] = mv;
        }
    }

    pub fn update_history(&mut self, mv: Move, depth: usize) {
        let piece = self.get_piece(mv) as usize;
        let target = mv.get_target_square() as usize;
        if piece < 12 && target < 64 {
            self.history_moves[piece][target] += depth;
        }
    }

    // pub fn get_stats(&self) -> (u64, u64, f64) {
    //     let fill_pct = self.tt.fill_percentage();
    //     (self.nodes_searched, self.tt_hits, fill_pct)
    // }

    pub fn is_trifold_repetition(&self) -> bool {
        self.rep_table.is_draw(self.board_position.hash)
    }

    pub fn is_king_attacked(&self) -> bool {
        is_square_attacked(self.board_position.bitboards[6*self.board_position.side+5].trailing_zeros() as u8, &self.board_position)
    }

    #[inline(always)]
    pub fn probe_tt(&mut self) -> Option<&TTEntry> {
        self.tt.probe(self.board_position.hash)
    }

    #[inline(always)]
    pub fn store_tt(
        &mut self,
        depth: usize,
        score: i32,
        flag: TTFlag,
        best_move: Move,
    ) {
        self.tt.store(
            self.board_position.hash,
            depth as i32,
            score,
            flag,
            best_move, // or .into()
        );
    }

    #[inline(always)]
    pub fn tt_move(&mut self) -> Option<Move> {
        self.tt
            .probe(self.board_position.hash)
            .and_then(|e| {
                if !e.best_move.is_null() {
                    Some(e.best_move)
                } else {
                    None
                }
            })
    }

    pub fn get_tt_stats(&self) -> (u64, u64, u64, u64) {
        self.tt.stats()
    }

}

impl Default for SearchState {
    fn default() -> Self {
        Self::new(START_POSITION)
    }
}