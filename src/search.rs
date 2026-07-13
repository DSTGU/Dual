use std::{vec};
use coarsetime::{Instant};

use crate::evaluate::{nnue_evaluate};
use crate::move_gen::{generate_moves, is_square_attacked};
use crate::types::board::{BoardPosition};
use crate::types::consts::{DRAW_SCORE, MATE_SCORE, MATE_THRESHOLD, MIN_DEPTH};
use crate::types::search_state::SearchState;
use crate::types::shared::Color::White;
use crate::types::shared::{Move, Piece, SearchAnswer, move_to_alg};
use crate::types::tt::{TTFlag, score_from_tt};

pub fn sort_move_list(board_position : &BoardPosition, search_state: &mut SearchState, move_list: Vec<Move>) -> Vec<Move> {
    let mut scored_moves: Vec<(Move, i32)> = move_list
        .into_iter()
        .map(|m| {
            let score = if Some(m) == search_state.tt_move(board_position.hash) {
                i32::MAX
            } else {
                search_state.get_move_score(board_position, m)
            };

            (m, score)
        })
        .collect();

    scored_moves.sort_unstable_by_key(|&(_, score)| -score);
    scored_moves.into_iter().map(|(mv, _)| mv).collect()
}

#[allow(clippy::approx_constant)]
pub fn reduce_lmr_by(depth: usize, moves: usize) -> usize {
    // Obsidian function
    (0.99 + (depth as f32).ln() * (moves as f32).ln() / 3.14) as usize
}

pub fn quiescence(board_position: &BoardPosition, search_state: &mut SearchState, alpha: i32, beta: i32, ply: usize) -> i32 {

    search_state.seldepth = search_state.seldepth.max(ply);

    if search_state.is_trifold_repetition(board_position.hash) || board_position.fifty_mr >= 100 {
        search_state.nodes += 1;
        return DRAW_SCORE;
    }

    // // ------------------------------------------------------------
    // // QS TT probe
    // // ------------------------------------------------------------
    // let probe = search_state.probe_tt(board_position.hash);
    
    // if let Some(entry) = probe {

    //     if !search_state.is_twofold_repetition(board_position.hash) {
    //         let score = score_from_tt(entry.score, search_state.ply);
    //         match entry.flag {

    //             TTFlag::Exact => {
    //                 return SearchAnswer {
    //                     move_list: vec![Some(entry.best_move)],
    //                     node_count: 1,
    //                     eval: score,
    //                 };
    //             }

    //             TTFlag::Alpha => {
    //                 if score <= alpha {
    //                     return SearchAnswer {
    //                         move_list: vec![],
    //                         node_count: 1,
    //                         eval: score,
    //                     };
    //                 }
    //             }

    //             TTFlag::Beta => {
    //                 if score >= beta {
    //                     return SearchAnswer {
    //                         move_list: vec![Some(entry.best_move)],
    //                         node_count: 1,
    //                         eval: score,
    //                     };
    //                 }
    //             }
    //         }
    //     }
    // }

    //PESTO eval
    let eval = nnue_evaluate(&board_position);

    if eval >= beta
    {
        search_state.nodes += 1;
        return beta;
        //return SearchAnswer { move_list: vec![], node_count: 0, eval: beta };
    }

    let mut new_alpha = alpha;

    if eval > alpha
    {
        new_alpha = eval;
    }

    let move_list = generate_moves(&board_position, true);
    let filtered_move_list = sort_move_list(board_position, search_state, move_list);

    for mv in filtered_move_list {

        // let captured_value = DELTA_VALUES[mv.get_taken_piece() as usize % 6];
        // // Delta pruning
        // if eval + captured_value + DELTA_PRUNING_MARGIN < new_alpha {
        //     continue;
        // }

        let new_board = board_position.make_move(mv);

        if new_board.is_none() {
            continue;
        }

        let new_board = new_board.unwrap();

        search_state.make_move(board_position.hash);
        
            let res = quiescence(&new_board, search_state, -beta, -new_alpha, ply + 1);
            search_state.take_back();

            if -res >= beta {
                search_state.nodes += 1;
                return beta;
            }

            if -res > new_alpha {
                new_alpha = -res;
            }
        }

    search_state.nodes += 1;
    new_alpha
    //SearchAnswer { move_list: vec![], node_count: nodes, eval: new_alpha }

}

pub trait NodeType {
    const PV: bool;
    const ROOT: bool;
}

struct Root;
impl NodeType for Root {
    const PV: bool = true;
    const ROOT: bool = true;
}

struct PV;
impl NodeType for PV {
    const PV: bool = true;
    const ROOT: bool = false;
}

struct NonPV;
impl NodeType for NonPV {
    const PV: bool = false;
    const ROOT: bool = false;
}

pub fn pvs<NODE: NodeType>(board_position: &BoardPosition, search_state: &mut SearchState, alpha: i32, beta: i32, depth: usize) -> SearchAnswer {
    
    let is_pv_node_deprecated = beta - alpha > 1;

    if search_state.stop_condition.should_hard_quit(1) {
       return SearchAnswer { move_list: vec![], node_count: 1, eval: 0};  
    }

    if search_state.is_trifold_repetition(board_position.hash) || board_position.fifty_mr >= 100 {
        return SearchAnswer { move_list: vec![], node_count: 1, eval: DRAW_SCORE };
    }
    
    if depth == 0 {
        return SearchAnswer { move_list: vec![], node_count: 1, eval: quiescence(board_position, search_state, alpha, beta, search_state.ply) };
    }

    let mut new_alpha = alpha;

    // ------------------------------------------------------------
    // TT probe
    // ------------------------------------------------------------
    let probe = search_state.probe_tt(board_position.hash);
    
    if let Some(entry) = probe {

        if !NODE::ROOT && entry.depth as usize >= depth && !search_state.is_twofold_repetition(board_position.hash) {
            let score = score_from_tt(entry.score, search_state.ply);
            match entry.flag {

                TTFlag::Exact => {
                    return SearchAnswer {
                        move_list: vec![Some(entry.best_move)],
                        node_count: 1,
                        eval: score,
                    };
                }

                TTFlag::Alpha => {
                    if score <= alpha {
                        return SearchAnswer {
                            move_list: vec![],
                            node_count: 1,
                            eval: score,
                        };
                    }
                }

                TTFlag::Beta => {
                    if score >= beta {
                        return SearchAnswer {
                            move_list: vec![Some(entry.best_move)],
                            node_count: 1,
                            eval: score,
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
    let our_king = if board_position.side == White { Piece::K } else {Piece::k};
    let is_in_check = is_square_attacked(board_position.bitboards[our_king as usize].trailing_zeros() as u8, &board_position);

    let static_eval =  if is_in_check {
        -MATE_SCORE
    } else if probe.is_some() && probe.unwrap().flag == TTFlag::Exact {
        score_from_tt(probe.unwrap().score, search_state.ply)
    } else {
        nnue_evaluate(board_position)
    };

    // ------------------------------------------------------------
    // Reverse Futility Pruning (beta pruning)
    //
    // "Position is so good that even after margin reduction
    //  we still exceed beta."
    // ------------------------------------------------------------
    if !is_pv_node_deprecated
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
    // Null Move Pruning 
    // ------------------------------------------------------------
        if !is_in_check &&
        board_position.has_pieces() &&
        static_eval > beta &&
        !is_pv_node_deprecated &&
        depth >= 3
        {
            let r = 2 + depth / 4; // NMP Reduction
            let null_board = board_position.make_null_move();
            let search_answer = pvs::<NonPV>(&null_board, search_state, -beta, -(beta - 1), (depth - r - 1).max(0));

            if -search_answer.eval >= beta {
                return SearchAnswer {
                    move_list: vec![],
                    node_count: search_answer.node_count,
                    eval: -search_answer.eval,
                };
                //return search_answer;
            }
        }


    // ------------------------------------------------------------
    // Move generation / ordering
    // ------------------------------------------------------------
    let move_list = generate_moves(board_position, false);
    let move_list = sort_move_list(board_position, search_state, move_list);

    // Move, eval (alpha), nodes
    let mut nodes = 1;

    let mut best_move = None;
    let mut best_move_list = vec![];

    let mut legal_moves = 0;
    let mut previous_quiet_moves = vec![]; // malus purposes
    let history_bonus = 300 * depth as i32 - 250;
    
    for &mv in move_list.iter() {
        // --------------------------------------------------------
        // Futility pruning
        //
        // "Quiet move cannot raise alpha enough."
        // --------------------------------------------------------

        if !is_pv_node_deprecated && 
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
           !is_pv_node_deprecated {
           //and not inCheck
           //and not givesCheck:

            reduction = reduce_lmr_by(depth, legal_moves);
                            //improving);

            // Often reduce less for good-history moves
            //reduction -= historyBonus(move)

            reduction = reduction.clamp(0, depth - 1);
        }


        let new_board = board_position.make_move(mv);
        
        if new_board.is_none() {
            continue;  
        }
        
        let new_board = new_board.unwrap();

        search_state.make_move(board_position.hash);

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
                let score: SearchAnswer = pvs::<PV>(&new_board, search_state, -beta, -new_alpha, depth-1);
                search_state.take_back();
                nodes += score.node_count;

                if -score.eval > new_alpha {
                    if -score.eval >= beta {
                        if search_state.stop_condition.should_hard_quit(nodes as u64) {
                            return SearchAnswer { move_list: vec![], node_count: nodes, eval: 0};
                        }
                        
                        if mv.is_quiet() {
                            search_state.update_killer_move(mv);
                            search_state.update_history(board_position, mv, history_bonus);

                            // apply malus to previous quiet moves
                            for prev_mv in &previous_quiet_moves {
                                search_state.update_history(
                                    board_position,
                                    *prev_mv,
                                    -history_bonus,
                                );
                            }
                        }

                        search_state.store_tt(
                            depth as u8,
                            -score.eval,
                            TTFlag::Beta,
                            mv,
                            board_position.hash,
                        );

                        return SearchAnswer { move_list: vec![], node_count: nodes, eval: -score.eval };
                    }

                    new_alpha = -score.eval;
                    best_move = Some(mv);
                    best_move_list = score.move_list;
                }

                if mv.is_quiet() {
                    previous_quiet_moves.push(mv);
                }

            } else {
                // ----------------------------------------------------
                // First try:
                // reduced + null-window
                // ----------------------------------------------------

                let mut score = pvs::<NonPV>(&new_board, search_state, -new_alpha-1, -new_alpha, depth-1-reduction);
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
                    score = pvs::<NonPV>(&new_board, search_state, -new_alpha-1, -new_alpha, depth - 1);
                    nodes += score.node_count;
                } 

                // ----------------------------------------------------
                // Case 2:
                // Null-window search indicates possible PV move
                //
                // Need full-window re-search.
                // ----------------------------------------------------
                if -score.eval > new_alpha && -score.eval < beta  {
                    // research with window [alfa;beta]
                    score = pvs::<PV>(&new_board, search_state, -beta, -new_alpha, depth-1);
                    nodes += score.node_count;

                }

                search_state.take_back();

                if -score.eval > new_alpha {
                    if -score.eval >= beta {
                        
                        if search_state.stop_condition.should_hard_quit(nodes as u64) {
                            return SearchAnswer { move_list: vec![], node_count: nodes, eval: 0};
                        }

                        search_state.store_tt(
                            depth as u8,
                            -score.eval,
                            TTFlag::Beta,
                            mv,
                            board_position.hash
                        );

                        if mv.is_quiet() {
                            search_state.update_killer_move(mv);
                            search_state.update_history(board_position, mv, history_bonus);

                            // apply malus to previous quiet moves
                            for prev_mv in &previous_quiet_moves {
                                search_state.update_history(
                                        board_position,
                                    *prev_mv,
                                    -history_bonus,
                                );
                            }
                        }

                        return SearchAnswer { move_list: vec![], node_count: nodes, eval: -score.eval };
                    }

                    new_alpha = -score.eval;
                    best_move = Some(mv);
                    best_move_list = score.move_list;
                }

                if mv.is_quiet() {
                    previous_quiet_moves.push(mv);
                }

            }
    }

    if legal_moves == 0 {
        if board_position.is_king_attacked() {
            return SearchAnswer { move_list: vec![], node_count: 1, eval: -MATE_SCORE + search_state.ply as i32};
        }
        else {
            return SearchAnswer { move_list: vec![], node_count: 1, eval: 0};
        }
    }

    if let Some(mv) = best_move {
        if mv.is_quiet() {
            search_state.update_history(board_position, best_move.unwrap(), history_bonus);
        }
    }

    if search_state.stop_condition.should_hard_quit(nodes as u64) {
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
        depth as u8,
        new_alpha,
        flag,
        best_move.unwrap_or(Move::create_null()),
        board_position.hash
    );

    best_move_list.push(best_move);
    SearchAnswer { move_list: best_move_list, node_count: nodes, eval: new_alpha }
}

pub fn score_to_mate( score: i32 ) -> i32 {
    let distance = MATE_SCORE - score.abs();
    if score > 0 {
        return (distance + 1) / 2
    }
    - distance / 2
}

pub fn collect_pv(moves: &[Option<Move>]) -> String {
    moves
        .iter()
        .filter(|&&mv| mv.is_some() && mv.unwrap() != Move::create_null())
        .filter_map(|x| x.as_ref().map(move_to_alg))
        .rev()
        .reduce(|a, b| a + " " + &b)
        .unwrap_or_default()
}

pub fn single_depth_search(board_position: &BoardPosition, search_state: &mut SearchState, depth: usize) -> SearchAnswer {
    let score = pvs::<Root>(board_position, search_state, -MATE_SCORE, MATE_SCORE, depth);
    search_state.nodes += score.node_count as u64;
    score
}

pub fn single_depth_search_aspirated(board_position: &BoardPosition, search_state: &mut SearchState, depth: usize, eval: i32) -> SearchAnswer {
    let mut aspiration_lower = 50;
    let mut aspiration_higher = 50;

    let mut score ;

    for _ in 0..3 {
        //println!("low: {}, high: {}", eval-aspiration_lower, eval+aspiration_higher);
        score = pvs::<Root>(board_position, search_state, eval-aspiration_lower, eval+aspiration_higher, depth);
        //println!("aspiration, score: {:?}", score.eval);
        search_state.nodes += score.node_count as u64;
        
        if !score.move_list.is_empty() && score.move_list[0].is_some() {
            return score;
        }

        //println!("aspiration failed, score: {:?}", score.eval);
        if score.eval < eval {
            aspiration_lower *= 2;
        }
        else {
            aspiration_higher *= 2;
        }
    }

    //fallback
    single_depth_search(board_position, search_state, depth)
}


pub fn search(board_position: &BoardPosition, search_state: &mut SearchState) {

    search_state.stop_condition.started_search = Instant::now();

    search_state.reset_for_new_iteration(MIN_DEPTH);

    let mut score: SearchAnswer = single_depth_search(board_position, search_state, MIN_DEPTH);
        
    print_info_string(&score, search_state);
        
    let mut depth = MIN_DEPTH;

    search_state.reset_for_new_iteration(depth);        

    while !search_state.stop_condition.should_soft_quit(depth, search_state.nodes) && !search_state.stop_condition.should_hard_quit(0) {
        depth += 1;
        search_state.reset_for_new_iteration(depth);        
        
        let new_score = single_depth_search_aspirated(board_position, search_state, depth, score.eval);

        if !new_score.move_list.is_empty() {
            score = new_score;
            print_info_string(&score, search_state);
        }
    }
    
    println!("bestmove {}", move_to_alg(&score.move_list.pop().unwrap().unwrap()));

    // search_state.print_history_stats();
    
}

pub fn print_info_string(score: &SearchAnswer, search_state: &SearchState) {
    let pv: String = collect_pv(&score.move_list);
    let micros = if search_state.stop_condition.started_search.elapsed().as_micros() > 0 {search_state.stop_condition.started_search.elapsed().as_micros()} else {1};

    if score.eval.abs() > MATE_THRESHOLD {
        let mate = score_to_mate( score.eval );
        println!("info score mate {} depth {} seldepth {} nodes {} time {} nps {} pv {}", mate, search_state.max_depth, 
            search_state.seldepth, search_state.nodes, micros/1000, search_state.nodes * 1000000 / micros, pv);
    }
    else {
        println!("info score cp {} depth {} seldepth {} nodes {} time {} nps {} pv {}", score.eval, search_state.max_depth, 
            search_state.seldepth, search_state.nodes, micros/1000, search_state.nodes * 1000000 / micros, pv);
    }
}


#[cfg(test)]
mod tests {

    use std::thread;
    use crate::gui::parse_position_command;
    use crate::search::{search, single_depth_search};
    use crate::types::search_state::SearchState;


    #[test]
    fn test_forced_trifold_repetition() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command = "position fen Q6K/8/8/8/8/8/7R/1k6 w - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new();
                
                let board_position = parse_position_command(&mut search_state, command);
                search_state.reset_for_new_iteration(4);       
                let score = single_depth_search(&board_position, &mut search_state, 4); 

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
                let mut search_state = SearchState::new();
                
                println!("{:?}", search_state.rep_table);
                
                let board_position = parse_position_command(&mut search_state, command);

                println!("{:?}", search_state.rep_table);
                
                search_state.reset_for_new_iteration(3);       
                
                println!("{:?}", search_state.rep_table);
                println!("{:?}", board_position.hash);

                let score = single_depth_search(&board_position, &mut search_state, 3);

                println!("{:?}", search_state.rep_table);

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
                let mut search_state = SearchState::new();
                let board_position = parse_position_command(&mut search_state, command);
                search_state.reset_for_new_iteration(4);       
                let score = single_depth_search(&board_position, &mut search_state, 4);

                println!("{:?}", score);

                assert!(score.node_count < 3);
                assert_eq!(score.eval, 0);
                
            })
            .unwrap();
        handler.join().unwrap();
    }

    #[test]
    fn test_mate_normalisation() {
        let builder = thread::Builder::new().stack_size(80 * 1024 * 1024);
        let handler = builder
            .spawn(|| {
                let command1 = "position fen 8/7p/P1N2k2/1BBp2p1/4b1K1/6P1/r7/8 b - - 1 49";
                let mut search_state = SearchState::new();
                search_state.stop_condition.depth = Some(8);
                let board_position = parse_position_command(&mut search_state, command1);
                search(&board_position, &mut search_state); 
                let command2 = "position fen 8/7p/P1N2k2/1BBp2p1/4b1K1/6P1/r7/8 b - - 1 49 moves h7h5 g4h5";
                let board_position = parse_position_command(&mut search_state, command2);
                search_state.stop_condition.depth = Some(3);
                search(&board_position, &mut search_state); 
            })
            .unwrap();
        handler.join().unwrap();
    }
}