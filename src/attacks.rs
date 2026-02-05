use std::sync::{Once};
use lazy_static::lazy_static;

use crate::shared::set_bit;
use crate::shared::pop_bit;
/**********************************\
 ==================================

              Attacks

 ==================================
\**********************************/

const NOT_A_FILE: u64 = 18374403900871474942;
const NOT_H_FILE: u64 = 9187201950435737471;
const NOT_HG_FILE: u64 = 4557430888798830399;
const NOT_AB_FILE: u64 = 18229723555195321596;

// PAWN_ATTACKS[side][square] - 0 = białe, 1 = czarne
lazy_static! {
    pub static ref PAWN_ATTACKS: [[u64; 64]; 2] = {
        let mut pawn_attacks = [[0; 64]; 2];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                pawn_attacks[0][square] = mask_pawn_attacks(0,square);
                pawn_attacks[1][square] = mask_pawn_attacks(1,square);
            }
        });
            pawn_attacks
    };
}

lazy_static! {
    pub static ref KNIGHT_ATTACKS: [u64; 64] = {
        let mut knight_attacks = [0; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                knight_attacks[square] = mask_knight_attacks(square);
            }
        });
        knight_attacks
    };
}

lazy_static! {
    pub static ref KING_ATTACKS: [u64; 64] = {
        let mut king_attacks = [0; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                king_attacks[square] = mask_king_attacks(square);
            }
        });
        king_attacks
    };
}

fn mask_pawn_attacks(side: usize, square:usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    set_bit(&mut bitboard, square);

    if side == 0 {
        if (bitboard >> 7) & NOT_A_FILE != 0 {
            attacks |= bitboard >> 7;
        }
        if (bitboard >> 9) & NOT_H_FILE != 0 {
            attacks |= bitboard >> 9;
        }
    } else {
        if (bitboard << 7) & NOT_H_FILE != 0 {
            attacks |= bitboard << 7;
        }
        if (bitboard << 9) & NOT_A_FILE != 0 {
            attacks |= bitboard << 9;
        }
    }

    attacks
}

fn mask_knight_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    set_bit(&mut bitboard, square);

    if (bitboard >> 17) & NOT_H_FILE != 0 {
        attacks |= bitboard >> 17;
    }
    if (bitboard >> 15) & NOT_A_FILE != 0 {
        attacks |= bitboard >> 15;
    }
    if (bitboard >> 10) & NOT_HG_FILE != 0 {
        attacks |= bitboard >> 10;
    }
    if (bitboard >> 6) & NOT_AB_FILE != 0 {
        attacks |= bitboard >> 6;
    }
    if (bitboard << 17) & NOT_A_FILE != 0 {
        attacks |= bitboard << 17;
    }
    if (bitboard << 15) & NOT_H_FILE != 0 {
        attacks |= bitboard << 15;
    }
    if (bitboard << 10) & NOT_AB_FILE != 0 {
        attacks |= bitboard << 10;
    }
    if (bitboard << 6) & NOT_HG_FILE != 0 {
        attacks |= bitboard << 6;
    }

    attacks
}

fn mask_king_attacks(square: usize) -> u64 {
    let mut attacks: u64 = 0;
    let mut bitboard: u64 = 0;

    set_bit(&mut bitboard, square);

    if bitboard >> 8 != 0 {
        attacks |= bitboard >> 8;
    }
    if (bitboard >> 9) & NOT_H_FILE != 0 {
        attacks |= bitboard >> 9;
    }
    if (bitboard >> 7) & NOT_A_FILE != 0 {
        attacks |= bitboard >> 7;
    }
    if (bitboard >> 1) & NOT_H_FILE != 0 {
        attacks |= bitboard >> 1;
    }
    if bitboard << 8 != 0 {
        attacks |= bitboard << 8;
    }
    if (bitboard << 9) & NOT_A_FILE != 0 {
        attacks |= bitboard << 9;
    }
    if (bitboard << 7) & NOT_H_FILE != 0 {
        attacks |= bitboard << 7;
    }
    if (bitboard << 1) & NOT_A_FILE != 0 {
        attacks |= bitboard << 1;
    }

    attacks
}

fn mask_bishop_attacks(square: usize) -> u64 {
    // result attacks bitboard
    let mut attacks: u64 = 0;

    // init ranks & files
    //int r, f;

    let tr : i32 = square as i32 / 8;
    let tf : i32 = square as i32 % 8;

    let mut r : i32 = tr  + 1;
    let mut f : i32 = tf  + 1;

    while r <= 6 && f <= 6
    {
        let bitboard : u64 = 1;
        attacks |= bitboard << (r * 8 + f) as u64;
        r += 1;
        f += 1;
    }

    r = tr - 1;
    f = tf + 1;

    while r >= 1 && f <= 6
    {
        let bitboard : u64 = 1;
        attacks |= bitboard << ((r * 8 + f ) as u64);
        r -= 1;
        f += 1;
    }

    r = tr + 1;
    f = tf - 1;

    while r <= 6 && f >= 1
    {
        let bitboard : u64 = 1;
        attacks |= bitboard << ((r * 8 + f) as u64);
        r += 1;
        f -= 1;
    }

    r = tr - 1;
    f = tf - 1;

    while r >= 1 && f >= 1
    {
        let bitboard : u64 = 1;
        attacks |= bitboard << ((r * 8 + f) as u64);
        r -= 1;
        f -= 1;
    }

    attacks
}

fn mask_rook_attacks(square: usize) -> u64 {
    // result attacks bitboard
    let mut attacks: u64 = 0;

    // init ranks & files
    let tr: i32 = square as i32 / 8;
    let tf: i32 = square as i32 % 8;

    // mask relevant rook occupancy bits
    for r in (tr + 1)..=6 {
        attacks |= 1u64 << (r * 8 + tf);
    }
    for r in (1..tr).rev() {
        attacks |= 1u64 << (r * 8 + tf);
    }
    for f in (tf + 1)..=6 {
        attacks |= 1u64 << (tr * 8 + f);
    }
    for f in (1..tf).rev() {
        attacks |= 1u64 << (tr * 8 + f);
    }

    // return attack map
    attacks
}

fn bishop_attacks_on_the_fly(square: usize, block: u64) -> u64 {
// result attacks bitboard
    let mut attacks: u64 = 0;

    // init ranks & files
    let tr: i32 = (square / 8) as i32;
    let tf: i32 = (square % 8) as i32;

    // generate bishop attacks
    let mut r = tr + 1;
    let mut f = tf + 1;
    while r <= 7 && f <= 7 {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & block != 0 {
            break;
        }
        r += 1;
        f += 1;
    }

    r = tr - 1;
    f = tf + 1;
    while r >= 0 && f <= 7 {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & block != 0 {
            break;
        }
        r -= 1;
        f += 1;
    }

    r = tr + 1;
    f = tf - 1;
    while r <= 7 && f >= 0 {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & block != 0 {
            break;
        }
        r += 1;
        f -= 1;
    }

    r = tr - 1;
    f = tf - 1;
    while r >= 0 && f >= 0 {
        attacks |= 1u64 << (r * 8 + f);
        if (1u64 << (r * 8 + f)) & block != 0 {
            break;
        }
        r -= 1;
        f -= 1;
    }

    // return attack map
    attacks
}

fn rook_attacks_on_the_fly(square: usize, block: u64) -> u64 {
    // result attacks bitboard
    let mut attacks: u64 = 0;

    // init ranks & files
    let tr: i32 = (square / 8) as i32;
    let tf: i32 = (square % 8) as i32;

    // generate rook attacks
    for r in (tr + 1)..=7 {
        attacks |= 1u64 << (r * 8 + tf);
        if (1u64 << (r * 8 + tf)) & block != 0 {
            break;
        }
    }

    for r in (0..tr).rev() {
        attacks |= 1u64 << (r * 8 + tf);
        if (1u64 << (r * 8 + tf)) & block != 0 {
            break;
        }
    }

    for f in (tf + 1)..=7 {
        attacks |= 1u64 << (tr * 8 + f);
        if (1u64 << (tr * 8 + f)) & block != 0 {
            break;
        }
    }

    for f in (0..tf).rev() {
        attacks |= 1u64 << (tr * 8 + f);
        if (1u64 << (tr * 8 + f)) & block != 0 {
            break;
        }
    }

    // return attack map
    attacks
}

pub fn set_occupancy(index: i32, bits_in_mask: u32, mut attack_mask: u64) -> u64 {
    let mut occupancy: u64 = 0;

    for count in 0..bits_in_mask {
        let square = attack_mask.trailing_zeros() as usize;

        pop_bit(&mut attack_mask, square);

        if index & (1 << count) != 0 {
            occupancy |= 1 << square;
        }
    }

    occupancy
}



/*************************************\

                SLIDERS

\*************************************/
// rook magic numbers
const ROOK_MAGIC_NUMBERS: [u64; 64] = [
0x8a80104000800020,
0x140002000100040,
0x2801880a0017001,
0x100081001000420,
0x200020010080420,
0x3001c0002010008,
0x8480008002000100,
0x2080088004402900,
0x800098204000,
0x2024401000200040,
0x100802000801000,
0x120800800801000,
0x208808088000400,
0x2802200800400,
0x2200800100020080,
0x801000060821100,
0x80044006422000,
0x100808020004000,
0x12108a0010204200,
0x140848010000802,
0x481828014002800,
0x8094004002004100,
0x4010040010010802,
0x20008806104,
0x100400080208000,
0x2040002120081000,
0x21200680100081,
0x20100080080080,
0x2000a00200410,
0x20080800400,
0x80088400100102,
0x80004600042881,
0x4040008040800020,
0x440003000200801,
0x4200011004500,
0x188020010100100,
0x14800401802800,
0x2080040080800200,
0x124080204001001,
0x200046502000484,
0x480400080088020,
0x1000422010034000,
0x30200100110040,
0x100021010009,
0x2002080100110004,
0x202008004008002,
0x20020004010100,
0x2048440040820001,
0x101002200408200,
0x40802000401080,
0x4008142004410100,
0x2060820c0120200,
0x1001004080100,
0x20c020080040080,
0x2935610830022400,
0x44440041009200,
0x280001040802101,
0x2100190040002085,
0x80c0084100102001,
0x4024081001000421,
0x20030a0244872,
0x12001008414402,
0x2006104900a0804,
0x1004081002402
];

// bishop magic numbers
const BISHOP_MAGIC_NUMBERS: [u64; 64] = [
0x40040844404084,
0x2004208a004208,
0x10190041080202,
0x108060845042010,
0x581104180800210,
0x2112080446200010,
0x1080820820060210,
0x3c0808410220200,
0x4050404440404,
0x21001420088,
0x24d0080801082102,
0x1020a0a020400,
0x40308200402,
0x4011002100800,
0x401484104104005,
0x801010402020200,
0x400210c3880100,
0x404022024108200,
0x810018200204102,
0x4002801a02003,
0x85040820080400,
0x810102c808880400,
0xe900410884800,
0x8002020480840102,
0x220200865090201,
0x2010100a02021202,
0x152048408022401,
0x20080002081110,
0x4001001021004000,
0x800040400a011002,
0xe4004081011002,
0x1c004001012080,
0x8004200962a00220,
0x8422100208500202,
0x2000402200300c08,
0x8646020080080080,
0x80020a0200100808,
0x2010004880111000,
0x623000a080011400,
0x42008c0340209202,
0x209188240001000,
0x400408a884001800,
0x110400a6080400,
0x1840060a44020800,
0x90080104000041,
0x201011000808101,
0x1a2208080504f080,
0x8012020600211212,
0x500861011240000,
0x180806108200800,
0x4000020e01040044,
0x300000261044000a,
0x802241102020002,
0x20906061210001,
0x5a84841004010310,
0x4010801011c04,
0xa010109502200,
0x4a02012000,
0x500201010098b028,
0x8040002811040900,
0x28000010020204,
0x6000020202d0240,
0x8918844842082200,
0x4010011029020020
];

lazy_static! {
    pub static ref BISHOP_ATTACKS: [[u64; 512]; 64] = {
        let mut bishop_attacks = [[0; 512]; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                let attack_mask = BISHOP_MASKS[square];

                // Initialize relevant occupancy bit count
                let relevant_bits_count = attack_mask.count_ones();

                // Initialize occupancy indices
                let occupancy_indices = 1 << relevant_bits_count;

                // Loop over occupancy indices
                for index in 0..occupancy_indices {
                    // Initialize current occupancy variation
                    let occupancy = set_occupancy(index, relevant_bits_count, attack_mask);

                        // Initialize magic index
                    let magic_index = ((occupancy.wrapping_mul( BISHOP_MAGIC_NUMBERS[square])) >> (64 - BISHOP_RELEVANT_BITS[square])) as usize;

                    // Initialize bishop attacks
                    bishop_attacks[square][magic_index] = bishop_attacks_on_the_fly(square, occupancy);
                }
            }
        });
            bishop_attacks
    };

}

lazy_static! {
    pub static ref BISHOP_MASKS: [u64; 64] = {
        let mut bishop_masks = [0; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                bishop_masks[square] = mask_bishop_attacks(square);


            }
        });
            bishop_masks
    };
}

lazy_static! {
    pub static ref ROOK_ATTACKS: [[u64; 4096]; 64] = {
        let mut rook_attacks = [[0; 4096]; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                let attack_mask = ROOK_MASKS[square];

                // Initialize relevant occupancy bit count
                let relevant_bits_count = attack_mask.count_ones();

                // Initialize occupancy indices
                let occupancy_indices = 1 << relevant_bits_count;

                // Loop over occupancy indices
                for index in 0..occupancy_indices {
                    // Initialize current occupancy variation
                    let occupancy = set_occupancy(index, relevant_bits_count, attack_mask);

                        // Initialize magic index
                    let magic_index = ((occupancy.wrapping_mul( ROOK_MAGIC_NUMBERS[square])) >> (64 - ROOK_RELEVANT_BITS[square])) as usize;

                    // Initialize bishop attacks
                    rook_attacks[square][magic_index] = rook_attacks_on_the_fly(square, occupancy);
                }
            }
        });
            rook_attacks
    };

}

lazy_static! {
    pub static ref ROOK_MASKS: [u64; 64] = {
        let mut rook_masks = [0; 64];
        let once = Once::new();
        once.call_once(|| {
            for square  in 0..64 {
                rook_masks[square] = mask_rook_attacks(square);
            }
        });
            rook_masks
    };
}


// bishop relevant occupancy bit count for every square on board
const BISHOP_RELEVANT_BITS: [usize;64] = [
6, 5, 5, 5, 5, 5, 5, 6,
5, 5, 5, 5, 5, 5, 5, 5,
5, 5, 7, 7, 7, 7, 5, 5,
5, 5, 7, 9, 9, 7, 5, 5,
5, 5, 7, 9, 9, 7, 5, 5,
5, 5, 7, 7, 7, 7, 5, 5,
5, 5, 5, 5, 5, 5, 5, 5,
6, 5, 5, 5, 5, 5, 5, 6
];

// rook relevant occupancy bit count for every square on board
const ROOK_RELEVANT_BITS: [usize;64] = [
12, 11, 11, 11, 11, 11, 11, 12,
11, 10, 10, 10, 10, 10, 10, 11,
11, 10, 10, 10, 10, 10, 10, 11,
11, 10, 10, 10, 10, 10, 10, 11,
11, 10, 10, 10, 10, 10, 10, 11,
11, 10, 10, 10, 10, 10, 10, 11,
11, 10, 10, 10, 10, 10, 10, 11,
12, 11, 11, 11, 11, 11, 11, 12
];
/*
pub fn init_sliders_attacks(bishop: bool) {
    let mut rook_masks = ROOK_MASKS.lock().unwrap();
    let mut bishop_masks = BISHOP_MASKS.lock().unwrap();
    let mut rook_attacks = ROOK_ATTACKS.lock().unwrap();
    let mut bishop_attacks = BISHOP_ATTACKS.lock().unwrap();
    // Loop over 64 board squares
    for square in 0..64 {
        // Initialize bishop & rook masks
        bishop_masks[square] = mask_bishop_attacks(square);
        rook_masks[square] = mask_rook_attacks(square);

        // Initialize attack mask
        let attack_mask = if bishop {
            bishop_masks[square]
        } else {
            rook_masks[square]
        };

        // Initialize relevant occupancy bit count
        let relevant_bits_count = attack_mask.count_ones();

        // Initialize occupancy indices
        let occupancy_indices = 1 << relevant_bits_count;

        // Loop over occupancy indices
        for index in 0..occupancy_indices {
            // Bishop
            if bishop {
                // Initialize current occupancy variation
                let occupancy = set_occupancy(index, relevant_bits_count, attack_mask);

                // Initialize magic index
            let magic_index = ((occupancy.wrapping_mul( BISHOP_MAGIC_NUMBERS[square])) >> (64 - BISHOP_RELEVANT_BITS[square])) as usize;

                // Initialize bishop attacks
                bishop_attacks[square][magic_index] = bishop_attacks_on_the_fly(square, occupancy);
            }
            // Rook
            else {
                // Initialize current occupancy variation
                let occupancy = set_occupancy(index, relevant_bits_count, attack_mask);

                // Initialize magic index
                let magic_index = ((occupancy.wrapping_mul( ROOK_MAGIC_NUMBERS[square])) >> (64 - ROOK_RELEVANT_BITS[square])) as usize;

                // Initialize rook attacks
                rook_attacks[square][magic_index] = rook_attacks_on_the_fly(square, occupancy);
            }
        }
    }
}
*/
// Get bishop attacks
pub fn get_bishop_attacks(square: usize, occupancy: u64) -> u64 {
    // Get bishop attacks assuming current board occupancy

    let mut occupancy = occupancy & BISHOP_MASKS[square];
    occupancy = occupancy.wrapping_mul(BISHOP_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - BISHOP_RELEVANT_BITS[square];

    // Return bishop attacks
    BISHOP_ATTACKS[square][occupancy as usize]
}

// Get rook attacks
pub fn get_rook_attacks(square: usize, occupancy: u64) -> u64 {
    // Get rook attacks assuming current board occupancy
    let mut occupancy = occupancy & ROOK_MASKS[square];
    occupancy = occupancy.wrapping_mul(ROOK_MAGIC_NUMBERS[square]);
    occupancy >>= 64 - ROOK_RELEVANT_BITS[square];

    // Return rook attacks
    ROOK_ATTACKS[square][occupancy as usize]
}

pub fn get_queen_attacks(square: usize, occupancy: u64) -> u64 {
    get_rook_attacks(square,occupancy) | get_bishop_attacks(square,occupancy)
}