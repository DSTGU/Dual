mod attacks;
mod move_gen;
mod perft;
mod gui;
mod search;
mod evaluate;
mod types;
mod bench;
mod nnue;

use std::io;
use std::thread;
use types::shared::{get_bit, pop_bit, print_bitboard, Piece};
use attacks::PAWN_ATTACKS;
use attacks::KNIGHT_ATTACKS;
use attacks::KING_ATTACKS;
use crate::bench::bench_engine;
use crate::evaluate::evaltest;
use crate::gui::parse_position_command;
use crate::gui::parse_ucinewgame;
use crate::gui::{parse_go};
use crate::types::board::BoardPosition;
use crate::types::search_state::SearchState;

/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

pub fn print_identification() {
    println!("id name Dual v0.3.2");
    println!("id author Tomasz Stawowy");
    println!("uciok");
}

pub fn uci_loop() {

    let mut search_state: SearchState = SearchState::new();
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
