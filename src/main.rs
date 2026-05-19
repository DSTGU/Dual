mod shared;
mod attacks;
mod move_gen;
mod perft;
mod gui;
mod search;
mod evaluate;
mod types;
mod bench;

use std::io;
use std::thread;

/**********************************\
 ==================================

          Bit manipulations

 ==================================
\**********************************/

use shared::get_bit;
use shared::pop_bit;

use shared::print_bitboard;

/**********************************\
 ==================================

              Attacks

 ==================================
\**********************************/

use attacks::PAWN_ATTACKS;
use attacks::KNIGHT_ATTACKS;
use attacks::KING_ATTACKS;
use crate::bench::bench_engine;
use crate::gui::{parse_go};
use crate::types::search_state::SearchState;
use crate::shared::{ Piece, START_POSITION};

/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

pub fn print_identification() {
    println!("id name Dual v0.2.9");
    println!("id author Tomasz Stawowy");
    println!("uciok");
}

pub fn uci_loop() {

    let mut search_state: SearchState = SearchState::new(START_POSITION);
    loop {  
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Trim the input to remove newline characters
        let command = input.trim();
        let words : Vec<&str> = command.split_ascii_whitespace().collect();

        if words.len() == 0 {
            continue;
        }

        // Handle the command
        match words[0] {
            "exit" => return,
            "quit" => return,
            "go" => parse_go(command, &mut search_state),
            "position" => search_state.parse_position_command(command),
            "ucinewgame" => search_state.parse_position_command("position startpos"),
            "uci" => print_identification(),
            "printboard" => search_state.board_position.print_board(),
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
