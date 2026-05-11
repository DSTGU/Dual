use std::time::SystemTime;

use crate::search::{collect_pv, single_depth_search_aspirated}; 
use crate::shared::{ENDGAME_PERFT, KIWIPETE, MIN_DEPTH, Move, START_POSITION, SearchAnswer}; 
use crate::types::{board::BoardPosition, search_state::SearchState};


pub fn test_position(fen: &str, depth: usize) {
    let mut search_state = &mut SearchState::new(BoardPosition::new(fen));

    let now = SystemTime::now();
    let mut local_depth = MIN_DEPTH;
    let mut score = SearchAnswer{eval: 0, move_list: vec![], node_count: 0};
    let mut total_node_count = 0;

    while local_depth <= depth {
            search_state.reset_for_new_search(depth, Move::create_null());

            score = single_depth_search_aspirated(&mut search_state, local_depth, score.eval);
                        
            local_depth = local_depth + 1;
            total_node_count += score.node_count;
    }

    let time = now.elapsed().unwrap().as_millis();
    println!("Eval: {}, Depth: {}, Seldepth: {}, nodes: {}, time: {}, nps: {}knps", score.eval, search_state.max_depth, search_state.seldepth, total_node_count, time, total_node_count as u128/time);
    println!("PV: {}", collect_pv(&score.move_list));
}

pub fn bench_engine() {

    println!("Startpos:");
    test_position(START_POSITION, 7);
    println!("Kiwipete:");
    test_position(KIWIPETE, 6);
    println!("Endgame pos");
    test_position(ENDGAME_PERFT, 10);

}