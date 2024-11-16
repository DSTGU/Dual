use std::borrow::Borrow;
use std::ops::BitXor;
use crate::{BoardPosition, get_bit, KING_ATTACKS, KNIGHT_ATTACKS, PAWN_ATTACKS, Piece, pop_bit, print_bitboard, print_board, set_bit};
use crate::attacks::{get_bishop_attacks, get_queen_attacks, get_rook_attacks};
use crate::shared::{Move, pieceTousize, SQUARE_TO_COORDINATES, coordinates_to_squares, print_square};
use crate::shared::Piece::{k, r, K, R};

pub fn is_square_attacked(square: usize, board: &BoardPosition) -> bool {
    // Attacked by white pawns
    if board.side == 1 && (PAWN_ATTACKS[1][square] & board.bitboards[Piece::P as usize]) != 0 {
        return true;
    }

    // Attacked by black pawns
    if board.side == 0 && (PAWN_ATTACKS[0][square] & board.bitboards[Piece::p as usize]) != 0 {
        return true;
    }

    // Attacked by knights
    if KNIGHT_ATTACKS[square] & (if board.side == 0 { board.bitboards[Piece::n as usize] } else { board.bitboards[Piece::N as usize] }) != 0 {
        return true;
    }


    // Attacked by bishops
    if get_bishop_attacks(square, board.occupancies[2]) & (if board.side == 0 { board.bitboards[Piece::b as usize] } else { board.bitboards[Piece::B as usize] }) != 0 {
        return true;
    }


    // Attacked by rooks
    if get_rook_attacks(square, board.occupancies[2]) & (if board.side == 0 { board.bitboards[Piece::r as usize] } else { board.bitboards[Piece::R as usize] }) != 0 {
        return true;
    }

    // Attacked by queens
    if get_queen_attacks(square, board.occupancies[2]) & (if board.side == 0 { board.bitboards[Piece::q as usize] } else { board.bitboards[Piece::Q as usize] }) != 0 {
        return true;
    }

    // Attacked by kings
    if KING_ATTACKS[square] & (if board.side == 0 { board.bitboards[Piece::k as usize] } else { board.bitboards[Piece::K as usize] }) != 0 {

        return true;

    }

    // By default, return false
    false
}


pub fn run_through_attacks(board_position: &BoardPosition) -> u64 {
    
    let mut cnt = 0;
    for y in 0..8 {
        for x in 0..8{
            
            //println!("Square: {}, coordinate: {}, Attacked: {}", x+8*y, SQUARE_TO_COORDINATES[x+8*y], is_square_attacked(x+8*y, board_position));
            cnt = cnt * 2;
            if is_square_attacked(x+8*y, board_position) {
                cnt += 1;
            }
        }
    }

    // for y in 0..8 {
    //         println!("{} {} {} {} {} {} {} {}", is_square_attacked(8*y, board_position) as usize, is_square_attacked(8*y+1, board_position) as usize, is_square_attacked(8*y+2, board_position) as usize, is_square_attacked(8*y+3, board_position) as usize, is_square_attacked(8*y+4, board_position) as usize, is_square_attacked(8*y+5, board_position) as usize, is_square_attacked(8*y+6, board_position) as usize, is_square_attacked(8*y+7, board_position) as usize)
    // }
    
    print_bitboard(cnt);
    cnt

}

pub fn generate_moves(board: &BoardPosition) -> Vec<Move> {
    // Define source and target squares
    let mut source_square: usize;
    let mut target_square: usize;

    // Define current piece's bitboard copy and its attacks
    let mut bitboard;
    let mut attacks: u64;
    
    let mut move_list: Vec<Move> = Vec::new();
    if board.side == 0
    {
            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::P as usize].clone();
            // Loop over white pawns within white pawn bitboard
            while bitboard != 0 {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                // Init target square
                let target_square = source_square.wrapping_sub(8) as usize;

                // Generate quiet pawn moves
                if !(target_square > 63) && !get_bit(board.occupancies[2], target_square as usize) {
                    // Pawn promotion
                    if source_square >= 8 && source_square <= 15 {
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::Q, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::R, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::N, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::B, ..Default::default()});
                    } else {
                        // One square ahead pawn move
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, ..Default::default()});

                        // Two squares ahead pawn move
                        if source_square >= 48 && source_square <= 55 && !get_bit(board.occupancies[2], (target_square - 8) as usize) {
                            move_list.push(Move {source_square: source_square as u8, target_square: (target_square - 8) as u8, double_push: true, ..Default::default()});
                        }
                    }
                }

                // Init pawn attacks bitboard
                attacks = PAWN_ATTACKS[board.side][source_square] & board.occupancies[1];

                // Generate pawn captures
                while attacks != 0 {
                    // Init target square
                    let target_square = attacks.trailing_zeros() as usize;

                    // Pawn promotion
                    if source_square >= 8 && source_square <= 15 {

                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::Q, capture: true, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::R, capture: true, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::N, capture: true, ..Default::default()});
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, promoted_piece: Piece::B, capture: true, ..Default::default()});

                    } else {
                        // One square ahead pawn move
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::P, capture:true,  ..Default::default()});
                    }

                    // Pop ls1b of the pawn attacks
                    pop_bit(&mut attacks, target_square);
                }

                // Generate en passant captures
                if board.enpassant < 64 {
                    // Lookup pawn attacks and bitwise AND with enpassant square (bit)
                    let enpassant_attacks = PAWN_ATTACKS[board.side][source_square] & (1u64 << board.enpassant);

                    // Make sure enpassant capture available
                    if enpassant_attacks != 0 {
                        // Init enpassant capture target square
                        let target_enpassant = enpassant_attacks.trailing_zeros() as usize;
                        move_list.push(Move {source_square: source_square as u8, target_square: target_enpassant as u8, piece: Piece::P, capture: true, enpassant: true, ..Default::default()});
                    }
                }

                // Pop ls1b from piece bitboard copy
                pop_bit(&mut bitboard, source_square as usize);
            }

            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::K as usize].clone();
            while bitboard != 0 {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                let mut attacks = KING_ATTACKS[source_square] & !board.occupancies[0];

                while attacks != 0 {
                    let target_square = attacks.trailing_zeros() as usize;
                    // One square ahead pawn move
                    if get_bit(board.occupancies[1], target_square){
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::K, capture: true, ..Default::default()});
                    }else {
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::K, ..Default::default()});
                    }
                    pop_bit(&mut attacks, target_square);
                }
                pop_bit(&mut bitboard, source_square);
            }

            // King side castling is available
            if board.castle & 1 != 0 {
                // Make sure squares between king and king's rook are empty
                if !get_bit(board.occupancies[2], 61) && !get_bit(board.occupancies[2], 62) {
                    // Make sure king and the f1 squares are not under attack
                    if !is_square_attacked(60, &board) && !is_square_attacked(61, board) {
                        move_list.push(Move {source_square: 60, target_square: 62, piece: Piece::K, castling: true, ..Default::default()});
                    }
                }
            }

            // Queen side castling is available
            if board.castle & 2 != 0 {
                // Make sure squares between king and queen's rook are empty
                if !get_bit(board.occupancies[2], 59) && !get_bit(board.occupancies[2], 58) && !get_bit(board.occupancies[2], 57) {
                    // Make sure king and the d1 squares are not under attack
                    if !is_square_attacked(60, board) && !is_square_attacked(59, board) {
                        move_list.push(Move {source_square: 60, target_square: 58, piece: Piece::K, castling: true, ..Default::default()});
                    }
                }
            }


            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::N as usize].clone();
            while bitboard != 0
            {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                let mut attacks = KNIGHT_ATTACKS[source_square] & !board.occupancies[0];

                while attacks != 0 {
                    let target_square = attacks.trailing_zeros() as usize;
                    if get_bit(board.occupancies[1], target_square){
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::N, capture: true, ..Default::default()});
                    } else {
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::N, ..Default::default() });
                    }
                    pop_bit(&mut attacks, target_square);
                }
                pop_bit(&mut bitboard, source_square);
            }

            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::B as usize].clone();
            while bitboard != 0
            {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                let mut attacks = get_bishop_attacks(source_square,board.occupancies[2]) & !board.occupancies[0];

                while attacks != 0 {
                    let target_square = attacks.trailing_zeros() as usize;
                    // One square ahead pawn move

                    if get_bit(board.occupancies[1], target_square){
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::B, capture: true, ..Default::default()});
                    } else {
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::B, ..Default::default() });
                    }
                    pop_bit(&mut attacks, target_square);
                }
                pop_bit(&mut bitboard, source_square);
            }

            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::R as usize].clone();
            while bitboard != 0
            {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                let mut attacks = get_rook_attacks(source_square,board.occupancies[2]) & !board.occupancies[0];

                while attacks != 0 {
                    let target_square = attacks.trailing_zeros() as usize;
                    // One square ahead pawn move
                    if get_bit(board.occupancies[1], target_square){
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::R, capture: true, ..Default::default()});
                    } else {
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::R, ..Default::default() });
                    }
                    pop_bit(&mut attacks, target_square);
                }
                pop_bit(&mut bitboard, source_square);
            }

            // Init piece bitboard copy
            bitboard = board.bitboards[Piece::Q as usize].clone();
            while bitboard != 0
            {
                // Init source square
                let source_square = bitboard.trailing_zeros() as usize;

                let mut attacks = get_queen_attacks(source_square,board.occupancies[2]) & !board.occupancies[0];

                while attacks != 0 {
                    let target_square = attacks.trailing_zeros() as usize;
                    // One square ahead pawn move
                    if get_bit(board.occupancies[1], target_square){
                        move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::Q, capture: true, ..Default::default()});
                    } else {
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::Q, ..Default::default() });
                    }
                    pop_bit(&mut attacks, target_square);
                }
                pop_bit(&mut bitboard, source_square);
            }

        }
    else {
        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::p as usize].clone();
        // Loop over black pawns within white pawn bitboard
        while bitboard != 0 {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            // Init target square
            let target_square = source_square + 8 as usize;

            // Generate quiet pawn moves
            if !(target_square > 63) && !get_bit(board.occupancies[2], target_square as usize) {
                // Pawn promotion
                if source_square >= 48 && source_square <= 55 {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::q, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::r, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::n, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::b, ..Default::default()});


                } else {
                    // One square ahead pawn move
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, ..Default::default()});

                    // Two squares ahead pawn move
                    if source_square >= 8 && source_square <= 15 && !get_bit(board.occupancies[2], (target_square + 8) as usize) {
                        move_list.push(Move {source_square: source_square as u8, target_square: (target_square+8) as u8, piece: Piece::p, double_push: true, ..Default::default()});
                    }
                }
            }

            // Init pawn attacks bitboard
            attacks = PAWN_ATTACKS[board.side][source_square] & board.occupancies[0];

            // Generate pawn captures
            while attacks != 0 {
                // Init target square
                let target_square = attacks.trailing_zeros() as usize;

                // Pawn promotion
                if source_square >= 48 && source_square <= 55 {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::q, capture: true, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::r, capture: true, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::n, capture: true, ..Default::default()});
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, promoted_piece: Piece::b, capture: true, ..Default::default()});

                } else {
                    // One square ahead pawn move
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::p, capture: true, ..Default::default()});

                }
                // Pop ls1b of the pawn attacks
                pop_bit(&mut attacks, target_square);
            }

            // Generate en passant captures
            if board.enpassant < 64 {
                // Lookup pawn attacks and bitwise AND with enpassant square (bit)
                let enpassant_attacks = PAWN_ATTACKS[board.side][source_square] & (1u64 << board.enpassant);

                // Make sure enpassant capture available
                if enpassant_attacks != 0 {
                    // Init enpassant capture target square
                    let target_enpassant = enpassant_attacks.trailing_zeros() as usize;
                    move_list.push(Move {source_square: source_square as u8, target_square: target_enpassant as u8, piece: Piece::p, capture: true, enpassant: true, ..Default::default()});

                }
            }

            // Pop ls1b from piece bitboard copy
            pop_bit(&mut bitboard, source_square as usize);
        }

        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::k as usize].clone();
        while bitboard != 0 {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            let mut attacks = KING_ATTACKS[source_square] & !board.occupancies[1];
            while attacks != 0 {
                let target_square = attacks.trailing_zeros() as usize;
                    if get_bit(board.occupancies[0], target_square){
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::k, capture: true, ..Default::default()});
                    } else {
                        move_list.push(Move { source_square: source_square as u8, target_square: target_square as u8, piece: Piece::k, ..Default::default() });
                    }
                pop_bit(&mut attacks, target_square);
            }
            pop_bit(&mut bitboard, source_square);
        }
        
        
        // King side castling is available
        if board.castle & 4 != 0 {

            // Make sure squares between king and king's rook are empty
            if !get_bit(board.occupancies[2], 5 ) && !get_bit(board.occupancies[2], 6) {
                // Make sure king and the f1 squares are not under attack
                if !is_square_attacked(4, board) && !is_square_attacked(5, board) {
                    move_list.push(Move {source_square: 4, target_square: 6, piece: Piece::k, castling: true, ..Default::default()});
                }
            }
        }

        // Queen side castling is available1
        if board.castle & 8 != 0 {
            // Make sure squares between king and queen's rook are empty
            if !get_bit(board.occupancies[2], 3) && !get_bit(board.occupancies[2], 2) && !get_bit(board.occupancies[2], 1) {
                // Make sure king and the d1 squares are not under attack
                if !is_square_attacked(4, board) && !is_square_attacked(3, board) {
                    move_list.push(Move {source_square: 4, target_square: 2, piece: Piece::k, castling: true, ..Default::default()});
                }
            }
        }

        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::n as usize].clone();
        while bitboard != 0
        {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            let mut attacks = KNIGHT_ATTACKS[source_square] & !board.occupancies[1];

            while attacks != 0 {
                let target_square = attacks.trailing_zeros() as usize;
                // One square ahead n move
                if get_bit(board.occupancies[0], target_square){
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::n, capture: true, ..Default::default()});
                } else {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::n, ..Default::default()});
                }

                pop_bit(&mut attacks, target_square);
            }
            pop_bit(&mut bitboard, source_square);
        }

        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::b as usize].clone();
        while bitboard != 0
        {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            let mut attacks = get_bishop_attacks(source_square,board.occupancies[2]) & !board.occupancies[1];

            while attacks != 0 {
                let target_square = attacks.trailing_zeros() as usize;
                if get_bit(board.occupancies[0], target_square){
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::b, capture: true, ..Default::default()});
                } else {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::b, ..Default::default()});
                }
                pop_bit(&mut attacks, target_square);
            }
            pop_bit(&mut bitboard, source_square);
        }

        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::r as usize].clone();
        while bitboard != 0
        {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            let mut attacks = get_rook_attacks(source_square,board.occupancies[2]) & !board.occupancies[1];
            while attacks != 0 {
                let target_square = attacks.trailing_zeros() as usize;
                // One square ahead pawn move
                if get_bit(board.occupancies[0], target_square){
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::r, capture: true, ..Default::default()});
                } else {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::r, ..Default::default()});
                }
                pop_bit(&mut attacks, target_square);
            }
            pop_bit(&mut bitboard, source_square);
        }

        // Init piece bitboard copy
        bitboard = board.bitboards[Piece::q as usize].clone();
        while bitboard != 0
        {
            // Init source square
            let source_square = bitboard.trailing_zeros() as usize;

            let mut attacks = get_queen_attacks(source_square,board.occupancies[2]) & !board.occupancies[1];

            while attacks != 0 {
                let target_square = attacks.trailing_zeros() as usize;
                // One square ahead pawn move
                if get_bit(board.occupancies[0], target_square){
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::q, capture: true, ..Default::default()});
                } else {
                    move_list.push(Move {source_square: source_square as u8, target_square: target_square as u8, piece: Piece::q, ..Default::default()});
                }
                pop_bit(&mut attacks, target_square);
            }
            pop_bit(&mut bitboard, source_square);
        }
    }
    move_list
}

pub fn make_move(board: &BoardPosition, moveToMake: &Move) -> Option<BoardPosition> {
    let mut newPosition = BoardPosition {
        bitboards: board.bitboards.clone(),
        occupancies: board.occupancies.clone(),
        side: (board.side + 1) % 2,
        enpassant: board.enpassant,
        castle: board.castle,
    };

    let piece = pieceTousize(&moveToMake.piece);

    //move
    pop_bit(&mut newPosition.bitboards[piece], moveToMake.source_square as usize);
    set_bit(&mut newPosition.bitboards[piece], moveToMake.target_square as usize);


    //capture
    if moveToMake.capture == true {
        for i in 0..6{
            pop_bit(&mut newPosition.bitboards[newPosition.side * 6 + i], moveToMake.target_square as usize);
        }
        pop_bit(&mut newPosition.occupancies[newPosition.side], moveToMake.target_square as usize);
    }

    if moveToMake.promoted_piece != Piece::P {
        pop_bit(&mut newPosition.bitboards[piece], moveToMake.target_square as usize);
        set_bit(&mut newPosition.bitboards[pieceTousize(&moveToMake.promoted_piece)], moveToMake.target_square as usize)
    }

    if moveToMake.enpassant {
        if piece < 6 {
            pop_bit(&mut newPosition.bitboards[6], (moveToMake.target_square + 8) as usize)
        }
        else {
            pop_bit(&mut newPosition.bitboards[0], (moveToMake.target_square - 8) as usize)
        }
    }

    newPosition.enpassant = 64;

    if moveToMake.double_push {
        if piece < 6 {
            newPosition.enpassant = (moveToMake.target_square + 8) as usize;
        }
        else {
            newPosition.enpassant = (moveToMake.target_square - 8) as usize;
        }
    }

    if moveToMake.castling {
        match moveToMake.target_square {
            58 => {
                pop_bit(&mut newPosition.bitboards[pieceTousize(&R)], coordinates_to_squares("a1") as usize);
                set_bit(&mut newPosition.bitboards[pieceTousize(&R)], coordinates_to_squares("d1") as usize);
            },
            62 => {
                pop_bit(&mut newPosition.bitboards[pieceTousize(&R)], coordinates_to_squares("h1") as usize);
                set_bit(&mut newPosition.bitboards[pieceTousize(&R)], coordinates_to_squares("f1") as usize);
            },
            2 => {
                pop_bit(&mut newPosition.bitboards[pieceTousize(&r)], coordinates_to_squares("a8") as usize);
                set_bit(&mut newPosition.bitboards[pieceTousize(&r)], coordinates_to_squares("d8") as usize);
            },
            6 => {
                pop_bit(&mut newPosition.bitboards[pieceTousize(&r)], coordinates_to_squares("h8") as usize);
                set_bit(&mut newPosition.bitboards[pieceTousize(&r)], coordinates_to_squares("f8") as usize);
            },
            _ => {}
        }
    }

    newPosition.castle = newPosition.castle & CASTLING_RIGHTS[moveToMake.source_square as usize] as usize;
    newPosition.castle = newPosition.castle & CASTLING_RIGHTS[moveToMake.target_square as usize] as usize;

    newPosition.occupancies[0] = newPosition.bitboards[0 .. 6].iter().fold(0, |acc, &b| acc | b);
    newPosition.occupancies[1] = newPosition.bitboards[6 .. 12].iter().fold(0, |acc, &b| acc | b);
    newPosition.occupancies[2] = newPosition.occupancies[0 .. 2].iter().fold(0, |acc, &b| acc | b);
    
    let mut kingSq: usize = 65;

    if newPosition.side != 0 {
        kingSq = newPosition.bitboards[5].trailing_zeros() as usize;
    }
    else {
        kingSq = newPosition.bitboards[11].trailing_zeros() as usize;
    }
    newPosition.side = board.side;
    
    if is_square_attacked(kingSq, &newPosition) {
        return None;
    }  
    
    newPosition.side = board.side.bitxor(1);

    Some(newPosition)

}

pub const CASTLING_RIGHTS: [u8; 64] = [
7, 15, 15, 15,  3, 15, 15, 11,
15, 15, 15, 15, 15, 15, 15, 15,
15, 15, 15, 15, 15, 15, 15, 15,
15, 15, 15, 15, 15, 15, 15, 15,
15, 15, 15, 15, 15, 15, 15, 15,
15, 15, 15, 15, 15, 15, 15, 15,
15, 15, 15, 15, 15, 15, 15, 15,
13, 15, 15, 15, 12, 15, 15, 14];



#[cfg(test)]
mod tests {
    use std::thread;
    use crate::moveGen::{is_square_attacked, make_move, run_through_attacks};
    use crate::shared::{coordinates_to_squares, parse_fen, Move, Piece};

    #[test]
    fn test_attacked_squares_kiwipete() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq -"); //Rook on e3
            assert_eq!(run_through_attacks(&boardPos), 18437032593966828032);
        }).unwrap();
        handler.join().unwrap();
    }
    
    #[test]
    fn test_rook_attacks_true() {

        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
            assert_eq!(is_square_attacked(coordinates_to_squares("d3"), &boardPos), true);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_rook_attacks_false() {

        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("8/8/8/8/8/4R3/8/8 b - - 0 1"); //Rook on e3
            assert_eq!(is_square_attacked(coordinates_to_squares("b1"), &boardPos), false);
        }).unwrap();
        handler.join().unwrap();
    }

    fn test_double_push() {

        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("rnbqkbnr/1ppppppp/p7/P7/8/8/1PPPPPPP/RNBQKBNR b KQkq - 0 2"); //Rook on e3
            let mv = Move {source_square: coordinates_to_squares("b7") as u8, target_square: coordinates_to_squares("b5") as u8, piece: Piece::P, double_push: true, ..Default::default()};
            let newBoard = make_move(&boardPos, &mv).unwrap();
            assert_eq!(newBoard.enpassant, coordinates_to_squares("b6"));
        }).unwrap();
        handler.join().unwrap();
    }
}