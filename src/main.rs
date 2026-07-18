mod gui;
mod movegen;
mod search;
mod search_objs;
mod evaluation;
mod primitives;
mod bench;

use std::io;
use std::thread;
use primitives::shared::{get_bit, pop_bit, print_bitboard, Piece};
use movegen::attacks::PAWN_ATTACKS;
use movegen::attacks::KNIGHT_ATTACKS;
use movegen::attacks::KING_ATTACKS;
use crate::bench::bench_engine;
use crate::evaluation::evaluate::evaltest;
use crate::gui::parse_position_command;
use crate::gui::parse_setoption;
use crate::gui::parse_ucinewgame;
use crate::gui::{parse_go};
use crate::primitives::board::BoardPosition;
use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::SearchState;

/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

pub fn print_identification() {
    println!("id name Dual v0.3.2");
    println!("id author Tomasz Stawowy");
    println!("option name Hash type spin default 256 min 0 max 1024");
    println!("uciok");
}

pub fn uci_loop() {

    let mut engine_config: EngineConfig = EngineConfig::default();
    let mut search_state: SearchState = SearchState::new(&engine_config);
    let mut board_position: BoardPosition = parse_position_command(&mut search_state, "position startpos");
    loop {  
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Trim the input to remove newline characters
        let command = input.trim();
        let words : Vec<&str> = command.split_ascii_whitespace().collect();

        if words.is_empty() {
            continue;
        }

        // Handle the command
        match words[0] {
            "exit" => return,
            "quit" => return,
            "go" => parse_go(&board_position, &mut search_state, command),
            "position" => {board_position = parse_position_command(&mut search_state, command)},
            "eval" => evaltest(&board_position, &search_state),
            "ucinewgame" => {board_position = parse_ucinewgame(&mut search_state)},
            "uci" => print_identification(),
            "setoption" => {
                parse_setoption(&mut engine_config, command);
                search_state = SearchState::new(&engine_config);
                board_position = parse_position_command(&mut search_state, "position startpos");
            }
            "printboard" => board_position.print_board(),
            "printbitboard" => print_bitboard(words[1].parse().unwrap_or_default()),
            "isready" => println!("readyok"),
            "bench" => bench_engine(&mut search_state),
            // Add more commands here as needed
            _ => println!("Unknown command: {}", command),
        }
    }
}


fn main() {
    print_identification();
    let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
    let handler = builder.spawn(|| {
        // thread code
        //
        uci_loop()
    }).unwrap();
    handler.join().unwrap();
}
