mod shared;
mod attacks;
mod move_gen;
mod perft;
mod gui;
mod search;
mod evaluate;
mod types;

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
use crate::gui::{parse_go, parse_position};
use crate::types::search_state::SearchState;
use crate::shared::{ parse_fen, Piece, START_POSITION};

/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

pub fn uci_loop() {
    println!("id name Dual v0.2.7");
    println!("id author Tomasz Stawowy");
    println!("uciok");
    let mut search_state: SearchState = SearchState::new(parse_fen(START_POSITION));
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
            "go" => parse_go(command, &mut search_state),
            "position" => search_state = parse_position(command),
            "ucinewgame" => search_state = parse_position("position startpos"),
            "uci" => println!("id name Dual v0.2.7\nid author Tomasz Stawowy\nuciok"),
            "printboard" => search_state.board_position.print_board(),
            "printbitboard" => print_bitboard(words[1].parse().unwrap_or_default()),
            "isready" => println!("readyok"),
            // Add more commands here as needed
            _ => println!("Unknown command: {}", command),
        }
    }
}


fn main() {
    let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
    let handler = builder.spawn(|| {
        // thread code
        //
        uci_loop()
    }).unwrap();
    handler.join().unwrap();
}
