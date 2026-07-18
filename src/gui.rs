use crate::evaluation::nnue::NNUE;
use crate::primitives::board::BoardPosition;
use crate::movegen::move_gen::{generate_moves};
use crate::movegen::perft::perft;
use crate::search::{search};
use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::{SearchState};
use crate::primitives::shared::Color::{Black, White};
use crate::primitives::shared::{KIWIPETE, Move, START_POSITION, coordinates_to_squares};
use crate::primitives::shared::Piece::{B, N, Q, R};

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
        "q" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(White) == Q).collect::<Vec<Move>>().pop(),
        "n" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(White) == N).collect::<Vec<Move>>().pop(),
        "b" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(White) == B).collect::<Vec<Move>>().pop(),
        "r" => legal_moves.into_iter().filter(|x| x.get_promoted_piece(White) == R).collect::<Vec<Move>>().pop(),
        _ => legal_moves.pop()
    }
}

// pub fn depth_func(figures: u32) -> usize{
//     ((9.0 / ((figures - 1) as f32).powf(0.20)) + 1.5) as usize
// }

pub fn parse_go(board_position: &BoardPosition, search_state: &mut SearchState, command: &str) {        
    search_state.stop_condition.reset();

    let words : Vec<&str> = command.split_ascii_whitespace().collect();
    let mut wtime : Option<u64> = None;
    let mut btime : Option<u64> = None;
    let mut winc : Option<u64> = None;
    let mut binc : Option<u64> = None;

    for i in 0..words.len()/2 {
        match words[2 * i + 1] {
            "depth" => search_state.stop_condition.depth = Some(words[2*i+2].parse().unwrap_or(6)),
            "perft" => {perft(board_position, words[2*i+2].parse().unwrap_or(4)); return;},
            "wtime" => wtime = Some(words[2*i+2].parse().unwrap_or(1000)),
            "btime" => btime = Some(words[2*i+2].parse().unwrap_or(1000)),
            "winc" => winc = Some(words[2*i+2].parse().unwrap_or(1000)),
            "binc" => binc = Some(words[2*i+2].parse().unwrap_or(1000)),
            "softnodes" => search_state.stop_condition.soft_nodecount = Some(words[2*i+2].parse().unwrap_or(1000)),
            "movetime" => search_state.stop_condition.movetime_deadline = Some(words[2*i+2].parse().unwrap_or(1000)),
            _ => ()
        }
    }

    search_state.stop_condition.our_time_ms = if board_position.side == Black { btime } else { wtime };
    search_state.stop_condition.our_inc_ms =  if board_position.side == Black { binc } else { winc };

    search(board_position, search_state);
}

pub fn parse_ucinewgame(search_state: &mut SearchState) -> BoardPosition {
    search_state.clear_persistent_data();
    parse_position_command(search_state, "position startpos")
}

pub fn parse_position_command(search_state: &mut SearchState, command: &str) -> BoardPosition {
        search_state.clear_data();

        let words : Vec<&str> = command.trim().split(" ").collect();

        if words.len() < 2 {
            return BoardPosition::new(START_POSITION);
        }

        let mut board_position : BoardPosition;

        match words[1] {
            "fen" => {
                board_position = BoardPosition::new(&command[13..]);

                if words.len() > 8 {
                    for &i in words[9..].iter() {
                        let mov = parse_move(&board_position, i);
                        if let Some(x) = mov {
                            let suggestion = board_position.make_move(x);
                            
                            if suggestion.is_none() {
                                return board_position;
                            }

                            search_state.make_move(x, &board_position);
                            board_position = suggestion.unwrap();
                        }
                    }
                }
            },
            "startpos" => {
                board_position = BoardPosition::new(START_POSITION);
                
                for &i in words[2..].iter() {
                        let mov = parse_move(&board_position, i);
                        if let Some(x) = mov {
                            let suggestion = board_position.make_move(x);
                            
                            if suggestion.is_none() {
                                return board_position;
                            }

                            search_state.make_move(x, &board_position);
                            board_position = suggestion.unwrap();
                        }
                }
            },
            "kiwipete" => {
                board_position = BoardPosition::new(KIWIPETE);
                
                for &i in words[2..].iter() {
                        let mov = parse_move(&board_position, i);
                        if let Some(x) = mov {
                            let suggestion = board_position.make_move(x);
                            
                            if suggestion.is_none() {
                                return board_position;
                            }

                            search_state.make_move(x, &board_position);
                            board_position = suggestion.unwrap();
                        }
                }

            },
            _ => board_position = BoardPosition::new(START_POSITION),
        }

        search_state.ply = 0;
        search_state.network_state.start_board(&board_position, &NNUE);

        board_position
}


pub fn parse_setoption(engine_config: &mut EngineConfig, command: &str) {
    let words : Vec<&str> = command.split_ascii_whitespace().collect();

    if words.len() < 3 {
        return;
    }
    //    0       1   2     3    4
    //setoption name <id> value <x>
    match words[2] {
        "Hash" => {
            let val =  words[4..].concat();
            let parse_result = val.parse::<usize>();
            if let Ok(hash) = parse_result {
                engine_config.hash = hash;
            }
        },
        _ => (),
    }

}




#[cfg(test)]
mod tests {
    use crate::gui::{parse_go, parse_position_command};
    use crate::primitives::shared::{START_POSITION};
    use crate::primitives::board::BoardPosition;
    use crate::search_objs::config::EngineConfig;
    use crate::search_objs::search_state::{SearchState};
    use std::thread;


    #[test]
    fn test_position_fen_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder.spawn(|| {
            let board_pos = BoardPosition::new("r1bqkbnr/1p1ppppp/8/p1p5/3nP2P/5N2/PPPPQPP1/RNB1KB1R w KQkq - 2 5");
            let mut search_state = SearchState::new(&EngineConfig::thin());
            let created = parse_position_command(&mut search_state, "position fen r1bqkbnr/1p1ppppp/2n5/p1p5/4P2P/5N2/PPPP1PP1/RNBQKB1R w KQkq - 0 4 moves d1e2 c6d4");
            assert_eq!(board_pos, created);
        }).unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_position_startpos_moves() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let board_pos = BoardPosition::new("rnbqkbnr/ppp1pppp/8/3p4/4P3/8/PPPP1PPP/RNBQKBNR w KQkq d6 0 2");
                let mut search_state = SearchState::new(&EngineConfig::thin());
                let created = parse_position_command(&mut search_state,"position startpos e2e4 d7d5");
                assert_eq!(board_pos, created);
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_go() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let mut search_state = SearchState::new(&EngineConfig::thin());
                let board_position = BoardPosition::new(START_POSITION);
                parse_go(&board_position, &mut search_state,"go depth 6");
            })
            .unwrap();
        handler.join().unwrap();
    }
}
