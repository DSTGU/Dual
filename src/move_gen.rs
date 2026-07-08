use crate::types::board::BoardPosition;
use crate::{
    get_bit, pop_bit, Piece, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS,
};
use crate::attacks::{get_bishop_attacks, get_queen_attacks, get_rook_attacks};
use crate::shared::{Move, MoveCode};

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

// ---------------------------------------------------------------------------
// is_square_attacked
// ---------------------------------------------------------------------------

/// Returns true if `square` is attacked by any piece of the side that is
/// **not** `board.side` (i.e. the side that is about to move is checking
/// whether the given square is under attack by the opponent).
pub fn is_square_attacked(square: u8, board: &BoardPosition) -> bool {
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
    if PAWN_ATTACKS[board.side][square as usize] & pawn_bb != 0 {
        return true;
    }

    // Knights
    let knight_bb = board.bitboards[if opponent == 0 {
        Piece::N as usize
    } else {
        Piece::n as usize
    }];
    if KNIGHT_ATTACKS[square as usize] & knight_bb != 0 {
        return true;
    }

    // Bishops
    let bishop_bb = board.bitboards[if opponent == 0 {
        Piece::B as usize
    } else {
        Piece::b as usize
    }];
    if get_bishop_attacks(square as usize, occ) & bishop_bb != 0 {
        return true;
    }

    // Rooks
    let rook_bb = board.bitboards[if opponent == 0 {
        Piece::R as usize
    } else {
        Piece::r as usize
    }];
    if get_rook_attacks(square as usize, occ) & rook_bb != 0 {
        return true;
    }

    // Queens
    let queen_bb = board.bitboards[if opponent == 0 {
        Piece::Q as usize
    } else {
        Piece::q as usize
    }];
    if get_queen_attacks(square as usize, occ) & queen_bb != 0 {
        return true;
    }

    // Kings
    let king_bb = board.bitboards[if opponent == 0 {
        Piece::K as usize
    } else {
        Piece::k as usize
    }];
    if KING_ATTACKS[square as usize] & king_bb != 0 {
        return true;
    }

    false
}

// ---------------------------------------------------------------------------
// Move generation helpers
// ---------------------------------------------------------------------------
#[inline(always)]
fn push_move(moves: &mut Vec<Move>, source: u8, target: u8, move_code: MoveCode, current_enpassant: u8, current_castle: usize, taken_piece : Piece) {
    let new_move = Move::create(
        source,
        target,
        move_code,
        current_enpassant, current_castle, taken_piece
    );
    moves.push(new_move);
}

// ---------------------------------------------------------------------------
// Per-piece move generation (side-parameterised)
// ---------------------------------------------------------------------------

/// Generate all pawn moves for `side`.
fn generate_pawn_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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
                for promo in promotion_codes() {
                    push_move(moves, source as u8, target as u8, promo, board.enpassant, board.castle, Piece::NONE);
                }
            } else {
                if !quiescence {
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                    
                    // Double push
                    if source >= start_rank_range.0 && source <= start_rank_range.1 {
                        let target2 = (target as isize + direction) as usize;
                        if target2 < 64 && !get_bit(all_occ, target2) {
                            push_move(moves, source as u8, target2 as u8,MoveCode::DoublePush, board.enpassant, board.castle, Piece::NONE);
                        }
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
                for promo in promotion_capture_codes() {
                    push_move(moves, source as u8, cap_target as u8, promo, board.enpassant, board.castle, board.find_capture_at_square(cap_target));
                }
            } else {
                push_move(moves, source as u8, cap_target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(cap_target));
            }
        }

        // En passant
        if board.enpassant != 0 {
            let ep_bit = PAWN_ATTACKS[side][source] & (1u64 << board.enpassant);
            if ep_bit != 0 {
                let ep_target = ep_bit.trailing_zeros() as u8;
                push_move(moves, source as u8, ep_target as u8, MoveCode::EnPassant, board.enpassant, board.castle, Piece::NONE);
            }
        }
    }
}

/// Generate all king moves (non-castling) for `side`.
fn generate_king_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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

            if get_bit(board.occupancies[1 - side], target) {
                push_move(moves, source as u8, target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(target));
            } else {
                if !quiescence {
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                }
            }
        }
    }
}

/// Generate castling moves for `side`.
fn generate_castling_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
) {
    let occ = board.occupancies[2];

    if side == 0 {
        // White kingside (O-O): king e1->g1, rook h1->f1
        if board.castle & 1 != 0
            && !get_bit(occ, 61)
            && !get_bit(occ, 62)
            && !is_square_attacked(60, board)
            && !is_square_attacked(61, board)
        {
            push_move(moves, 60, 62, MoveCode::KingCastle, board.enpassant, board.castle, Piece::NONE);
        }
        // White queenside (O-O-O): king e1->c1, rook a1->d1
        if board.castle & 2 != 0
            && !get_bit(occ, 59)
            && !get_bit(occ, 58)
            && !get_bit(occ, 57)
            && !is_square_attacked(60, board)
            && !is_square_attacked(59, board)
        {
            push_move(moves, 60, 58, MoveCode::QueenCastle, board.enpassant, board.castle, Piece::NONE);
        }
    } else {
        // Black kingside (O-O): king e8->g8, rook h8->f8
        if board.castle & 4 != 0
            && !get_bit(occ, 5)
            && !get_bit(occ, 6)
            && !is_square_attacked(4, board)
            && !is_square_attacked(5, board)
        {
            push_move(moves, 4, 6, MoveCode::KingCastle, board.enpassant, board.castle, Piece::NONE);
        }
        // Black queenside (O-O-O): king e8->c8, rook a8->d8
        if board.castle & 8 != 0
            && !get_bit(occ, 3)
            && !get_bit(occ, 2)
            && !get_bit(occ, 1)
            && !is_square_attacked(4, board)
            && !is_square_attacked(3, board)
        {
            push_move(moves, 4, 2, MoveCode::QueenCastle, board.enpassant, board.castle, Piece::NONE);
        }
    }
}

/// Generate all knight moves for `side`.
fn generate_knight_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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
            
            if get_bit(board.occupancies[1 - side], target) {
                push_move(moves, source as u8, target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(target));
            } else {
                if !quiescence {
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                }
            }
        }
    }
}

/// Generate all bishop moves for `side`.
fn generate_bishop_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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

            if get_bit(board.occupancies[1 - side], target) {
                push_move(moves, source as u8, target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(target));
            } else {
                if !quiescence {   
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                }
            }
        }
    }
}

/// Generate all rook moves for `side`.
fn generate_rook_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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

            if get_bit(board.occupancies[1 - side], target) {
                push_move(moves, source as u8, target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(target));
            } else {
                if !quiescence {   
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                }
            }
        }
    }
}

/// Generate all queen moves for `side`.
fn generate_queen_moves(
    board: &BoardPosition,
    side: usize,
    moves: &mut Vec<Move>,
    quiescence: bool
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

            if get_bit(board.occupancies[1 - side], target) {
                push_move(moves, source as u8, target as u8, MoveCode::Capture, board.enpassant, board.castle, board.find_capture_at_square(target));
            } else {
                if !quiescence {
                    push_move(moves, source as u8, target as u8, MoveCode::QuietMove, board.enpassant, board.castle, Piece::NONE);
                }
            }
        }
    }
}

// /// Return the four promotion piece types for the given side.
// #[inline(always)]
// fn promotion_pieces(side: usize) -> [Piece; 4] {
//     if side == 0 {
//         [Piece::Q, Piece::R, Piece::B, Piece::N]
//     } else {
//         [Piece::q, Piece::r, Piece::n, Piece::b]
//     }
// }

#[inline(always)]
fn promotion_codes() -> [MoveCode; 4] {
    [MoveCode::KnightPromotion, MoveCode::BishopPromotion, MoveCode::RookPromotion, MoveCode::QueenPromotion]
}

#[inline(always)]
fn promotion_capture_codes() -> [MoveCode; 4] {
    [MoveCode::KnightPromotionCapture, MoveCode::BishopPromotionCapture, MoveCode::RookPromotionCapture, MoveCode::QueenPromotionCapture]
}

// ---------------------------------------------------------------------------
// Public API: generate_moves
// ---------------------------------------------------------------------------

/// Generate all pseudo-legal moves for the side to move.
pub fn generate_moves(board: &BoardPosition, quiescence: bool) -> Vec<Move> {
    let side = board.side;
    // Typical legal positions have ~35 moves; 64 avoids most reallocations.
    let mut moves = Vec::with_capacity(if quiescence { 64 } else { 16 });

    generate_pawn_moves(board, side, &mut moves, quiescence);
    generate_king_moves(board, side, &mut moves, quiescence);
    if !quiescence {
        generate_castling_moves(board, side, &mut moves);
    }
    generate_knight_moves(board, side, &mut moves, quiescence);
    generate_bishop_moves(board, side, &mut moves, quiescence);
    generate_rook_moves(board, side, &mut moves, quiescence);
    generate_queen_moves(board, side, &mut moves, quiescence);

    moves
}


// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use crate::types::board::BoardPosition;
    use crate::move_gen::is_square_attacked;
    use crate::shared::{coordinates_to_squares, print_bitboard};
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
                    BoardPosition::new("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -");
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
                let board_pos = BoardPosition::new("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
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
                let board_pos = BoardPosition::new("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
                assert_eq!(
                    is_square_attacked(coordinates_to_squares("b1"), &board_pos),
                    false
                );
            })
            .unwrap();
        handler.join().unwrap();
    }
}
