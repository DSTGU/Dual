mod shared;
mod attacks;
mod moveGen;
mod perft;
mod gui;
mod search;
mod evaluate;

use std::{default, env, io};
use std::thread;
use std::time::SystemTime;
use shared::Sq;
use shared::BoardPosition;

/**********************************\
 ==================================

          Bit manipulations

 ==================================
\**********************************/

use shared::set_bit;
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
use crate::attacks::get_bishop_attacks;
use crate::gui::{parse_go, parse_position};
use crate::shared::{ parse_fen, Piece, print_board, start_position, coordinates_to_squares};

/**********************************\
 ==================================

             Main driver

 ==================================
\**********************************/

pub fn uci_loop() {
    println!("id name Dual v0.2.7");
    println!("id author Tomasz Stawowy");
    println!("uciok");
    let mut boardpos : BoardPosition = parse_fen(start_position);
    loop {  
        // Read user input
        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();

        // Trim the input to remove newline characters
        let command = input.trim();
        let words : Vec<&str> = command.split_ascii_whitespace().collect();

        // Handle the command
        match words[0] {
            "exit" => return,
            "go" => parse_go(command, &boardpos),
            "position" => boardpos = parse_position(command),
            "ucinewgame" => boardpos = parse_position("position startpos"),
            "uci" => println!("id name Dual v0.2.7\nid author Tomasz Stawowy\nuciok"),
            "printboard" => print_board(&boardpos),
            "isready" => println!("readyok"),
            "rw" => print_bitboard(get_bishop_attacks(coordinates_to_squares("b3"),0)),
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
