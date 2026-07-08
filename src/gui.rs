use crate::types::board::BoardPosition;
use crate::move_gen::{generate_moves};
use crate::perft::perft;
use crate::search::{search};
use crate::types::search_state::SearchState;
use crate::types::shared::{Move, coordinates_to_squares};
use crate::types::shared::Piece::{B, N, Q, R};

pub fn parse_move(board: &BoardPosition, move_to_parse: &str) -> Option<Move> {

    let legal_moves = generate_moves(board, false);

    let src = coordinates_to_squares(&move_to_parse[0..2]);
    let target = coordinates_to_squares(&move_to_parse[2..4]);
    let mut legal_moves : Vec<Move> = legal_moves.into_iter().filter(|x| x.get_source_square() == src && x.get_target_square() == target).collect();

    if legal_moves.len() < 2 {
        if legal_moves.is_empty(){
            return None
        }
        return legal_moves.pop();
    }
    
    let char = move_to_parse[4..5].to_ascii_lowercase();
    let ch = char.as_str();

    match ch {
        "q" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(false) == Q).collect::<Vec<Move>>().pop(),
        "n" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(false) == N).collect::<Vec<Move>>().pop(),
        "b" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(false) == B).collect::<Vec<Move>>().pop(),
        "r" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(false) == R).collect::<Vec<Move>>().pop(),
        _ => legal_moves.pop()
    }
}

// pub fn depth_func(figures: u32) -> usize{
//     ((9.0 / ((figures - 1) as f32).powf(0.20)) + 1.5) as usize
// }

pub fn parse_go(command: &str, search_state: &mut SearchState) {
    let mut depth = None; //depth_func(board_position.occupancies[2].count_ones());
    let words : Vec<&str> = command.split_ascii_whitespace().collect();
    let mut wtime : Option<usize> = None;
    let mut btime : Option<usize> = None;
    let mut winc : Option<usize> = None;
    let mut binc : Option<usize> = None;
    let mut movetime : Option<usize> = None;

    for i in 0..words.len()/2 {
        match words[2 * i + 1] {
            "depth" => depth = Some(words[2*i+2].parse().unwrap_or(6)),
            "perft" => {perft(search_state, words[2*i+2].parse().unwrap_or(4)); return;},
            "wtime" => wtime = Some(words[2*i+2].parse().unwrap_or(1000)),
            "btime" => btime = Some(words[2*i+2].parse().unwrap_or(1000)),
            "winc" => winc = Some(words[2*i+2].parse().unwrap_or(1000)),
            "binc" => binc = Some(words[2*i+2].parse().unwrap_or(1000)),
            "movetime" => movetime = Some(words[2*i+2].parse().unwrap_or(1000)),
            _ => ()
        }
    }

    if movetime.is_some() {
        search(search_state, depth, movetime);
        return;
    }

    let time : Option<usize> = if search_state.board_position.side == 1 { btime } else { wtime };
    let inc: Option<usize> =  if search_state.board_position.side == 1 { binc } else { winc };

    let time_available : Option<usize> = time.map(|timeval| (timeval/20 + inc.unwrap_or(0)/2).min(timeval*3/4));
    search(search_state, depth, time_available);
}

#[cfg(test)]
mod tests {
    use crate::gui::{parse_go};
    use crate::types::shared::{START_POSITION};
    use crate::types::board::BoardPosition;
    use crate::types::search_state::{SearchState};
    use std::thread;

    #[test]
    fn test_position_startpos() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos: BoardPosition = BoardPosition::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 xdddddd");
                let mut search_state = SearchState::new(START_POSITION);
                search_state.parse_position_command("position startpos");
                assert_eq!(board_pos, search_state.board_position);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_fen() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let board_pos = BoardPosition::new("r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4");
            let mut search_state = SearchState::new(START_POSITION);
            search_state.parse_position_command("position fen r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4");
            assert_eq!(board_pos, search_state.board_position);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_fen_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let board_pos = BoardPosition::new("r1bqkbnr/1p1ppppp/8/p1p5/3nP2P/5N2/PPPPQPP1/RNB1KB1R w KQkq - 2 5");
            let mut search_state = SearchState::new(START_POSITION);
            search_state.parse_position_command("position fen r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4 moves d1e2 c6d4");
            assert_eq!(board_pos, search_state.board_position);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_startpos_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = BoardPosition::new("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
                let mut search_state = SearchState::new(START_POSITION);
                search_state.parse_position_command("position startpos e2e4 d7d5");
                assert_eq!(board_pos, search_state.board_position);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_go() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(START_POSITION);
                parse_go("go depth 6", &mut search_state);
            })
            .unwrap();
        handler.join().unwrap();
    }
}
