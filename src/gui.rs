use std::thread;
use crate::moveGen::{generate_moves, make_move};
use crate::perft::perft;
use crate::search::{search};
use crate::shared::{coordinates_to_squares, parse_fen, print_square, start_position, BoardPosition, Move};
use crate::shared::Piece::{b, n, q, r, B, N, Q, R};
use crate::uci_loop;

pub fn parse_move(board: &BoardPosition, moveToParse: &str) -> Option<Move> {

    let legal_moves = generate_moves(board);

    let src = coordinates_to_squares(&moveToParse[0..2]);
    let target = coordinates_to_squares(&moveToParse[2..4]);
    let mut legal_moves : Vec<Move> = legal_moves.into_iter().filter(|x| x.source_square == src as u8 && x.target_square == target as u8).collect();

    if legal_moves.len() < 2 {
        if legal_moves.len() == 0 {
            return None
        }
        return legal_moves.pop();
    }

    let mut piece = 0;

    let char = moveToParse[5..5].to_ascii_lowercase();
    let ch = char.as_str();

    match ch {
        "q" => legal_moves.into_iter().filter(|x| x.promoted_piece == Q || x.promoted_piece == q).collect::<Vec<Move>>().pop(),
        "n" => legal_moves.into_iter().filter(|x| x.promoted_piece == N || x.promoted_piece == n).collect::<Vec<Move>>().pop(),
        "b" => legal_moves.into_iter().filter(|x| x.promoted_piece == b || x.promoted_piece == B).collect::<Vec<Move>>().pop(),
        "r" => legal_moves.into_iter().filter(|x| x.promoted_piece == R || x.promoted_piece == r).collect::<Vec<Move>>().pop(),
        _ => legal_moves.pop()
    }
}

pub fn parse_position(command: &str) -> BoardPosition {
    let words : Vec<&str> = command.split(" ").collect();

    match words[1] {
        "fen" => {
            let mut pos = parse_fen(&command[13..]);
            for &i in words[8..].iter() {
                let mov = parse_move(&pos, i);
                if let Some(x) = mov {
                    pos = make_move(&pos, &x).unwrap();
                }

            }
            pos
        },
        "startpos" => {
            let mut pos = parse_fen(start_position);
            for &i in words[2..].iter() {
                let mov = parse_move(&pos, i);
                if let Some(x) = mov {
                    pos = make_move(&pos, &x).unwrap();
                }

            }
            pos },
        _ => parse_fen(start_position)
    }

}

pub fn depth_func(figures: u32) -> usize{
    ((9.0 / ((figures - 1) as f32).powf(0.28)) + 2.0) as usize
}

pub fn parse_go(command: &str, board_position: &BoardPosition) {
    let mut depth = depth_func(board_position.occupancies[2].count_ones());
    let words : Vec<&str> = command.split_ascii_whitespace().collect();

    for i in 0..words.len()/2 {
        match words[2 * i + 1] {
            "depth" => depth = words[2*i+2].parse().unwrap_or(6),
            "perft" => {perft(board_position, words[2*i+2].parse().unwrap_or(4)); return;}
            _ => ()
        }
    }
    
    let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
    let bp = board_position.clone();
    let handler = builder.spawn(move || {
        search(&bp, depth);
    }).unwrap();
    handler.join().unwrap();

}

#[cfg(test)]
mod tests {
    use std::thread;
    use crate::gui::{parse_go, parse_position};
    use crate::moveGen::{is_square_attacked, make_move, run_through_attacks};
    use crate::shared::{coordinates_to_squares, parse_fen, start_position, Move, Piece};

    #[test]
    fn test_position_startpos() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 xdddddd");
            let cmdResult = parse_position("position startpos");
            assert_eq!(boardPos, cmdResult);
        }).unwrap();
        handler.join().unwrap();
    }


    #[test]
    fn test_position_fen() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4");
            let cmdResult = parse_position("position fen r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4");
            assert_eq!(boardPos, cmdResult);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_fen_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("r1bqkbnr/1p1ppppp/8/p1p5/3nP2P/5N2/PPPPQPP1/RNB1KB1R w KQkq - 2 5");
            let cmdResult = parse_position("position fen r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4 d1e2 c6d4");
            assert_eq!(boardPos, cmdResult);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_startpos_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let boardPos = parse_fen("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
            let cmdResult = parse_position("position startpos e2e4 d7d5");
            assert_eq!(boardPos, cmdResult);
        }).unwrap();
        handler.join().unwrap();
    }


    #[test]
    fn test_go() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let board = parse_fen(start_position);
            parse_go("go depth 13", &board);
        }).unwrap();
        handler.join().unwrap();
    }
}