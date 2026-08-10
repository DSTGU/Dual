use std::sync::{Once};
use lazy_static::lazy_static;

use crate::primitives::{board::BoardPosition, shared::{Move, MoveCode, Piece, pop_bit, set_bit}};
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

// -----------------------------------------------------------------------------
// Slider attacks: "fancy" magic bitboards with per-square table sizes.
//
// Instead of one fixed-size table per square (`[[u64; 512]; 64]` / `[[u64; 4096]; 64]`,
// ~2.25 MiB, mostly wasted space) all attacks live in a single dense, flat table
// per piece. Square `sq` occupies `1 << relevant_bits[sq]` entries starting at
// `base[sq]`, so the whole set is only ~840 KiB and stays cache-friendly.
//
// Index computation:  idx = (occ & mask) * magic >> (64 - bits)
// The chosen magics are collision-free on [0, 2^bits), so the map is a bijection
// and the per-square slices can be packed back-to-back without gaps.
//
// A BMI2 `pext`-based fast path is also available (see USE_PEXT below). It is
// disabled by default: on CPUs where `pext` is microcoded it is much slower than 
// the magic multiply-shift, and LLVM silently
// software-emulates `pext` when the codegen target lacks BMI2 (e.g. a plain
// `cargo build --release`), which is far worse again. Enable it only on CPUs
// with native (fast) PEXT and always build with `-C target-cpu=native`.
// -----------------------------------------------------------------------------

/// Master switch: use the BMI2 `pext` fast path instead of magic multiply-shift.
/// Keep `false` unless you build with BMI2 codegen (e.g. `cargo rustc --release
/// -- -C target-cpu=native`, as the Makefile does) and you have verified that
/// `pext` is fast on your CPU.
const USE_PEXT: bool = false;

// bishop relevant occupancy bit count for every square on board
const BISHOP_RELEVANT_BITS: [usize; 64] = [
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
const ROOK_RELEVANT_BITS: [usize; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11,
    12, 11, 11, 11, 11, 11, 11, 12
];

// Per-square base offsets into the flat attack tables (compile-time prefix sums).
const fn slider_bases(bits: &[usize; 64]) -> [usize; 64] {
    let mut bases = [0usize; 64];
    let mut i = 1usize;
    while i < 64 {
        bases[i] = bases[i - 1] + (1usize << bits[i - 1] as u32);
        i += 1;
    }
    bases
}

const BISHOP_BASE: [usize; 64] = slider_bases(&BISHOP_RELEVANT_BITS);
const ROOK_BASE: [usize; 64] = slider_bases(&ROOK_RELEVANT_BITS);

/// Dense table index via the classic magic multiply-shift.
#[inline(always)]
fn magic_index(magic: u64, shift: usize, mask: u64, occupancy: u64) -> usize {
    let occ = (occupancy & mask).wrapping_mul(magic);
    (occ >> shift) as usize
}

/// Dense table index via the BMI2 `pext` instruction. `pext` applies the mask
/// itself, so no separate `& mask` is needed. Only compiled when the codegen
/// target actually has BMI2; otherwise `use_pext()` cannot work.
#[cfg(target_feature = "bmi2")]
#[inline(always)]
fn pext_index(mask: u64, occupancy: u64) -> usize {
    unsafe { std::arch::x86_64::_pext_u64(occupancy, mask) as usize }
}

/// Guard for misconfigured builds: if USE_PEXT is enabled but the crate was not
/// compiled with BMI2 codegen, fail loudly instead of silently letting LLVM
/// software-emulate the instruction (correct, but far slower than magic).
#[cfg(not(target_feature = "bmi2"))]
fn pext_index(_mask: u64, _occupancy: u64) -> usize {
    unreachable!(
        "USE_PEXT requires building with BMI2 codegen, e.g. \
         `cargo rustc --release -- -C target-cpu=native`"
    )
}

/// Fill a dense slider attack table.
fn build_slider_table(
    masks: &[u64; 64],
    bases: &[usize; 64],
    index_of: impl Fn(usize, u64) -> usize,
    attacks_of: impl Fn(usize, u64) -> u64,
) -> Box<[u64]> {
    let total: usize = masks.iter().map(|m| 1usize << m.count_ones()).sum();
    let mut table = vec![0u64; total];
    for square in 0..64 {
        let mask = masks[square];
        let bits = mask.count_ones();
        let base = bases[square];
        for index in 0..(1usize << bits) {
            let occupancy = set_occupancy(index as i32, bits, mask);
            table[base + index_of(square, occupancy)] = attacks_of(square, occupancy);
        }
    }
    table.into_boxed_slice()
}

lazy_static! {
    /// Relevant occupancy bits for bishop attacks on each square (edges excluded).
    pub static ref BISHOP_MASKS: [u64; 64] = {
        let mut bishop_masks = [0; 64];
        for square in 0..64 {
            bishop_masks[square] = mask_bishop_attacks(square);
        }
        bishop_masks
    };
}

lazy_static! {
    /// Relevant occupancy bits for rook attacks on each square (edges excluded).
    pub static ref ROOK_MASKS: [u64; 64] = {
        let mut rook_masks = [0; 64];
        for square in 0..64 {
            rook_masks[square] = mask_rook_attacks(square);
        }
        rook_masks
    };
}

lazy_static! {
    /// Flat bishop attack table, magic order (per-square base offsets).
    /// Empty while USE_PEXT is enabled.
    pub static ref BISHOP_ATTACKS: Box<[u64]> = {
        if USE_PEXT {
            Vec::new().into_boxed_slice()
        } else {
            build_slider_table(
                &BISHOP_MASKS,
                &BISHOP_BASE,
                |sq, occ| magic_index(
                    BISHOP_MAGIC_NUMBERS[sq],
                    64 - BISHOP_RELEVANT_BITS[sq],
                    BISHOP_MASKS[sq],
                    occ,
                ),
                |sq, occ| bishop_attacks_on_the_fly(sq, occ),
            )
        }
    };
}

lazy_static! {
    /// Flat rook attack table, magic order (per-square base offsets).
    /// Empty while USE_PEXT is enabled.
    pub static ref ROOK_ATTACKS: Box<[u64]> = {
        if USE_PEXT {
            Vec::new().into_boxed_slice()
        } else {
            build_slider_table(
                &ROOK_MASKS,
                &ROOK_BASE,
                |sq, occ| magic_index(
                    ROOK_MAGIC_NUMBERS[sq],
                    64 - ROOK_RELEVANT_BITS[sq],
                    ROOK_MASKS[sq],
                    occ,
                ),
                |sq, occ| rook_attacks_on_the_fly(sq, occ),
            )
        }
    };
}

lazy_static! {
    /// Flat bishop attack table, PEXT order (used when USE_PEXT is enabled).
    pub static ref BISHOP_ATTACKS_PEXT: Box<[u64]> = {
        if USE_PEXT {
            build_slider_table(
                &BISHOP_MASKS,
                &BISHOP_BASE,
                |sq, occ| pext_index(BISHOP_MASKS[sq], occ),
                |sq, occ| bishop_attacks_on_the_fly(sq, occ),
            )
        } else {
            Vec::new().into_boxed_slice()
        }
    };
}

lazy_static! {
    /// Flat rook attack table, PEXT order (used when USE_PEXT is enabled).
    pub static ref ROOK_ATTACKS_PEXT: Box<[u64]> = {
        if USE_PEXT {
            build_slider_table(
                &ROOK_MASKS,
                &ROOK_BASE,
                |sq, occ| pext_index(ROOK_MASKS[sq], occ),
                |sq, occ| rook_attacks_on_the_fly(sq, occ),
            )
        } else {
            Vec::new().into_boxed_slice()
        }
    };
}


// Get bishop attacks
#[inline(always)]
pub fn get_bishop_attacks(square: usize, occupancy: u64) -> u64 {
    if USE_PEXT {
        let mask = BISHOP_MASKS[square];
        BISHOP_ATTACKS_PEXT[BISHOP_BASE[square] + pext_index(mask, occupancy)]
    } else {
        let idx = magic_index(
            BISHOP_MAGIC_NUMBERS[square],
            64 - BISHOP_RELEVANT_BITS[square],
            BISHOP_MASKS[square],
            occupancy,
        );
        BISHOP_ATTACKS[BISHOP_BASE[square] + idx]
    }
}

// Get rook attacks
#[inline(always)]
pub fn get_rook_attacks(square: usize, occupancy: u64) -> u64 {
    if USE_PEXT {
        let mask = ROOK_MASKS[square];
        ROOK_ATTACKS_PEXT[ROOK_BASE[square] + pext_index(mask, occupancy)]
    } else {
        let idx = magic_index(
            ROOK_MAGIC_NUMBERS[square],
            64 - ROOK_RELEVANT_BITS[square],
            ROOK_MASKS[square],
            occupancy,
        );
        ROOK_ATTACKS[ROOK_BASE[square] + idx]
    }
}

pub fn get_queen_attacks(square: usize, occupancy: u64) -> u64 {
    get_rook_attacks(square,occupancy) | get_bishop_attacks(square,occupancy)
}


// Function for taking from which squares can a given piece attack a given square for that bp
// That's why pawns are inverted
pub fn get_piece_attacks(board_position: &BoardPosition, square: u8, piece: Piece) -> u64 {

    match piece {
        Piece::P => PAWN_ATTACKS[1][square as usize],
        Piece::p => PAWN_ATTACKS[0][square as usize],
        Piece::K | Piece::k => KING_ATTACKS[square as usize],
        Piece::N | Piece::n => KNIGHT_ATTACKS[square as usize],
        Piece::B | Piece::b => get_bishop_attacks(square as usize, board_position.occupancies[2]),
        Piece::R | Piece::r => get_rook_attacks(square as usize, board_position.occupancies[2]),
        Piece::Q | Piece::q => get_queen_attacks(square as usize,board_position.occupancies[2]),

        Piece::NONE => 0
    }
}

pub fn get_least_valuable_attacker(board_position: &BoardPosition, square: u8) -> ( Move, Option<BoardPosition> ) {
    
    let side = board_position.side;

    for piece_idx in 0 + 6*side as usize..6 + 6*side as usize {
        let attacker = Piece::new(piece_idx);

        let attacks = get_piece_attacks(board_position, square, attacker);
        let mut attacking_pieces = attacks & board_position.bitboards[piece_idx];

        for _ in 0..attacking_pieces.count_ones() {
            let source = attacking_pieces.trailing_zeros() as u8;
            pop_bit(&mut attacking_pieces, source as usize);

            // let mv = if piece_idx % 6 == 0 && square < 8 || square >= 56 
            //     { Move::create(source, square, MoveCode::QueenPromotionCapture)} else 
            //     { Move::create(source, square, MoveCode::Capture) };

            let mv = { Move::create(source, square, MoveCode::Capture) };
            let board_position = board_position.make_move(mv);

            if board_position.is_some() {
                return (mv, board_position);
            }
        }

    }

    (Move::create_null(), None)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Cross-check both table orderings (PEXT, when enabled, and magic) against
    /// the reference on-the-fly generators, for every square and every relevant
    /// occupancy.
    #[test]
    fn slider_tables_match_on_the_fly() {
        // Force the tables to be built.
        let _ = &*BISHOP_ATTACKS;
        let _ = &*ROOK_ATTACKS;
        let _ = &*BISHOP_ATTACKS_PEXT;
        let _ = &*ROOK_ATTACKS_PEXT;

        for square in 0..64 {
            let bmask = BISHOP_MASKS[square];
            let bbits = bmask.count_ones();
            let bmagic_idx = |occ: u64| {
                magic_index(
                    BISHOP_MAGIC_NUMBERS[square],
                    64 - BISHOP_RELEVANT_BITS[square],
                    bmask,
                    occ,
                )
            };
            for index in 0..(1usize << bbits) {
                let occ = set_occupancy(index as i32, bbits, bmask);
                let expected = bishop_attacks_on_the_fly(square, occ);
                if USE_PEXT {
                    assert_eq!(
                        BISHOP_ATTACKS_PEXT[BISHOP_BASE[square] + pext_index(bmask, occ)],
                        expected,
                        "pext bishop index mismatch sq {} occ {}",
                        square,
                        occ
                    );
                } else {
                    assert_eq!(
                        BISHOP_ATTACKS[BISHOP_BASE[square] + bmagic_idx(occ)],
                        expected,
                        "magic bishop index mismatch sq {} occ {}",
                        square,
                        occ
                    );
                }
            }

            let rmask = ROOK_MASKS[square];
            let rbits = rmask.count_ones();
            let rmagic_idx = |occ: u64| {
                magic_index(
                    ROOK_MAGIC_NUMBERS[square],
                    64 - ROOK_RELEVANT_BITS[square],
                    rmask,
                    occ,
                )
            };
            for index in 0..(1usize << rbits) {
                let occ = set_occupancy(index as i32, rbits, rmask);
                let expected = rook_attacks_on_the_fly(square, occ);
                if USE_PEXT {
                    assert_eq!(
                        ROOK_ATTACKS_PEXT[ROOK_BASE[square] + pext_index(rmask, occ)],
                        expected,
                        "pext rook index mismatch sq {} occ {}",
                        square,
                        occ
                    );
                } else {
                    assert_eq!(
                        ROOK_ATTACKS[ROOK_BASE[square] + rmagic_idx(occ)],
                        expected,
                        "magic rook index mismatch sq {} occ {}",
                        square,
                        occ
                    );
                }
            }
        }
    }
}