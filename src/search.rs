use std::{thread, vec};
use std::time::SystemTime;
use crate::evaluate::evaluate;
use crate::move_gen::{generate_moves, is_square_attacked, make_move};
use crate::shared::{BoardPosition, Move, SearchAnswer, SearchState, get_bit, move_to_alg};

static mut MAX_DEPTH : usize = 0;
static mut KILLER_MOVE : [[u32; 256]; 2 ] = [[0; 256]; 2];
static mut HISTORY_MOVE : [[usize; 64]; 12 ] = [[0; 64]; 12];
static mut PREVITER_BESTMOVE : u32 = 0;

// pub fn get_victim(board_position: &BoardPosition, mv: &Move) -> usize {
//     let sidevar = ((board_position.side + 1) % 2) * 6;

//     for i in 0+sidevar..6+sidevar {
//         if get_bit(board_position.bitboards[i], mv.get_target_square() as usize) {
//             return i;
//         }
//     }

//     0
// }

// pub fn get_move_score(board_position: &BoardPosition, mv: &Move, ply: usize) -> usize {

//     unsafe {
//         if ply == 0 && mv.mv == PREVITER_BESTMOVE {
//             return 605000001;
//         }
//     }

//     //println!("ply: {}", ply);

//     if mv.get_capture() == true {
//         let victim = get_victim(board_position, mv);
//         return get_MVV_LVA(victim, mv.get_piece() as usize);
//     }
//     else {
//         unsafe {
//             if KILLER_MOVE[0][ply] == mv.mv {
//                 return 9000000;
//             }
//             if KILLER_MOVE[1][ply] == mv.mv {
//                 return 8000000;
//             }
//                 return HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize];
//         }
//     }
// }


// pub fn rand_search(board_position: &BoardPosition) {

//     let mut moves = generate_moves(board_position);
    
//     let mut mv = moves.pop();
    
//     while mv.is_none() {
//         mv = moves.pop();
//     }
    
//     println!("bestmove {}", move_to_alg(&mv.unwrap()))
// }

// (eval, nodes)
pub fn quiescence(search_state: &mut SearchState, alpha: i32, beta: i32, ply: usize) -> SearchAnswer {

    let eval = evaluate(&search_state.get_board_position());

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

    let move_list = generate_moves(&search_state.get_board_position());
    let mut filtered_move_list : Vec<Move> = move_list.into_iter().filter(|mv| mv.get_capture() == true).collect();
    filtered_move_list.sort_by(|a, b| {
        unsafe {
            let score_a = search_state.get_move_score(a, MAX_DEPTH + ply);
            let score_b = search_state.get_move_score(b, MAX_DEPTH + ply);
            score_b.cmp(&score_a)
        }
    });

    let mut nodes = 1;

    for mv in filtered_move_list {
        let nbp_option = make_move(&search_state.get_board_position(), &mv);

        if let Some(nbp) = nbp_option {
            let original_position = search_state.get_board_position();
            search_state.make_move_for_state(nbp);
            let res = quiescence(search_state, -beta, -new_alpha, ply + 1);
            search_state.take_back_for_state(original_position);
            nodes += res.node_count;

            if -res.eval >= beta {
                return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
            }

            if -res.eval > new_alpha {
                new_alpha = -res.eval;
            }
        }
    }

    return SearchAnswer { move_list: vec![], node_count: nodes, eval: new_alpha };

}

pub fn negamax(mut search_state: &mut SearchState, alpha: i32, beta: i32, depth: usize) -> SearchAnswer {

    if depth == 0 {
        return quiescence(search_state, alpha, beta, 1);
    }

    let mut new_alpha = alpha;

    let mut move_list = generate_moves(&search_state.get_board_position());
    move_list.sort_by(|a, b| {
        unsafe {
        let score_a = search_state.get_move_score(a, MAX_DEPTH - depth);
        let score_b = search_state.get_move_score(b, MAX_DEPTH - depth);
        score_b.cmp(&score_a)
        }
    });
    
    // Move, eval (alpha), nodes
    let mut nodes = 1;

    let mut best_move = None;
    let mut best_move_list = vec![];

    let mut legal_moves = 0;

    #[allow(non_snake_case, unused_variables)]
    let mut is_PV_node = false;

    for (_idx, mv) in move_list.iter().enumerate() {

        let nbp_option = make_move(&search_state.get_board_position(), mv);

        if let Some(nbp) = nbp_option {
            legal_moves += 1;
            let _newdepth = depth-1;
            
            let original_position = search_state.get_board_position();
            search_state.make_move_for_state(nbp);
            let res = negamax(&mut search_state, -beta, -new_alpha, depth-1);
            search_state.take_back_for_state(original_position);
            nodes += res.node_count;
            

            if -res.eval >= beta {                
                if mv.get_capture() == false {
                    unsafe {
                        KILLER_MOVE[1][MAX_DEPTH - depth] = KILLER_MOVE[0][MAX_DEPTH - depth];
                        KILLER_MOVE[0][MAX_DEPTH - depth] = mv.mv;
                    }
                }
                return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
            }

            if -res.eval > new_alpha {
                
                if mv.get_capture() == false {
                    unsafe {
                        HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize] += depth;
                        //println!( "{}, {} - {} -> {}", depth, mv.get_piece(), mv.get_target_square(), HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize])
                    }
                }
                
                new_alpha = -res.eval;
                best_move = Some(*mv);
                best_move_list = res.move_list;
                is_PV_node = true;
            }
        }
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

    loop {
        println!("low: -{}, high: {}", aspiration_lower, aspiration_higher);
        score = negamax(&mut search_state, eval-aspiration_lower, eval+aspiration_higher, depth);

        //println!("aspiration, score: {:?}", score);

        if score.move_list.len() > 0 {
            if score.move_list[0].is_some() {
                //println!("returning: {:?}", score);
                return score;
            }
        }

        //println!("aspiration failed, score: {:?}", score);
        if score.eval < eval {
            aspiration_lower = aspiration_lower * 2;
        }
        else {
            aspiration_higher = aspiration_higher * 2;
        }
    }
}


pub fn search(mut search_state: &mut SearchState, depth: Option<usize>, time: Option<usize>) {

    unsafe {
        KILLER_MOVE = [[0; 256]; 2];
        HISTORY_MOVE = [[0; 64]; 12];
        PREVITER_BESTMOVE = 0;
    }


    if time.is_none() && depth.is_some() {

        unsafe {
            MAX_DEPTH = depth.unwrap();
        }

        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);

        //let handler = builder.spawn(move || {
            let mut score = single_depth_search(&mut search_state, depth.unwrap());

            let pv = collect_pv(&score.move_list);

            if score.eval > 4000000 || score.eval < -4000000 {

                let mate = score_to_mate( score.eval, depth.unwrap());

                println!("info score mate {} depth {} nodes {} pv {}", mate, depth.unwrap(), score.node_count, pv);
            }
            else {
                println!("info score cp {} depth {} nodes {} pv {}", score.eval, depth.unwrap(), score.node_count, pv);
            }
            println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));
            
        //}).unwrap();
        //handler.join().unwrap();
    } else {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let now = SystemTime::now();
        let time_avail;
        if let Some(x) = time {
            time_avail = x/60
        }
        else {
            time_avail = 10000000;
        }
        let mut depth = 3;
        unsafe {
            MAX_DEPTH = depth;
            KILLER_MOVE = [[0; 256]; 2];
            HISTORY_MOVE = [[0; 64]; 12];
            PREVITER_BESTMOVE = 0;
        }
        let bp = search_state.get_board_position().clone();
        let mut score = single_depth_search(search_state, 3);
        depth = 4;
        //let handler = builder.spawn(move || {
            while now.elapsed().unwrap().as_millis() < time_avail as u128 {
                unsafe {
                    MAX_DEPTH = depth;
                    KILLER_MOVE = [[0; 256]; 2];
                    HISTORY_MOVE = [[0; 64]; 12];
                    //previter_bestmove = score.0.pop().unwrap().unwrap().mv;
                }
                
                score = single_depth_search_aspirated(&mut search_state, depth, score.eval);

                let pv = collect_pv(&score.move_list);

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
        //}).unwrap();
        //handler.join().unwrap();
        
    }
    
}



