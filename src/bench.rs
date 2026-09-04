use std::ops::AddAssign;

use crate::gui::parse_position_command;
use crate::search::search; 
use crate::primitives::shared::{ENDGAME_PERFT, KIWIPETE, START_POSITION}; 
use crate::search_objs::search_state::{Reporting, SearchState};


pub fn test_position(search_state: &mut SearchState, fen: &str, depth: i32) -> BenchResults { // nodes
    //let board_position = BoardPosition::new(fen);
    let board_position = parse_position_command(search_state, &("position fen ".to_owned() + fen));
    search_state.clear_data();
    search_state.clear_persistent_data();
    search_state.stop_condition.depth = Some(depth);

    search(&board_position, search_state);

    let time = search_state.stop_condition.started_search.elapsed().as_micros();
    BenchResults{nodes: search_state.nodes, time: time}
}


#[derive(Default, Clone, Copy)]
pub struct BenchResults {
    nodes: u64,
    time: u64,
}

impl AddAssign for BenchResults {
    fn add_assign(&mut self, rhs: Self) {
        self.nodes += rhs.nodes;
        self.time += rhs.time;
    }
}

pub fn bench_engine(search_state: &mut SearchState) {

    let reporting = search_state.reporting;
    search_state.reporting = Reporting::Quiet;

    let mut results = BenchResults{nodes: 0, time: 0};
    results += test_position(search_state, START_POSITION, 13); // (u64, u64)
    results += test_position(search_state, KIWIPETE, 15);
    results += test_position(search_state, ENDGAME_PERFT, 25);

    println!("{} nodes {} nps", results.nodes, results.nodes * 1000000 / results.time);
    search_state.reporting = reporting;
}