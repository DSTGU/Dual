use std::{vec};
use coarsetime::{Duration, Instant};

use crate::evaluate::evaluate;
use crate::move_gen::{generate_moves, is_square_attacked};
use crate::types::search_state::SearchState;
use crate::shared::{DRAW_SCORE, MATE_SCORE, MIN_DEPTH, Move, MoveSuccess, Piece, SearchAnswer, move_to_alg};
use crate::types::tt::TTFlag;

pub fn sort_move_list(search_state: &mut SearchState, move_list: Vec<Move>) -> Vec<Move> {
    let mut scored_moves: Vec<(Move, i32)> = move_list
        .into_iter()
        .map(|m| {
            let score = if Some(m) == search_state.tt_move() {
                i32::MAX
            } else {
                search_state.get_move_score(m) as i32
            };

            (m, score)
        })
        .collect();

    scored_moves.sort_unstable_by_key(|&(_, score)| -score);
    scored_moves.into_iter().map(|(mv, _)| mv).collect()
}

pub fn reduce_lmr_by(depth: usize, moves: usize) -> usize {
    // Obsidian function
    (0.99 + (depth as f32).ln() * (moves as f32).ln() / 3.14) as usize
}

pub fn quiescence(search_state: &mut SearchState, alpha: i32, beta: i32, ply: usize) -> SearchAnswer {

    search_state.seldepth = search_state.seldepth.max(search_state.max_depth+ply-1);

    //PESTO eval
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

    let move_list = generate_moves(&search_state.board_position, true);
    let filtered_move_list = sort_move_list(search_state, move_list);

    let mut nodes = 1;

    for mv in filtered_move_list {
        let move_result = search_state.make_move(mv);
        
        if move_result == MoveSuccess::Success {
            let res = quiescence(search_state, -beta, -new_alpha, ply + 1);
            search_state.take_back(mv);
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

pub fn pvs(mut search_state: &mut SearchState, alpha: i32, beta: i32, depth: usize) -> SearchAnswer {

    let is_pv_node = beta - alpha > 1;

    if search_state.should_quit() {
       return SearchAnswer { move_list: vec![], node_count: 1, eval: 0};  
    }
    
    if depth == 0 {
        return quiescence(search_state, alpha, beta, 1);
    }

    if search_state.is_trifold_repetition() {
        return SearchAnswer { move_list: vec![], node_count: 1, eval: DRAW_SCORE };
    }

    let mut new_alpha = alpha;



    // ------------------------------------------------------------
    // TT probe
    // ------------------------------------------------------------
    let probe = search_state.probe_tt();
    
    if let Some(entry) = probe {

        if entry.depth >= depth as i32 && !search_state.is_twofold_repetition() {

            match entry.flag {

                TTFlag::Exact => {
                    return SearchAnswer {
                        move_list: vec![Some(entry.best_move)],
                        node_count: 1,
                        eval: entry.score,
                    };
                }

                TTFlag::Alpha => {
                    if entry.score <= alpha {
                        return SearchAnswer {
                            move_list: vec![],
                            node_count: 1,
                            eval: alpha,
                        };
                    }
                }

                TTFlag::Beta => {
                    if entry.score >= beta {
                        return SearchAnswer {
                            move_list: vec![Some(entry.best_move)],
                            node_count: 1,
                            eval: beta,
                        };
                    }
                }
            }
        }
    }

    // ------------------------------------------------------------
    // Static eval
    // ------------------------------------------------------------

    //Todo: move to movegen
    let our_king = if search_state.board_position.side == 0 { Piece::K } else {Piece::k};
    let is_in_check = is_square_attacked(search_state.board_position.bitboards[our_king as usize].trailing_zeros() as u8, &search_state.board_position);

    let static_eval =  if is_in_check {
        -MATE_SCORE
    } else if probe.is_some() && probe.unwrap().flag == TTFlag::Exact {
        probe.unwrap().score
    } else {
        evaluate(&search_state.board_position)
    };

    // ------------------------------------------------------------
    // Reverse Futility Pruning (beta pruning)
    //
    // "Position is so good that even after margin reduction
    //  we still exceed beta."
    // ------------------------------------------------------------
    if !is_pv_node
       && depth <= 6
       && !is_in_check
       && static_eval - (150*depth) as i32 >= beta {
            return SearchAnswer {
                move_list: vec![],
                node_count: 1,
                eval: static_eval,
            };
       }

    // ------------------------------------------------------------
    // Move generation / ordering
    // ------------------------------------------------------------
    let move_list = generate_moves(&search_state.board_position, false);
    let move_list = sort_move_list(search_state, move_list);

    // Move, eval (alpha), nodes
    let mut nodes = 1;

    let mut best_move = None;
    let mut best_move_list = vec![];

    let mut legal_moves = 0;

    for &mv in move_list.iter() {
        // --------------------------------------------------------
        // Futility pruning
        //
        // "Quiet move cannot raise alpha enough."
        // --------------------------------------------------------

        if !is_pv_node && 
            depth <= 5 &&
            legal_moves > 1 &&
            mv.is_quiet() &&
            !is_in_check {
                if static_eval + 150 * depth as i32 <= alpha {
                    continue;
                }
            }
            
        // --------------------------------------------------------
        // LMR (Late Move Reductions)
        // --------------------------------------------------------
        let mut reduction = 0;
        if depth >= 3 &&
           legal_moves > 1 &&
           mv.is_quiet() &&
           !is_pv_node {
           //and not inCheck
           //and not givesCheck:

            reduction = reduce_lmr_by(depth, legal_moves);
                            //improving);

            // Often reduce less for good-history moves
            //reduction -= historyBonus(move)

            reduction = reduction.clamp(0, depth - 1);
        }

        let move_result = search_state.make_move( mv);

        if move_result == MoveSuccess::Success {
            legal_moves += 1;

                // --------------------------------------------------------
                // PVS SEARCH LOGIC
                //
                // First move:
                //   full window
                //
                // Later moves:
                //   null window first
                //
                // LMR usually applies ONLY to the null-window search.
                // --------------------------------------------------------
            
            if legal_moves == 1 {
                let score: SearchAnswer = pvs(&mut search_state, -beta, -new_alpha, depth-1);
                search_state.take_back(mv);
                nodes += score.node_count;

                if -score.eval > new_alpha {
                    if -score.eval >= beta {
                        if search_state.should_quit() {
                            return SearchAnswer { move_list: vec![], node_count: nodes, eval: 0};
                        }
                        
                        if mv.is_quiet() {
                            search_state.update_killer_move(mv);
                        }

                        search_state.store_tt(
                            depth,
                            beta,
                            TTFlag::Beta,
                            mv,
                        );

                        return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
                    }

                    if mv.is_quiet() {
                        search_state.update_history(mv, depth);
                    }

                    new_alpha = -score.eval;
                    best_move = Some(mv);
                    best_move_list = score.move_list;
                }

            } else {
                // ----------------------------------------------------
                // First try:
                // reduced + null-window
                // ----------------------------------------------------

                let mut score = pvs(&mut search_state, -new_alpha-1, -new_alpha, depth-1-reduction);
                nodes += score.node_count;


                // ----------------------------------------------------
                // Case 1:
                // Reduced search failed high
                //
                // The move may actually be good.
                //
                // Re-search at FULL DEPTH still using null window.
                // ----------------------------------------------------
                if reduction > 0 && -score.eval > new_alpha {
                    score = pvs(&mut search_state, -new_alpha-1, -new_alpha, depth - 1);
                } 

                // ----------------------------------------------------
                // Case 2:
                // Null-window search indicates possible PV move
                //
                // Need full-window re-search.
                // ----------------------------------------------------
                if -score.eval > new_alpha && -score.eval < beta  {
                    // research with window [alfa;beta]
                    score = pvs(&mut search_state, -beta, -new_alpha, depth-1);
                    nodes += score.node_count;

                }

                search_state.take_back(mv);

                if -score.eval > new_alpha {
                    if -score.eval >= beta {
                        
                        if search_state.should_quit() {
                            return SearchAnswer { move_list: vec![], node_count: nodes, eval: 0};
                        }

                        search_state.store_tt(
                            depth,
                            beta,
                            TTFlag::Beta,
                            mv,
                        );

                        if mv.is_quiet() {
                            search_state.update_killer_move(mv);
                        }
                        return SearchAnswer { move_list: vec![], node_count: nodes, eval: beta };
                    }

                    if mv.is_quiet() {
                        search_state.update_history(mv, depth);
                    }

                    new_alpha = -score.eval;
                    best_move = Some(mv);
                    best_move_list = score.move_list;
                }

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

    if search_state.should_quit() {
       return SearchAnswer { move_list: vec![], node_count: nodes, eval: 0};
    }

    let flag = if new_alpha <= alpha {
        TTFlag::Alpha
    } else if new_alpha >= beta {
        TTFlag::Beta
    } else {
        TTFlag::Exact
    };

    search_state.store_tt(
        depth,
        new_alpha,
        flag,
        best_move.unwrap_or(Move::create_null()),
    );
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
    pvs(search_state, -5000000, 5000000, depth)
}

pub fn single_depth_search_aspirated(mut search_state: &mut SearchState, depth: usize, eval: i32) -> SearchAnswer {
    let mut aspiration_lower = 50;
    let mut aspiration_higher = 50;

    let mut score ;

    for _ in 0..3 {
        //println!("low: {}, high: {}", eval-aspiration_lower, eval+aspiration_higher);
        score = pvs(&mut search_state, eval-aspiration_lower, eval+aspiration_higher, depth);
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


pub fn search(mut search_state: &mut SearchState, depth: Option<usize>, time_available: Option<usize>) {

    if time_available.is_none() && depth.is_some() {
        search_state.set_deadline(Instant::now().checked_add(Duration::from_secs(1000000)).unwrap());
        if depth.unwrap() <= MIN_DEPTH {
            search_state.reset_for_new_iteration(depth.unwrap(), Move::create_null());        
            let mut score: SearchAnswer = single_depth_search(&mut search_state, depth.unwrap());
            print_info_string(&score, search_state, depth.unwrap());
            println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));
        } else {
            search_state.reset_for_new_iteration(MIN_DEPTH, Move::create_null());

            let mut score: SearchAnswer = single_depth_search(search_state, MIN_DEPTH);
            
            print_info_string(&score, search_state, MIN_DEPTH);
            
            let mut curr_depth = MIN_DEPTH + 1;
            search_state.reset_for_new_iteration(curr_depth, score.move_list.get(score.move_list.len() - 1).unwrap().unwrap());        

            while curr_depth <= depth.unwrap()   {
            
                score = single_depth_search_aspirated(&mut search_state, curr_depth, score.eval);
                
                print_info_string(&score, search_state, curr_depth);
                
                curr_depth = curr_depth + 1;
                
                search_state.reset_for_new_iteration(curr_depth, score.move_list.get(score.move_list.len() - 1).unwrap().unwrap());        
            }


            println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));
        }
        
        


    } else {
        let now: Instant = Instant::now();
        let time_avail: usize;
        if let Some(x) = time_available {
            time_avail = x;
        }
        else {
            time_avail = 1000;
        }

        search_state.set_deadline(Instant::now().checked_add(Duration::from_millis(time_avail as u64)).unwrap());

        search_state.reset_for_new_iteration(MIN_DEPTH, Move::create_null());

        let mut score: SearchAnswer = single_depth_search(search_state, MIN_DEPTH);
        
        print_info_string(&score, search_state, MIN_DEPTH);
        
        let mut depth = MIN_DEPTH + 1;

        search_state.reset_for_new_iteration(depth, score.move_list.get(score.move_list.len() - 1).unwrap().unwrap());        

        while now.elapsed().as_millis() < (time_avail/3) as u64 {
        
            let new_score = single_depth_search_aspirated(&mut search_state, depth, score.eval);
            if (!search_state.should_quit() && score.move_list.len() > 0) {
                score = new_score;
                print_info_string(&score, search_state, depth);
            }

            
            depth = depth + 1;
            
            search_state.reset_for_new_iteration(depth, score.move_list.get(score.move_list.len() - 1).unwrap().unwrap());        
        }

        println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));
        
    }
    
}

pub fn print_info_string(score: &SearchAnswer, search_state: &SearchState, depth: usize) {
    let pv: String = collect_pv(&score.move_list);

    if score.eval > 4000000 || score.eval < -4000000 {
        let mate = score_to_mate( score.eval, depth);
        println!("info score mate {} depth {} seldepth {} nodes {} pv {}", mate, depth, search_state.seldepth, score.node_count, pv);
    }
    else {
        println!("info score cp {} depth {} seldepth {} nodes {} pv {}", score.eval, depth, search_state.seldepth, score.node_count, pv);
    }
}


#[cfg(test)]
mod tests {

    use std::thread;
    use crate::search::single_depth_search;
    use crate::shared::{Move, START_POSITION};
    use crate::types::search_state::SearchState;


    #[test]
    fn test_forced_trifold_repetition() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command = "position fen Q6K/8/8/8/8/8/7R/1k6 w - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(START_POSITION);
                search_state.parse_position_command(command);
                search_state.reset_for_new_iteration(4, Move::create_null());       
                let score = single_depth_search(&mut search_state, 4); 

                println!("{:?}", score);

                assert!(score.node_count < 3);
                assert_eq!(score.eval, 0);
                
            })
            .unwrap();
        handler.join().unwrap();
    }


    #[test]
    fn test_forced_trifold_repetition_start_with_black() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command = "position fen q6k/8/8/8/8/8/7r/1K6 b - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(START_POSITION);
                search_state.parse_position_command(command);
                search_state.reset_for_new_iteration(4, Move::create_null());       
                let score = single_depth_search(&mut search_state, 4); 

                println!("{:?}", score);

                assert!(score.node_count < 3);
                assert_eq!(score.eval, 0);
                
            })
            .unwrap();
        handler.join().unwrap();
    }


    #[test]
    fn test_forced_trifold_repetition_switched_sides() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command = "position fen q6k/8/8/8/8/8/7r/2K5 w - - 0 1 moves c1b1 a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(START_POSITION);
                search_state.parse_position_command(command);
                search_state.reset_for_new_iteration(4, Move::create_null());       
                let score = single_depth_search(&mut search_state, 4); 

                println!("{:?}", score);

                assert!(score.node_count < 3);
                assert_eq!(score.eval, 0);
                
            })
            .unwrap();
        handler.join().unwrap();
    }
}