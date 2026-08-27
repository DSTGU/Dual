//! OpenBench "genfens" interface — Data Generation workloads.
//!
//! OpenBench runs the engine from the command line as:
//!
//! ```text
//! ./engine "genfens N seed S book <None|Books/my_book.epd> <?extra>" "quit"
//! ```
//!
//! * `N`    – number of openings (FENs) to print.
//! * `S`    – unsigned 64-bit seed; the upper 32 bits are the workload id and
//!   the lower 32 bits are the book offset. All 64 bits are faithfully
//!   used to seed the internal RNG.
//! * `book` – optional EPD file whose lines are used as starting points, or
//!   `None` to always start from the start position.
//! * extra  – optional workload-specific arguments (currently ignored).
//!
//! Each generated opening is printed immediately to stdout as
//! `info string genfens <fen>`, so workers never stall the 15-second
//! watchdog. All regular UCI output is suppressed during generation.

use std::io::{self, Write};

use crate::evaluation::nnue::NNUE;
use crate::movegen::move_gen::generate_all_moves;
use crate::primitives::board::BoardPosition;
use crate::primitives::consts::MIN_DEPTH;
use crate::primitives::shared::{Move, Piece, START_POSITION};
use crate::search::single_depth_search;
use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::{Reporting, SearchState};

/// Number of random half-moves played before searching a candidate position.
const GENFENS_PLIES: usize = 8;

/// Accept a position when |score| (centipawns) is within `[MIN_CP, MAX_CP]`,
/// i.e. between 0.5 and 2.0 pawns.
const MIN_CP: i32 = 70;
const MAX_CP: i32 = 200;

/// SplitMix64 — a fast, high quality 64-bit PRNG.
///
/// Seeded with the *entire* 64-bit genfens seed so that both the workload id
/// (upper 32 bits) and the book offset (lower 32 bits) are faithfully used.
struct SplitMix64(u64);

impl SplitMix64 {
    fn new(seed: u64) -> Self {
        SplitMix64(seed)
    }

    #[inline]
    fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform index in `0..range`.
    #[inline]
    fn below(&mut self, range: usize) -> usize {
        (self.next_u64() as usize) % range
    }
}

/// Load openings from an EPD file. Each line's first four fields (piece
/// placement, side to move, castling rights, en passant square) are turned
/// into a FEN; lines without both kings are discarded.
fn load_book(path: &str) -> Vec<String> {
    let content = match std::fs::read_to_string(path) {
        Ok(content) => content,
        Err(err) => {
            eprintln!("genfens: failed to read book '{}': {}", path, err);
            return Vec::new();
        }
    };

    let mut openings = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') || line.starts_with('[') {
            continue;
        }

        // EPD: "<fen fields> ; ops" — take everything before the first ';'.
        let fields: Vec<&str> = line
            .split(';')
            .next()
            .unwrap_or("")
            .split_ascii_whitespace()
            .collect();

        if fields.len() < 4 {
            continue;
        }

        let fen = format!("{} {} {} {} 0 1", fields[0], fields[1], fields[2], fields[3]);

        let board = BoardPosition::new(&fen);
        if board.bitboards[Piece::K as usize] == 0 || board.bitboards[Piece::k as usize] == 0 {
            continue;
        }

        openings.push(fen);
    }

    openings
}

/// Play `plies` random legal half-moves from `start_fen`, keeping the search
/// state's NNUE accumulators and repetition table in sync. Returns `None` if
/// the game ends (checkmate/stalemate) before `plies` moves are played.
fn play_random_plies(
    rng: &mut SplitMix64,
    search_state: &mut SearchState,
    start_fen: &str,
    plies: usize,
) -> Option<BoardPosition> {
    search_state.clear_data();
    search_state.clear_persistent_data();

    let mut board = BoardPosition::new(start_fen);

    for _ in 0..plies {
        let pseudo = generate_all_moves(&board);

        // Filter down to legal moves (pseudo-legal movegen can leave the king
        // en prise when pinned).
        let mut legal: Vec<Move> = Vec::with_capacity(pseudo.len());
        for mv in &pseudo {
            if board.make_move(mv.mv).is_some() {
                legal.push(mv.mv);
            }
        }

        if legal.is_empty() {
            return None;
        }

        let mv = legal[rng.below(legal.len())];
        let new_board = board.make_move(mv).unwrap();

        search_state.prefill_position_info(board.hash);
        board = new_board;
    }

    search_state.network_state.start_board(&board, &NNUE);

    // Match normal play: the search starts at ply 0 while the played plies
    // remain in the repetition table for draw detection.
    search_state.ply = 0;

    Some(board)
}

/// Generate `n` openings, writing them to `out` as `info string genfens <fen>`
/// lines. Returns the number of openings written.
fn generate(n: usize, seed: u64, book: &[String], out: &mut dyn Write) -> io::Result<usize> {
    let mut rng = SplitMix64::new(seed);

    // Datagen positions are fully independent, so the transposition table is
    // disabled and each position searches from a pristine state. This keeps
    // every position's score deterministic and uninfluenced by earlier ones.
    let mut search_state = SearchState::new(&EngineConfig {
        hash: 0,
        soft_nodes: None,
    });
    search_state.reporting = Reporting::Quiet;

    let mut written = 0;
    while written < n {
        // 1. Starting position: random book line, or the start position.
        let start_fen = if book.is_empty() {
            START_POSITION.to_string()
        } else {
            book[rng.below(book.len())].clone()
        };

        // 2. Play 8 random half-moves; retry if the game ended early.
        let Some(board) = play_random_plies(&mut rng, &mut search_state, &start_fen, GENFENS_PLIES)
        else {
            continue;
        };

        // 3. Fixed MIN_DEPTH search, fully quiet.
        search_state.reset_for_new_iteration(MIN_DEPTH);
        let result = single_depth_search(&board, &mut search_state, MIN_DEPTH);

        // 4. Accept only positions whose |score| sits within 0.5-2.0 pawns.
        let score = result.abs();
        if !(MIN_CP..=MAX_CP).contains(&score) {
            continue;
        }

        // 5. Emit immediately (the caller line-buffers) so the worker's
        //    15-second watchdog never fires.
        let fullmove = 1 + GENFENS_PLIES / 2;
        writeln!(out, "info string genfens {}", board.to_fen(fullmove))?;
        written += 1;
    }

    Ok(written)
}

/// Entry point for the `genfens` command line interface.
///
/// `tokens` are the words following "genfens", e.g.
/// `["1000", "seed", "42", "book", "None", "extra...", "quit"]`.
pub fn run_genfens(tokens: Vec<&str>) {
    
    let mut n: usize = 0;
    let mut seed: u64 = 0;
    let mut book_path: Option<String> = None;

    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            "seed" => {
                if i + 1 < tokens.len() {
                    seed = parse_seed(&tokens[i + 1]);
                }
                i += 2;
            }
            "book" => {
                if i + 1 < tokens.len() {
                    book_path = Some(tokens[i + 1].to_owned());
                }
                i += 2;
            }
            other => {
                // The first positional token is the number of openings; any
                // remaining unknown tokens are workload extras (ignored).
                if n == 0 {
                    n = other.parse().unwrap_or(0);
                }
                i += 1;
            }
        }
    }

    if n == 0 {
        eprintln!("genfens: invalid or missing opening count");
        return;
    }

    let book = match book_path {
        Some(path) if path.eq_ignore_ascii_case("none") || path.is_empty() => {
            eprintln!("genfens: no opening book, using the start position");
            Vec::new()
        }
        Some(path) => {
            let openings = load_book(&path);
            if openings.is_empty() {
                eprintln!(
                    "genfens: book '{}' yielded no valid openings, using the start position",
                    path
                );
            } else {
                eprintln!("genfens: loaded {} openings from '{}'", openings.len(), path);
            }
            openings
        }
        None => {
            eprintln!("genfens: no opening book, using the start position");
            Vec::new()
        }
    };

    // Line-buffered stdout: every FEN line is flushed immediately.
    let stdout = io::stdout();
    let mut writer = io::LineWriter::new(stdout.lock());
    match generate(n, seed, &book, &mut writer) {
        Ok(written) => eprintln!("genfens: wrote {} openings", written),
        Err(err) => eprintln!("genfens: I/O error while writing openings: {}", err),
    }
}

/// Parse the seed: a decimal (or `0x`-prefixed hex) unsigned 64-bit value.
fn parse_seed(raw: &str) -> u64 {
    if let Some(hex) = raw.strip_prefix("0x").or_else(|| raw.strip_prefix("0X")) {
        return u64::from_str_radix(hex, 16).unwrap_or(0);
    }
    raw.parse().unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn test_generate_prints_expected_lines() {
            let mut out = Cursor::new(Vec::new());
            let written = generate(5, 0x1234_5678_9ABC_DEF0, &[], &mut out).unwrap();
            assert_eq!(written, 5);

            let text = String::from_utf8(out.into_inner()).unwrap();
            let lines: Vec<&str> = text.lines().collect();
            assert_eq!(lines.len(), 5);

            for line in &lines {
                assert!(line.starts_with("info string genfens "));
                let fen = line.trim_start_matches("info string genfens ").trim();
                let board = BoardPosition::new(fen);
                assert!(board.bitboards[Piece::K as usize] > 0);
                assert!(board.bitboards[Piece::k as usize] > 0);
            }
    }

    #[test]
    fn test_generate_with_book() {
            let book = vec![
                "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1".to_string(),
                "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1".to_string(),
            ];
            let mut out = Cursor::new(Vec::new());
            let written = generate(10, 7, &book, &mut out).unwrap();
            assert_eq!(written, 10);
    }

    #[test]
    fn test_splitmix_below_bounded() {
        let mut rng = SplitMix64::new(u64::MAX);
        for _ in 0..1000 {
            let v = rng.below(10);
            assert!(v < 10);
        }
    }

    #[test]
    fn test_parse_seed_full_64_bit() {
        // Upper 32 bits = workload id, lower 32 = book offset.
        assert_eq!(parse_seed("18446744073709551615"), u64::MAX);
        assert_eq!(parse_seed("0xDEADBEEF"), 0xDEADBEEF);
        assert_eq!(parse_seed("0Xdeadbeef"), 0xDEADBEEF);
        assert_eq!(parse_seed("42"), 42);
    }

    #[test]
    fn test_load_book_epd() {
        let path = std::env::temp_dir().join("dual_test_book.epd");
        std::fs::write(
            &path,
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - bm e2e4; id \"start\";\n\
             r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - bm g1f3;\n\
             garbage line without a fen\n",
        )
        .unwrap();

        let openings = load_book(path.to_str().unwrap());
        std::fs::remove_file(&path).ok();

        assert_eq!(openings.len(), 2);
        assert_eq!(
            openings[0],
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1"
        );
    }
}
