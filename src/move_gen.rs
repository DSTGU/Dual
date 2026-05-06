use crate::{
    get_bit, pop_bit, set_bit, BoardPosition, Piece, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS,
};
use crate::attacks::{get_bishop_attacks, get_queen_attacks, get_rook_attacks};
use crate::shared::Move;

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Castling rights mask indexed by square.  Each entry is a bitmask of the
/// castling rights that must be *preserved* when a piece leaves or lands on
/// that square.  A value of 15 (0b1111) means "no restriction".
pub const CASTLING_RIGHTS: [u8; 64] = [
    7, 15, 15, 15,  3, 15, 15, 11,
   15, 15, 15, 15, 15, 15, 15, 15,
   15, 15, 15, 15, 15, 15, 15, 15,
   15, 15, 15, 15, 15, 15, 15, 15,
   15, 15, 15, 15, 15, 15, 15, 15,
   15, 15, 15, 15, 15, 15, 15, 15,
   15, 15, 15, 15, 15, 15, 15, 15,
   13, 15, 15, 15, 12, 15, 15, 14,
];

// Pre-computed target squares for castling rook moves (source, target).
// Index: 0 = white kingside, 1 = white queenside, 2 = black kingside, 3 = black queenside.
// h1=63, f1=61, a1=56, d1=59, h8=7, f8=5, a8=0, d8=3
const CASTLING_ROOK_MOVES: [(usize, usize); 4] = [
    (63, 61), // White O-O:   h1 -> f1
    (56, 59), // White O-O-O: a1 -> d1
    (7, 5),   // Black O-O:   h8 -> f8
    (0, 3),   // Black O-O-O: a8 -> d8
];

// Piece index offsets: white pieces are 0..6, black pieces are 6..12.
// For a given side, the king bitboard index is:
//   side 0 (white) -> Piece::K = 5
//   side 1 (black) -> Piece::k = 11
const KING_INDEX: [usize; 2] = [Piece::K as usize, Piece::k as usize];

// ---------------------------------------------------------------------------
// is_square_attacked
// ---------------------------------------------------------------------------

/// Returns true if `square` is attacked by any piece of the side that is
/// **not** `board.side` (i.e. the side that is about to move is checking
/// whether the given square is under attack by the opponent).
pub fn is_square_attacked(square: usize, board: &BoardPosition) -> bool {
    // Check if the opponent of the side to move attacks this square.
    let opponent = 1 - board.side;
    let occ = board.occupancies[2];

    // Pawns: use PAWN_ATTACKS[board.side] which gives the reverse-attack
    // directions (i.e. squares from which an opponent pawn would attack).
    let pawn_bb = board.bitboards[if opponent == 0 {
        Piece::P as usize
    } else {
        Piece::p as usize
    }];
    if PAWN_ATTACKS[board.side][square] & pawn_bb != 0 {
        return true;
    }

    // Knights
    let knight_bb = board.bitboards[if opponent == 0 {
        Piece::N as usize
    } else {
        Piece::n as usize
    }];
    if KNIGHT_ATTACKS[square] & knight_bb != 0 {
        return true;
    }

    // Bishops
    let bishop_bb = board.bitboards[if opponent == 0 {
        Piece::B as usize
    } else {
        Piece::b as usize
    }];
    if get_bishop_attacks(square, occ) & bishop_bb != 0 {
        return true;
    }

    // Rooks
    let rook_bb = board.bitboards[if opponent == 0 {
        Piece::R as usize
    } else {
        Piece::r as usize
    }];
    if get_rook_attacks(square, occ) & rook_bb != 0 {
        return true;
    }

    // Queens
    let queen_bb = board.bitboards[if opponent == 0 {
        Piece::Q as usize
    } else {
        Piece::q as usize
    }];
    if get_queen_attacks(square, occ) & queen_bb != 0 {
        return true;
    }

    // Kings
    let king_bb = board.bitboards[if opponent == 0 {
        Piece::K as usize
    } else {
        Piece::k as usize
    }];
    if KING_ATTACKS[square] & king_bb != 0 {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Move generation helpers
// ---------------------------------------------------------------------------

/// Push a non-capture move into the list.
#[inline(always)]
fn push_move(moves: &mut Vec<Move>, source: usize, target: usize, piece: Piece, capture: u32) {
    moves.push(Move::create(
        source as u32,
        target as u32,
        piece,
        Piece::P,
        capture, 0, 0, 0,
    ));
}

/// Push a capture move into the list.
#[inline(always)]
fn push_promotion(moves: &mut Vec<Move>, source: usize, target: usize, piece: Piece, capture:u32, promoted: Piece) {
    moves.push(Move::create(
        source as u32,
        target as u32,
        piece,
        promoted,
        capture, 0, 0, 0,
    ));
}

/// Push a double-pawn-push move into the list.
#[inline(always)]
fn push_double_push(moves: &mut Vec<Move>, source: usize, target: usize, piece: Piece) {
    moves.push(Move::create(
        source as u32,
        target as u32,
        piece,
        Piece::P, // placeholder, not a real promotion
        0, 0, 0, 1,
    ));
}

/// Push an en-passant capture into the list.
#[inline(always)]
fn push_enpassant(moves: &mut Vec<Move>, source: usize, target: usize, piece: Piece) {
    moves.push(Move::create(
        source as u32,
        target as u32,
        piece,
        Piece::P, // placeholder
        1, 1, 0, 0,
    ));
}

/// Push a castling move into the list.
#[inline(always)]
fn push_castle(moves: &mut Vec<Move>, source: usize, target: usize, piece: Piece) {
    moves.push(Move::create(
        source as u32,
        target as u32,
        piece,
        Piece::P, // placeholder
        0, 0, 1, 0,
    ));
}

// ---------------------------------------------------------------------------
// Per-piece move generation (side-parameterised)
// ---------------------------------------------------------------------------

/// Generate all pawn moves for `side`.
fn generate_pawn_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::P } else { Piece::p };
    let promo_rank_range: (usize, usize) = if side == 0 { (8, 15) } else { (48, 55) };
    let start_rank_range: (usize, usize) = if side == 0 { (48, 55) } else { (8, 15) };
    let direction: isize = if side == 0 { -8 } else { 8 };
    let opp_occ = board.occupancies[1 - side];
    let all_occ = board.occupancies[2];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let target = (source as isize + direction) as usize;

        // Quiet moves (single push)
        if target < 64 && !get_bit(all_occ, target) {
            if source >= promo_rank_range.0 && source <= promo_rank_range.1 {
                // Promotion
                for promo in promotion_pieces(side) {
                    push_promotion(moves, source, target, piece, 0, promo);
                }
            } else {
                push_move(moves, source, target, piece, 0);

                // Double push
                if source >= start_rank_range.0 && source <= start_rank_range.1 {
                    let target2 = (target as isize + direction) as usize;
                    if target2 < 64 && !get_bit(all_occ, target2) {
                        push_double_push(moves, source, target2, piece);
                    }
                }
            }
        }

        // Captures
        let mut attacks = PAWN_ATTACKS[side][source] & opp_occ;
        while attacks != 0 {
            let cap_target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, cap_target);

            if source >= promo_rank_range.0 && source <= promo_rank_range.1 {
                for promo in promotion_pieces(side) {
                    push_promotion(moves, source, cap_target, piece, 1, promo);
                }
            } else {
                push_move(moves, source, cap_target, piece, 1);
            }
        }

        // En passant
        if board.enpassant < 64 {
            let ep_bit = PAWN_ATTACKS[side][source] & (1u64 << board.enpassant);
            if ep_bit != 0 {
                let ep_target = ep_bit.trailing_zeros() as usize;
                push_enpassant(moves, source, ep_target, piece);
            }
        }
    }
}

/// Generate all king moves (non-castling) for `side`.
fn generate_king_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::K } else { Piece::k };
    let our_occ = board.occupancies[side];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let mut attacks = KING_ATTACKS[source] & !our_occ;
        while attacks != 0 {
            let target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, target);

            let capture = get_bit(board.occupancies[1 - side], target) as u32; 
            push_move(moves, source, target, piece, capture);
        }
    }
}

/// Generate castling moves for `side`.
fn generate_castling_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::K } else { Piece::k };
    let occ = board.occupancies[2];

    if side == 0 {
        // White kingside (O-O): king e1->g1, rook h1->f1
        if board.castle & 1 != 0
            && !get_bit(occ, 61)
            && !get_bit(occ, 62)
            && !is_square_attacked(60, board)
            && !is_square_attacked(61, board)
        {
            push_castle(moves, 60, 62, piece);
        }
        // White queenside (O-O-O): king e1->c1, rook a1->d1
        if board.castle & 2 != 0
            && !get_bit(occ, 59)
            && !get_bit(occ, 58)
            && !get_bit(occ, 57)
            && !is_square_attacked(60, board)
            && !is_square_attacked(59, board)
        {
            push_castle(moves, 60, 58, piece);
        }
    } else {
        // Black kingside (O-O): king e8->g8, rook h8->f8
        if board.castle & 4 != 0
            && !get_bit(occ, 5)
            && !get_bit(occ, 6)
            && !is_square_attacked(4, board)
            && !is_square_attacked(5, board)
        {
            push_castle(moves, 4, 6, piece);
        }
        // Black queenside (O-O-O): king e8->c8, rook a8->d8
        if board.castle & 8 != 0
            && !get_bit(occ, 3)
            && !get_bit(occ, 2)
            && !get_bit(occ, 1)
            && !is_square_attacked(4, board)
            && !is_square_attacked(3, board)
        {
            push_castle(moves, 4, 2, piece);
        }
    }
}

/// Generate all knight moves for `side`.
fn generate_knight_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::N } else { Piece::n };
    let our_occ = board.occupancies[side];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let mut attacks = KNIGHT_ATTACKS[source] & !our_occ;
        while attacks != 0 {
            let target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, target);

            let capture = get_bit(board.occupancies[1 - side], target) as u32;
            push_move(moves, source, target, piece, capture);
        }
    }
}

/// Generate all bishop moves for `side`.
fn generate_bishop_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::B } else { Piece::b };
    let our_occ = board.occupancies[side];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let mut attacks = get_bishop_attacks(source, board.occupancies[2]) & !our_occ;
        while attacks != 0 {
            let target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, target);

            let capture = get_bit(board.occupancies[1 - side], target) as u32;
            push_move(moves, source, target, piece, capture);
        }
    }
}

/// Generate all rook moves for `side`.
fn generate_rook_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::R } else { Piece::r };
    let our_occ = board.occupancies[side];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let mut attacks = get_rook_attacks(source, board.occupancies[2]) & !our_occ;
        while attacks != 0 {
            let target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, target);

            let capture = get_bit(board.occupancies[1 - side], target) as u32;
            push_move(moves, source, target, piece, capture);
        }
    }
}

/// Generate all queen moves for `side`.
fn generate_queen_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let piece = if side == 0 { Piece::Q } else { Piece::q };
    let our_occ = board.occupancies[side];

    let mut bb = board.bitboards[piece as usize];
    while bb != 0 {
        let source = bb.trailing_zeros() as usize;
        pop_bit(&mut bb, source);

        let mut attacks = get_queen_attacks(source, board.occupancies[2]) & !our_occ;
        while attacks != 0 {
            let target = attacks.trailing_zeros() as usize;
            pop_bit(&mut attacks, target);

            let capture = get_bit(board.occupancies[1 - side], target) as u32; 
            push_move(moves, source, target, piece, capture);
        }
    }
}

/// Return the four promotion piece types for the given side.
#[inline(always)]
fn promotion_pieces(side: usize) -> [Piece; 4] {
    if side == 0 {
        [Piece::Q, Piece::R, Piece::B, Piece::N]
    } else {
        [Piece::q, Piece::r, Piece::n, Piece::b]
    }
}

// ---------------------------------------------------------------------------
// Public API: generate_moves
// ---------------------------------------------------------------------------

/// Generate all pseudo-legal moves for the side to move.
pub fn generate_moves(board: &BoardPosition) -> Vec<Move> {
    let side = board.side;
    // Typical legal positions have ~35 moves; 64 avoids most reallocations.
    let mut moves = Vec::with_capacity(64);

    generate_pawn_moves(board, side, &mut moves);
    generate_king_moves(board, side, &mut moves);
    generate_castling_moves(board, side, &mut moves);
    generate_knight_moves(board, side, &mut moves);
    generate_bishop_moves(board, side, &mut moves);
    generate_rook_moves(board, side, &mut moves);
    generate_queen_moves(board, side, &mut moves);

    moves
}

// ---------------------------------------------------------------------------
// make_move
// ---------------------------------------------------------------------------

/// Apply `move_to_make` to `board` and return the resulting position, or
/// `None` if the move leaves the king in check.
pub fn make_move(board: &BoardPosition, move_to_make: &Move) -> Option<BoardPosition> {
    let mut new = BoardPosition {
        bitboards: board.bitboards,
        occupancies: board.occupancies,
        side: board.side,
        enpassant: board.enpassant,
        castle: board.castle,
    };

    let piece_idx = move_to_make.get_piece() as usize;
    let source = move_to_make.get_source_square() as usize;
    let target = move_to_make.get_target_square() as usize;
    let is_capture = move_to_make.get_capture();
    let is_enpassant = move_to_make.get_enpassant();
    let is_castling = move_to_make.get_castling();
    let is_double_push = move_to_make.get_double_pawn_push();
    let promoted = move_to_make.get_promoted();

    // Move the piece: clear source, set target
    pop_bit(&mut new.bitboards[piece_idx], source);
    set_bit(&mut new.bitboards[piece_idx], target);

    // Handle captures: remove the captured piece from opponent's bitboards.
    if is_capture {
        let opp_side = 1 - new.side;
        let opp_occ = new.occupancies[opp_side];

        // Iterate only over opponent piece types that actually occupy the
        // target square.  This is faster than blindly clearing all 6 slots.
        let base = opp_side * 6;
        for i in 0..6 {
            let idx = base + i;
            if get_bit(opp_occ, target) {
                pop_bit(&mut new.bitboards[idx], target);
            }
        }
        pop_bit(&mut new.occupancies[opp_side], target);
    }

    // Handle promotion: replace the pawn with the promoted piece.
    if promoted != 0 {
        pop_bit(&mut new.bitboards[piece_idx], target);
        set_bit(&mut new.bitboards[promoted as usize], target);
    }

    // Handle en passant: remove the captured pawn (which is on a different
    // square from the target).
    if is_enpassant {
        if piece_idx < 6 {
            // White pawn captured a black pawn on the rank below target.
            pop_bit(&mut new.bitboards[Piece::p as usize], target + 8);
        } else {
            // Black pawn captured a white pawn on the rank above target.
            pop_bit(&mut new.bitboards[Piece::P as usize], target - 8);
        }
    }

    // Reset en passant square; set it again if this was a double pawn push.
    new.enpassant = 64;
    if is_double_push {
        if piece_idx < 6 {
            new.enpassant = target + 8;
        } else {
            new.enpassant = target - 8;
        }
    }

    // Handle castling: move the rook.
    if is_castling {
        match target {
            // White kingside
            62 => {
                pop_bit(&mut new.bitboards[Piece::R as usize], CASTLING_ROOK_MOVES[0].0);
                set_bit(&mut new.bitboards[Piece::R as usize], CASTLING_ROOK_MOVES[0].1);
            }
            // White queenside
            58 => {
                pop_bit(&mut new.bitboards[Piece::R as usize], CASTLING_ROOK_MOVES[1].0);
                set_bit(&mut new.bitboards[Piece::R as usize], CASTLING_ROOK_MOVES[1].1);
            }
            // Black kingside
            6 => {
                pop_bit(&mut new.bitboards[Piece::r as usize], CASTLING_ROOK_MOVES[2].0);
                set_bit(&mut new.bitboards[Piece::r as usize], CASTLING_ROOK_MOVES[2].1);
            }
            // Black queenside
            2 => {
                pop_bit(&mut new.bitboards[Piece::r as usize], CASTLING_ROOK_MOVES[3].0);
                set_bit(&mut new.bitboards[Piece::r as usize], CASTLING_ROOK_MOVES[3].1);
            }
            _ => {}
        }
    }

    // Update castling rights.
    new.castle &= CASTLING_RIGHTS[source] as usize;
    new.castle &= CASTLING_RIGHTS[target] as usize;

    // Recompute occupancies.
    new.occupancies[0] = new.bitboards[0..6].iter().fold(0, |acc, &b| acc | b);
    new.occupancies[1] = new.bitboards[6..12].iter().fold(0, |acc, &b| acc | b);
    new.occupancies[2] = new.occupancies[0] | new.occupancies[1];

    // Find the king of the side that just moved to check for legality.
    // is_square_attacked checks if the opponent of board.side attacks,
    // so we need board.side to be the mover's side.
    let king_idx = KING_INDEX[new.side];
    let king_sq = new.bitboards[king_idx].trailing_zeros() as usize;

    // Temporarily set side to the mover's side for the attack check
    // (new.side is already board.side from initialization).
    // is_square_attacked will check if the opponent (1 - new.side) attacks king_sq.

    if is_square_attacked(king_sq, &new) {
        return None;
    }

    // Flip side for the returned position.
    new.side = 1 - new.side;

    Some(new)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::move_gen::is_square_attacked;
    use crate::perft::perft_driver;
    use crate::shared::{
        coordinates_to_squares, parse_fen, print_bitboard, BoardPosition,
        ENDGAME_PERFT, KIWIPETE, START_POSITION,
    };
    use std::thread;

    pub fn run_through_attacks(board_position: &BoardPosition) -> u64 {
        let mut cnt = 0;
        for y in 0..8 {
            for x in 0..8 {
                cnt = cnt * 2;
                if is_square_attacked(x + 8 * y, board_position) {
                    cnt += 1;
                }
            }
        }

        print_bitboard(cnt);

        cnt
    }

    #[test]
    fn test_attacked_squares_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos =
                    parse_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
                assert_eq!(run_through_attacks(&board_pos), 18437032593966828032);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_rook_attacks_true() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = parse_fen("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
                assert_eq!(
                    is_square_attacked(coordinates_to_squares("d3"), &board_pos),
                    true
                );
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_rook_attacks_false() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = parse_fen("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
                assert_eq!(
                    is_square_attacked(coordinates_to_squares("b1"), &board_pos),
                    false
                );
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_perft_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = parse_fen(KIWIPETE); //Rook on e3
                let movecnt = perft_driver(&board_pos, 5);
                assert_eq!(movecnt, 193690690);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_perft_endgame() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = parse_fen(ENDGAME_PERFT); //Rook on e3
                let movecnt = perft_driver(&board_pos, 6);
                assert_eq!(movecnt, 11030083);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_perft_startpos_intermediate_depths() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                // These are the expected perft results for each depth from startpos
                let expected = [20, 400, 8902, 197281, 4865609, 119060324];
                let board_pos = parse_fen(START_POSITION);
                for (depth, &exp) in expected.iter().enumerate() {
                    let movecnt = perft_driver(&board_pos, depth + 1);
                    assert_eq!(movecnt, exp, "Perft mismatch at depth {}", depth + 1);
                }
            })
            .unwrap();
        handler.join().unwrap();
    }
}
