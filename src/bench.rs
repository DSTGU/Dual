use crate::gui::parse_position_command;
use crate::search::{collect_pv, single_depth_search_aspirated}; 
use crate::primitives::shared::{ENDGAME_PERFT, KIWIPETE, START_POSITION, SearchAnswer}; 
use crate::primitives::consts::MIN_DEPTH;
use crate::search_objs::search_state::SearchState;


pub fn test_position(search_state: &mut SearchState, fen: &str, depth: usize) {
    //let board_position = BoardPosition::new(fen);
    let board_position = parse_position_command(search_state, &("position fen ".to_owned() + fen));
    search_state.clear_data();
    search_state.clear_persistent_data();
    search_state.stop_condition.depth = Some(depth);

    let mut local_depth = MIN_DEPTH;
    let mut score = SearchAnswer{eval: 0, move_list: vec![], node_count: 0};
    search_state.reset_for_new_iteration(MIN_DEPTH);

    while !search_state.stop_condition.should_soft_quit(local_depth, search_state.nodes) {
        local_depth += 1;
        
        search_state.reset_for_new_iteration(local_depth);
        score = single_depth_search_aspirated(&board_position, search_state, local_depth, score.eval);
                        
    }

    let time = search_state.stop_condition.started_search.elapsed().as_micros();
    if time == 0 {
        println!("Eval: {}, Depth: {}, Seldepth: {}, nodes: {}, time: 0ms, nps: infinite knps", score.eval, search_state.max_depth, search_state.seldepth, search_state.nodes);
        println!("PV: {}", collect_pv(&score.move_list));
    } else {
        println!("Eval: {}, Depth: {}, Seldepth: {}, nodes: {}, time: {}ms, nps: {}knps", score.eval, search_state.max_depth, search_state.seldepth, search_state.nodes, time/1000, (search_state.nodes * 1000)/time);
        println!("PV: {}", collect_pv(&score.move_list));
    }
}

pub fn bench_engine(search_state: &mut SearchState) {

    println!("Startpos:");
    test_position(search_state, START_POSITION, 13);
    println!("Kiwipete:");
    test_position(search_state, KIWIPETE, 15);
    println!("Endgame pos");
    test_position(search_state, ENDGAME_PERFT, 25);

}