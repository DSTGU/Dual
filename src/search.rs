use std::{vec};
use coarsetime::{Instant};

use crate::evaluation::evaluate::{nnue_evaluate};
use crate::movegen::move_gen::{is_square_attacked};
use crate::movepicker::MovePicker;
use crate::primitives::board::{BoardPosition};
use crate::primitives::consts::{DRAW_SCORE, MATE_SCORE, MATE_THRESHOLD, MIN_DEPTH, NO_SCORE};
use crate::primitives::shared::Color::White;
use crate::primitives::shared::{Move, Piece, move_to_alg};
use crate::search_objs::see::{see_a_move_threshold};
use crate::search_objs::tt::{TTFlag, score_from_tt};
use crate::search_objs::search_state::{Reporting, SearchState};
use crate::tunable::*;

// value is 1024 * depth
#[allow(clippy::approx_constant)]
pub fn reduce_lmr_by(depth: i32, moves: usize) -> i32 {
    // Obsidian function with tunable base/div (scaled x100: 99=0.99, 314=3.14)
    let base = lmr_base() as f32 / 100.0;
    let div = lmr_div() as f32 / 100.0;
    ((base + (depth as f32).ln() * (moves as f32).ln() / div) * 1024.0) as i32
}

fn lmp_threshold(depth: i32) -> i32 {
    lmp_base() + lmp_scale() * depth * depth
}

pub fn quiescence(board_position: &BoardPosition, search_state: &mut SearchState, alpha: i32, beta: i32, ply: usize) -> i32 {

    search_state.seldepth = search_state.seldepth.max(ply as i32);
    search_state.nodes += 1;

    if search_state.has_occured_in_search(board_position.hash) || search_state.is_trifold_repetition(board_position.hash) || board_position.fifty_mr >= 100 {
        return DRAW_SCORE;
    }

    // // ------------------------------------------------------------
    // // QS TT probe
    // // ------------------------------------------------------------
    let probe = search_state.probe_tt(board_position.hash);
    let tt_move = if let Some(entry) = probe {
        entry.best_move
    } else {
        Move::create_null()
    };
    
    if let Some(entry) = probe {
        let score = score_from_tt(entry.score, search_state.ply);
        match entry.flag {
            TTFlag::Exact => {
                return score;
            }

            TTFlag::Alpha => {
                if score <= alpha {
                    return score;
                }
            }

            TTFlag::Beta => {
                if score >= beta {
                    return score;
                }
            }
        }
    }

    //PESTO eval
    let static_eval = if let Some(entry) = probe { entry.eval } else { nnue_evaluate(&board_position, search_state)};

    if static_eval >= beta
    {
        return beta;
    }

    let mut new_alpha = alpha;

    if static_eval > alpha
    {
        new_alpha = static_eval;
    }

    let mut move_picker = MovePicker::new(tt_move);

    while let Some((mv, new_board)) = move_picker.next(board_position, search_state, true) {

        // Static Exchange Evaluation Pruning (SEE Pruning)
        if !see_a_move_threshold(board_position, mv, &new_board, qs_see_threshold()) {
            continue;
        }

        search_state.make_move(mv, board_position, static_eval);
        
            let res = quiescence(&new_board, search_state, -beta, -new_alpha, ply + 1);
            search_state.take_back();

            if -res >= beta {

                // search_state.store_tt(
                //     0,
                //     new_alpha,
                //     flag,
                //     best_move.unwrap_or(Move::create_null()),
                //     board_position.hash
                // );

                return beta;
            }

            if -res > new_alpha {
                new_alpha = -res;
            }
        }

    new_alpha
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

pub fn pvs<NODE: NodeType>(board_position: &BoardPosition, search_state: &mut SearchState, alpha: i32, beta: i32, mut depth: i32) -> i32 {
    
    if NODE::PV {
        search_state.pv_table.clear(search_state.ply as usize);
    }

    if search_state.has_occured_in_search(board_position.hash) || search_state.is_trifold_repetition(board_position.hash) || board_position.fifty_mr >= 100 {
        search_state.nodes += 1;
        return DRAW_SCORE;
    }
    
    if depth <= 0 {
        return quiescence(board_position, search_state, alpha, beta, search_state.ply);
    }

    search_state.nodes += 1;

    if search_state.stop_condition.should_hard_quit(search_state.nodes) {
        return 0;  
    }


    let mut new_alpha = alpha;

    // ------------------------------------------------------------
    // TT probe
    // ------------------------------------------------------------
    let probe = search_state.probe_tt(board_position.hash);
    let tt_move = if let Some(entry) = probe {
        entry.best_move
    } else {
        Move::create_null()
    };
    
    if let Some(entry) = probe {
        if !NODE::ROOT && entry.depth as i32 >= depth {
            let score = score_from_tt(entry.score, search_state.ply);
            match entry.flag {

                TTFlag::Exact => {
                    return score;
                }

                TTFlag::Alpha => {
                    if score <= alpha {
                        return score;
                    }
                }

                TTFlag::Beta => {
                    if score >= beta {
                        return score;
                    }
                }
            }
        }
    } else {
        // IIR
        if depth >= iir_depth() {
            depth = depth - 1;
        }
    }

    // ------------------------------------------------------------
    // Static eval
    // ------------------------------------------------------------

    //Todo: move to movegen
    let our_king = if board_position.side == White { Piece::K } else {Piece::k};
    let is_in_check = is_square_attacked(board_position.bitboards[our_king as usize].trailing_zeros() as u8, &board_position);
    let static_eval = nnue_evaluate(board_position, search_state);


    // Improving is a very important modifier to many heuristics. It checks if our static eval has improved since our last move.
    // As we don't evaluate in check, we look for the first ply we weren't in check between 2 and 4 plies ago. If we find that
    // static eval has improved, or that we were in check both 2 and 4 plies ago, we set improving to true.
    let improving = if !is_in_check && search_state.move_stack.is_improving(static_eval) {true} else { false }; 

    // ------------------------------------------------------------
    // Reverse Futility Pruning (beta pruning)
    //
    // "Position is so good that even after margin reduction
    //  we still exceed beta."
    // ------------------------------------------------------------
    if !NODE::PV
       && depth <= rfp_max_depth()
       && !is_in_check {

        let d = depth as i32;
        let rfp_margin = static_eval - (rfp_a() * d * d + rfp_b() * d + rfp_c() - improving as i32 * rfp_improving());
        
        if rfp_margin >= beta {
            return static_eval;
        }
    }
    
    // ------------------------------------------------------------
    // Razoring
    // ------------------------------------------------------------
    // sf: alpha - 512 - (293 * depth * depth) as i32
    if !NODE::PV && static_eval < alpha - (razor_a() * depth as i32 * depth as i32 + razor_b() * depth as i32 + razor_c()) { // likely a fail-low node ?
        let new_score = quiescence(board_position, search_state, alpha, beta, search_state.ply + 1);
        if new_score < beta {
            return new_score; // fail soft
        }
    }


    // ------------------------------------------------------------
    // Null Move Pruning 
    // ------------------------------------------------------------
        if 
        board_position.has_pieces() &&
        static_eval > beta &&
        !is_in_check &&
        depth >= nmp_min_depth() &&
        !NODE::PV 
        {
            let r = nmp_base() + depth / nmp_divisor(); // NMP Reduction
            let null_board: BoardPosition = board_position.make_null_move();
            let new_depth = depth.saturating_sub(r+1);

            let search_answer = -pvs::<NonPV>(&null_board, search_state, -beta, -(beta - 1), new_depth);

            if search_answer >= beta {
                return search_answer;
            }
        }

    // Move, eval (alpha), nodes
    let mut best_score = i32::MIN;
    let mut best_move = None;

    let mut legal_moves = 0;
    let mut previous_quiet_moves = vec![]; // malus purposes
    let hist_base = hist_bonus_scale() * depth as i32 + hist_bonus_offset();
    // separate float multipliers (x100): 100 = 1.0
    let hist_beta_bonus = hist_base * hist_beta_mult() / 100;
    let hist_alpha_bonus = hist_base * hist_alpha_mult() / 100;
    let hist_malus_bonus = hist_base * hist_malus_mult() / 100;
    

    let mut move_picker = MovePicker::new(tt_move);

    while let Some((mv, new_board)) = move_picker.next(board_position, search_state, false) {
        // --------------------------------------------------------
        // Futility pruning
        //
        // "Quiet move cannot raise alpha enough."
        // --------------------------------------------------------
        
        if !NODE::PV && 
        depth <= fp_max_depth() &&
        legal_moves > 1 &&
        mv.is_quiet() &&
        !is_in_check {
            let d = depth as i32;
            if static_eval + (fp_a() * d * d + fp_b() * d + fp_c()) <= alpha {
                continue;
            }
        }

        // --------------------------------------------------------
        // Late move pruning
        // --------------------------------------------------------
        if !NODE::PV 
            && depth <= lmp_max_depth()
            && new_alpha.abs() <= MATE_THRESHOLD
            && mv.is_quiet()
            && previous_quiet_moves.len() as i32
                >= lmp_threshold(depth)
        {
            move_picker.skip_quiets();
            continue;
        }

        // Static Exchange Evaluation Pruning (SEE Pruning) — quadratic
        if !NODE::ROOT && !is_in_check {
            let d = depth as i32;
            let threshold= see_a() * d * d + see_b() * d + see_c();
            // Try out a history term
            // let threshold: i32 = if mv.is_quiet() {
            //     (-12 * depth as i32 * depth as i32 + 56 * depth as i32 + 27).min(0)
            // } else {
            //     (-7 * depth as i32 * depth as i32 - 36 * depth as i32 + 14).min(0)
            // };

            if !see_a_move_threshold(board_position, mv, &new_board, threshold) {
                continue;
            }
        }
        
        let mut score= MATE_SCORE;

        search_state.make_move(mv, board_position, if is_in_check {NO_SCORE} else {static_eval});

        legal_moves += 1;

        // --------------------------------------------------------
        // LMR (Late Move Reductions)
        // --------------------------------------------------------
        if depth >= lmr_min_depth() &&
           legal_moves > lmr_min_moves() as usize &&
           mv.is_quiet() {
           // !NODE::PV {
           //and not inCheck
           //and not givesCheck:

            let mut reduction = reduce_lmr_by(depth, legal_moves);

            reduction -= search_state.get_quiet_history(board_position.side, mv) as i32 / lmr_hist_div();

            let reduction = (reduction / 1024).max(0);
            let new_depth = depth - 1 - reduction;

            score = -pvs::<NonPV>( &new_board, search_state, -new_alpha - 1 , -new_alpha, new_depth);

            if score > new_alpha && reduction > 0 {
                score = -pvs::<NonPV>( &new_board, search_state, -new_alpha - 1 , -new_alpha , depth - 1);
            }

        }
        // Fulldepth
        else if !NODE::PV || legal_moves >= 2 {
            score = -pvs::<NonPV>( &new_board, search_state, -new_alpha - 1 , -new_alpha , depth-1 );
        }
        // PVS
        if NODE::PV && ( legal_moves == 1 || score > new_alpha) {
            score = -pvs::<PV>( &new_board, search_state, -beta , -new_alpha , depth-1 );
        }

        search_state.take_back();

        if score > best_score {
            best_score = score;
            if score > new_alpha {

                if NODE::PV {
                    search_state.pv_table.update(search_state.ply, mv);
                }

                if score >= beta {
                    
                    search_state.store_tt(
                        depth as u8,
                        score,
                        static_eval,
                        TTFlag::Beta,
                        mv,
                        board_position.hash
                    );
                    
                    if mv.is_quiet() {
                        search_state.update_killer_move(mv);
                        search_state.update_history(board_position, mv, hist_beta_bonus);
                        
                        // apply malus to previous quiet moves
                        for prev_mv in &previous_quiet_moves {
                            search_state.update_history(
                                board_position,
                                *prev_mv,
                                -hist_malus_bonus,
                            );
                        }
                    }
                    
                    return score;
                }
                
                new_alpha = score;
                best_move = Some(mv);
            }
        }
            
        if mv.is_quiet() {
            previous_quiet_moves.push(mv);
        }
    }

    if legal_moves == 0 {
        if board_position.is_king_attacked() {
            return -MATE_SCORE + search_state.ply as i32;
        }
        else {
            return DRAW_SCORE;
        }
    }

    if let Some(mv) = best_move {
        if mv.is_quiet() {
            search_state.update_history(board_position, best_move.unwrap(), hist_alpha_bonus);
        }
    }

    let flag: TTFlag = if best_score <= alpha {
        TTFlag::Alpha
    } else if best_score >= beta {
        TTFlag::Beta
    } else {
        TTFlag::Exact
    };

    search_state.store_tt(
        depth as u8,
        best_score,
        static_eval,
        flag,
        best_move.unwrap_or(Move::create_null()),
        board_position.hash
    );

    best_score
}

pub fn score_to_mate( score: i32 ) -> i32 {
    let distance = MATE_SCORE - score.abs();
    if score > 0 {
        return (distance + 1) / 2
    }
    - distance / 2
}

pub fn collect_pv(moves: &[Move]) -> String {
    moves
        .iter()
        .filter(|&&mv| mv != Move::create_null())
        .map(|x| move_to_alg(x))
        .reduce(|a, b| a + " " + &b)
        .unwrap_or_default()
}

pub fn single_depth_search(board_position: &BoardPosition, search_state: &mut SearchState, depth: i32) -> i32 {
    pvs::<Root>(board_position, search_state, -MATE_SCORE, MATE_SCORE, depth)
}

pub fn single_depth_search_aspirated(board_position: &BoardPosition, search_state: &mut SearchState, depth: i32, eval: i32) -> i32 {
    let mut aspiration_lower = asp_delta();
    let mut aspiration_higher = asp_delta();

    let mut score ;
    //println!(" ---------------- NEW SEARCH, depth: {} ----------------", depth);
    for _ in 0..asp_max_tries() {
        //println!("low: {}, high: {}", eval-aspiration_lower, eval+aspiration_higher);
        score = pvs::<Root>(board_position, search_state, eval-aspiration_lower, eval+aspiration_higher, depth);
        //println!("aspiration, score: {:?}", score.eval);

        //println!("stage: {:?}", search_state.search_stage);
        
        if score < eval+aspiration_higher && score > eval-aspiration_lower { // stopped ahead of time
            return score;
        }

        //println!("aspiration failed, score: {:?}", score.eval);
        if score < eval {
            aspiration_lower *= asp_mult();
        }
        else {
            aspiration_higher *= asp_mult();
        }
    }

    //fallback
    single_depth_search(board_position, search_state, depth)
}


pub fn search(board_position: &BoardPosition, search_state: &mut SearchState) {

    search_state.stop_condition.started_search = Instant::now();

    search_state.reset_for_new_iteration(MIN_DEPTH);

    let mut score = single_depth_search(board_position, search_state, MIN_DEPTH);
        
    print_info_string(score, search_state);
        
    let mut depth = MIN_DEPTH;
    let mut bestmove = search_state.pv_table.table[0][0];
    search_state.reset_for_new_iteration(depth);        

    while !search_state.stop_condition.should_soft_quit(depth, search_state.nodes) && !search_state.stop_condition.should_hard_quit(search_state.nodes) {
        depth += 1;
        search_state.reset_for_new_iteration(depth);        
        
        let new_score = single_depth_search_aspirated(board_position, search_state, depth, score);

        //if search_state.search_stage == Full {
        if !search_state.stop_condition.should_hard_quit(search_state.nodes) {
            score = new_score;
            print_info_string(score, search_state);
            bestmove = search_state.pv_table.table[0][0];
        }
    }

    if search_state.reporting != Reporting::Quiet {
        println!("bestmove {}", move_to_alg(&bestmove));
    }

    // search_state.print_history_stats();
    
}

pub fn print_info_string(score: i32, search_state: &SearchState) {
    if search_state.reporting == Reporting::Quiet {
        return;
    }
    
    let len = search_state.pv_table.len[0];
    let pv: String = collect_pv(&search_state.pv_table.table[0][..len]);

    let micros = if search_state.stop_condition.started_search.elapsed().as_micros() > 0 {search_state.stop_condition.started_search.elapsed().as_micros()} else {1};

    if score.abs() > MATE_THRESHOLD {
        let mate = score_to_mate( score );
        println!("info score mate {} depth {} seldepth {} nodes {} time {} nps {} pv {}", mate, search_state.max_depth, 
            search_state.seldepth, search_state.nodes, micros/1000, search_state.nodes * 1000000 / micros, pv);
    }
    else {
        println!("info score cp {} depth {} seldepth {} nodes {} time {} nps {} pv {}", score, search_state.max_depth, 
            search_state.seldepth, search_state.nodes, micros/1000, search_state.nodes * 1000000 / micros, pv);
    }
}


#[cfg(test)]
mod tests {
    use crate::gui::parse_position_command;
    use crate::search::{search, single_depth_search};
    use crate::search_objs::config::EngineConfig;
use crate::search_objs::search_state::SearchState;


    #[test]
    fn test_forced_trifold_repetition() {
        let command = "position fen Q6K/8/8/8/8/8/7R/1k6 w - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(&EngineConfig::thin());
                
                let board_position = parse_position_command(&mut search_state, command);
                search_state.reset_for_new_iteration(4);       
                let score = single_depth_search(&board_position, &mut search_state, 4); 

                println!("{:?}", score);

                assert!(search_state.nodes < 3);
                assert_eq!(score, 0);
    }


    #[test]
    fn test_forced_trifold_repetition_start_with_black() {
        let command = "position fen q6k/8/8/8/8/8/7r/1K6 b - - 0 1 moves a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(&EngineConfig::thin());
                
                println!("{:?}", search_state.move_stack);
                
                let board_position = parse_position_command(&mut search_state, command);

                println!("{:?}", search_state.move_stack);
                
                search_state.reset_for_new_iteration(3);       
                
                println!("{:?}", search_state.move_stack);
                println!("{:?}", board_position.hash);

                let score = single_depth_search(&board_position, &mut search_state, 3);

                println!("{:?}", search_state.move_stack);

                println!("{:?}", score);

                assert!(search_state.nodes < 3);
                assert_eq!(score, 0);
    }


    #[test]
    fn test_forced_trifold_repetition_switched_sides() {
        let command = "position fen q6k/8/8/8/8/8/7r/2K5 w - - 0 1 moves c1b1 a8b8 b1a1 b8a8 a1b1 a8b8 b1a1 b8a8";
                let mut search_state = SearchState::new(&EngineConfig::thin());
                let board_position = parse_position_command(&mut search_state, command);
                search_state.reset_for_new_iteration(4);       
                let score = single_depth_search(&board_position, &mut search_state, 4);

                println!("{:?}", score);

                assert!(search_state.nodes < 3);
                assert_eq!(score, 0);
    }

    #[test]
    fn test_mate_normalisation() {
        let command1 = "position fen 8/7p/P1N2k2/1BBp2p1/4b1K1/6P1/r7/8 b - - 1 49";
                let mut search_state = SearchState::new(&EngineConfig::thin());
                let board_position = parse_position_command(&mut search_state, command1);
                search_state.stop_condition.depth = Some(12);
                search(&board_position, &mut search_state); 
                let command2 = "position fen 8/7p/P1N2k2/1BBp2p1/4b1K1/6P1/r7/8 b - - 1 49 moves h7h5 g4h5";
                let board_position = parse_position_command(&mut search_state, command2);
                search_state.stop_condition.depth = Some(5);
                search(&board_position, &mut search_state);
    }
}