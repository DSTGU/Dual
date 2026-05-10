use std::{vec};
use std::time::SystemTime;
use crate::evaluate::evaluate;
use crate::move_gen::{generate_moves};
use crate::types::search_state::SearchState;
use crate::shared::{DRAW_SCORE, MIN_DEPTH, Move, SearchAnswer, move_to_alg};

pub fn quiescence(search_state: &mut SearchState, alpha: i32, beta: i32, ply: usize) -> SearchAnswer {

    let eval = evaluate(&search_state.board_position);

    if eval >= beta
    {
        return SearchAnswer { move_list: vec![], node_count: 0, eval: beta };
    }

    let mut new_alpha = alpha;

    if eval > alpha
    {
        new_alpha = eval;
    }

    if search_state.is_trifold_repetition() {
        return SearchAnswer { move_list: vec![], node_count: 0, eval: 0 };
    }

    let move_list = generate_moves(&search_state.board_position);
    let mut filtered_move_list : Vec<Move> = move_list.into_iter().filter(|mv| mv.is_capture() == true).collect();
    filtered_move_list.sort_by(|a, b| {
        let score_a = search_state.get_move_score(*a, search_state.max_depth + ply);
        let score_b = search_state.get_move_score(*b, search_state.max_depth + ply);
        score_b.cmp(&score_a)
    });

    let mut nodes = 1;

    for mv in filtered_move_list {
        let nbp_option = search_state.board_position.make_move(mv);

        // if let Some(nbp) = nbp_option {
        //     let original_position = search_state.get_board_position();
        //     search_state.make_move_for_state(nbp);
        //     let res = quiescence(search_state, -beta, -new_alpha, ply + 1);
        //     search_state.take_back_for_state(original_position);
        //     nodes += res.node_count;

        //     if -res.eval >= beta {
        //         return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
        //     }

        //     if -res.eval > new_alpha {
        //         new_alpha = -res.eval;
        //     }
        // }
    }

    return SearchAnswer { move_list: vec![], node_count: nodes, eval: new_alpha };

}

pub fn negamax(mut search_state: &mut SearchState, alpha: i32, beta: i32, depth: usize) -> SearchAnswer {

    if depth == 0 {
        return quiescence(search_state, alpha, beta, 1);
    }

    if search_state.is_trifold_repetition() {
        return SearchAnswer { move_list: vec![], node_count: 1, eval: DRAW_SCORE };
    }

    let mut new_alpha = alpha;

    let mut move_list = generate_moves(&search_state.board_position);
    move_list.sort_by(|a, b| {
        let score_a = search_state.get_move_score(*a, search_state.max_depth - depth);
        let score_b = search_state.get_move_score(*b, search_state.max_depth - depth);
        score_b.cmp(&score_a)
    });
    
    // Move, eval (alpha), nodes
    let mut nodes = 1;

    let mut best_move = None;
    let mut best_move_list = vec![];

    let mut legal_moves = 0;

    #[allow(non_snake_case, unused_variables)]
    let mut is_PV_node = false;

    for (_idx, mv) in move_list.iter().enumerate() {

        let nbp_option = search_state.board_position.make_move( *mv);

        // if let Some(nbp) = nbp_option {
        //     legal_moves += 1;
        //     let _newdepth = depth-1;
            
        //     let original_position = search_state.get_board_position();
        //     search_state.make_move_for_state(nbp);
        //     let res = negamax(&mut search_state, -beta, -new_alpha, depth-1);
        //     search_state.take_back_for_state(original_position);
        //     nodes += res.node_count;
            

        //     if -res.eval >= beta {                
        //         if mv.get_capture() == false {
        //             search_state.update_killer_move(mv, search_state.max_depth-depth);
        //         }
        //         return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
        //     }

        //     if -res.eval > new_alpha {
        //         if mv.get_capture() == false {
        //             search_state.update_history(mv, depth);
        //         }
                
        //         new_alpha = -res.eval;
        //         best_move = Some(*mv);
        //         best_move_list = res.move_list;
        //         is_PV_node = true;
        //     }
        // }
    }
    
    if legal_moves == 0 {
            if search_state.is_king_attacked() {
                return SearchAnswer { move_list: vec![], node_count: 1, eval: -4999900 - depth as i32};
            }
            else {
                return SearchAnswer { move_list: vec![], node_count: 1, eval: 0};
            }
    }

    best_move_list.push(best_move);
    return SearchAnswer { move_list: best_move_list, node_count: nodes, eval: new_alpha };
}

pub fn score_to_mate( score: i32, depth: usize) -> i32 {
    if score > 0 {
        return (- score + 4999901 + depth as i32 ) / 2
    }
    (- score - 4999900  - depth as i32 ) / 2
}

pub fn collect_pv(moves: &Vec<Option<Move>>) -> String {
    moves
        .iter()
        .filter_map(|x| x.as_ref().map(move_to_alg))
        .rev()
        .reduce(|a, b| a + " " + &b)
        .unwrap_or_default()
}

pub fn single_depth_search(search_state: &mut SearchState, depth: usize) -> SearchAnswer {
    negamax(search_state, -5000000, 5000000, depth)
}

pub fn single_depth_search_aspirated(mut search_state: &mut SearchState, depth: usize, eval: i32) -> SearchAnswer {
    let mut aspiration_lower = 50;
    let mut aspiration_higher = 50;

    let mut score ;

    for _ in 0..3 {
        //println!("low: {}, high: {}", eval-aspiration_lower, eval+aspiration_higher);
        score = negamax(&mut search_state, eval-aspiration_lower, eval+aspiration_higher, depth);
        //println!("aspiration, score: {:?}", score.eval);

        if score.move_list.len() > 0 {
            if score.move_list[0].is_some() {
                //println!("returning: {:?}", score);
                return score;
            }
        }

        //println!("aspiration failed, score: {:?}", score.eval);
        if score.eval < eval {
            aspiration_lower = aspiration_lower * 2;
        }
        else {
            aspiration_higher = aspiration_higher * 2;
        }
    }

    //fallback
    return single_depth_search(search_state, depth);
}


pub fn search(mut search_state: &mut SearchState, depth: Option<usize>, time: Option<usize>) {

    if time.is_none() && depth.is_some() {

        search_state.reset_for_new_search(depth.unwrap());        
        
        let mut score = single_depth_search(&mut search_state, depth.unwrap());

        println!("{:?}", score);

        let pv = collect_pv(&score.move_list);

        if score.eval > 4000000 || score.eval < -4000000 {

            let mate = score_to_mate( score.eval, depth.unwrap());
            println!("info score mate {} depth {} nodes {} pv {}", mate, depth.unwrap(), score.node_count, pv);
        }
        else {
            println!("info score cp {} depth {} nodes {} pv {}", score.eval, depth.unwrap(), score.node_count, pv);
        }
        println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));

    } else {
        let now = SystemTime::now();
        let time_avail;
        if let Some(x) = time {
            time_avail = x/60
        }
        else {
            time_avail = 10000000;
        }

        search_state.reset_for_new_search(MIN_DEPTH);

        let mut score = single_depth_search(search_state, MIN_DEPTH);
        let mut depth = MIN_DEPTH + 1;
        while now.elapsed().unwrap().as_millis() < time_avail as u128 {
        
            search_state.reset_for_new_search(depth);        
            score = single_depth_search_aspirated(&mut search_state, depth, score.eval);

            let pv: String = collect_pv(&score.move_list);

            if score.eval > 4000000 || score.eval < -4000000 {
                let mate = score_to_mate( score.eval, depth);
                println!("info score mate {} depth {} nodes {} pv {}", mate, depth, score.node_count, pv);
            }
            else {
                println!("info score cp {} depth {} nodes {} pv {}", score.eval, depth, score.node_count, pv);
            }
                
            depth = depth + 1;
        }


        println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));
        
    }
    
}


#[cfg(test)]
mod tests {

    use std::thread;
    use crate::{gui::{parse_position}, search::single_depth_search};


    #[test]
    fn test_go() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command = "position fen Q6K/8/8/8/8/8/7R/1k6 w - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                
                let mut board = parse_position(command.trim());
                board.reset_for_new_search(4);       
                let score = single_depth_search(&mut board, 4); 

                println!("{:?}", score);

                assert!(score.node_count < 10);
                assert_eq!(score.eval, 0);
                
            })
            .unwrap();
        handler.join().unwrap();
    }
    // 
}