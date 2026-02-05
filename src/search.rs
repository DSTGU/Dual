use std::thread;
use std::time::SystemTime;
use crate::evaluate::evaluate;
use crate::move_gen::{generate_moves, is_square_attacked, make_move};
use crate::shared::{get_bit, move_to_alg, BoardPosition, Move};

const MVV_LVA : [usize ; 36] = [
105000000, 205000000, 305000000, 405000000, 505000000, 605000000,
104000000, 204000000, 304000000, 404000000, 504000000, 604000000,
103000000, 203000000, 303000000, 403000000, 503000000, 603000000,
102000000, 202000000, 302000000, 402000000, 502000000, 602000000,
101000000, 201000000, 301000000, 401000000, 501000000, 601000000,
100000000, 200000000, 300000000, 400000000, 500000000, 600000000,
];

static mut MAX_DEPTH : usize = 0;
static mut KILLER_MOVE : [[u32; 256]; 2 ] = [[0; 256]; 2];
static mut HISTORY_MOVE : [[usize; 64]; 12 ] = [[0; 64]; 12];
static mut PREVITER_BESTMOVE : u32 = 0;

#[allow(non_snake_case)]
pub fn get_MVV_LVA(victim: usize, attacker: usize) -> usize {
    MVV_LVA[victim % 6 + attacker % 6 * 6]
}

pub fn get_victim(board_position: &BoardPosition, mv: &Move) -> usize {
    let sidevar = ((board_position.side + 1) % 2) * 6;

    for i in 0+sidevar..6+sidevar {
        if get_bit(board_position.bitboards[i], mv.get_target_square() as usize) {
            return i;
        }
    }

    0
}
pub fn get_move_score(board_position: &BoardPosition, mv: &Move, ply: usize) -> usize {

    unsafe {
        if ply == 0 && mv.mv == PREVITER_BESTMOVE {
            return 605000001;
        }
    }

    //println!("ply: {}", ply);

    if mv.get_capture() == true {
        let victim = get_victim(board_position, mv);
        return get_MVV_LVA(victim, mv.get_piece() as usize);
    }
    else {
        unsafe {
            if KILLER_MOVE[0][ply] == mv.mv {
                return 9000000;
            }
            if KILLER_MOVE[1][ply] == mv.mv {
                return 8000000;
            }
                return HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize];
        }
    }
}

// pub fn rand_search(board_position: &BoardPosition) {

//     let mut moves = generate_moves(board_position);
    
//     let mut mv = moves.pop();
    
//     while mv.is_none() {
//         mv = moves.pop();
//     }
    
//     println!("bestmove {}", move_to_alg(&mv.unwrap()))
// }


pub fn quiescence(board_position: &BoardPosition, alpha: i32, beta: i32, ply: usize) -> (i32, i32) {

    let eval = evaluate(board_position);

    if eval >= beta
    {
        return (beta,0);
    }

    let mut new_alpha = alpha;

    if eval > alpha
    {
        new_alpha = eval;
    }

    let move_list = generate_moves(&board_position);
    let mut filtered_move_list : Vec<Move> = move_list.into_iter().filter(|mv| mv.get_capture() == true).collect();
    filtered_move_list.sort_by(|a, b| {
        unsafe {
            let score_a = get_move_score(board_position, a, MAX_DEPTH + ply);
            let score_b = get_move_score(board_position, b, MAX_DEPTH + ply);
            score_b.cmp(&score_a)
        }
    });

    let mut nodes = 1;

    for mv in filtered_move_list {
        let nbp_option = make_move(&board_position, &mv);

        if let Some(nbp) = nbp_option {
            let res = quiescence(&nbp, -beta, -new_alpha, ply + 1);
            nodes += res.1;

            if -res.0 >= beta {
                return (beta, nodes);
            }

            if -res.0 > new_alpha {
                new_alpha = -res.0;
            }
        }
    }


    (new_alpha, nodes)
}

pub fn negamax(board_position: &BoardPosition, alpha: i32, beta: i32, depth: usize) -> (Vec<Option<Move>>, i32, i32) {

    if depth == 0 {
        let score = quiescence(board_position, alpha, beta, 1);
        return (vec![], score.0, score.1)
    }

    let mut new_alpha = alpha;

    let mut move_list = generate_moves(&board_position);
    move_list.sort_by(|a, b| {
        unsafe {
        let score_a = get_move_score(board_position, a, MAX_DEPTH - depth);
        let score_b = get_move_score(board_position, b, MAX_DEPTH - depth);
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

        let nbp_option = make_move(&board_position, mv);

        if let Some(nbp) = nbp_option {
            legal_moves += 1;
            let _newdepth = depth-1;
            let res;
            // if idx > 3 && depth >= 4 {
            //     newdepth = depth - 2;
            // }
            
             // if is_PV_node {
             //     res = negamax(&nbp, -new_alpha - 1, -new_alpha, newdepth);
             //     nodes += res.2;
             //
             //     if -res.1 > new_alpha && -res.1 < beta {
             //         //println!("failure");
             //         res = negamax(&nbp, -beta, -new_alpha, depth-1);
             //         nodes += res.2;
             //     }
             // }
             // else {
                res = negamax(&nbp, -beta, -new_alpha, depth-1);
                 nodes += res.2;
             // }
            
            // unsafe {
            //     println!("|{}|, {} - {:?}", max_depth - depth, is_PV_node, res);
            // }
            



            if -res.1 >= beta {
                
                if mv.get_capture() == false {
                    unsafe {
                        KILLER_MOVE[1][MAX_DEPTH - depth] = KILLER_MOVE[0][MAX_DEPTH - depth];
                        KILLER_MOVE[0][MAX_DEPTH - depth] = mv.mv;
                    }
                }
                return (vec![], beta, nodes);
            }

            if -res.1 > new_alpha {
                
                if mv.get_capture() == false {
                    unsafe {
                        HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize] += depth;
                        //println!( "{}, {} - {} -> {}", depth, mv.get_piece(), mv.get_target_square(), HISTORY_MOVE[mv.get_piece() as usize][mv.get_target_square() as usize])
                    }
                }
                
                new_alpha = -res.1;
                best_move = Some(*mv);
                best_move_list = res.0;
                is_PV_node = true;
            }
        }
    }
    
    if legal_moves == 0 {
            if is_square_attacked(board_position.bitboards[6*board_position.side+5].trailing_zeros() as usize, board_position) {
                return (vec![], -4999900 - depth as i32, 1)
            }
            else {
                return (vec![], 0, 1)
            }
    }

    best_move_list.push(best_move);
    (best_move_list, new_alpha, nodes)
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

pub fn single_depth_search(board_position: &BoardPosition, depth: usize) -> (Vec<Option<Move>>, i32, i32){
    negamax(&board_position, -5000000, 5000000, depth)
}

pub fn single_depth_search_aspirated(board_position: &BoardPosition, depth: usize, eval: i32) -> (Vec<Option<Move>>, i32, i32){
    let mut aspiration_lower = 50;
    let mut aspiration_higher = 50;

    let mut score ;

    loop {
        println!("low: -{}, high: {}", aspiration_lower, aspiration_higher);
        score = negamax(&board_position, eval-aspiration_lower, eval+aspiration_higher, depth);

        //println!("aspiration, score: {:?}", score);

        if score.0.len() > 0 {
            if score.0[0].is_some() {
                //println!("returning: {:?}", score);
                return score;
            }
        }

        //println!("aspiration failed, score: {:?}", score);
        if score.1 < eval {
            aspiration_lower = aspiration_lower * 2;
        }
        else {
            aspiration_higher = aspiration_higher * 2;
        }
    }
}


pub fn search(board_position: &BoardPosition, depth: Option<usize>, time: Option<usize>) {

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
        let bp = board_position.clone();
        let handler = builder.spawn(move || {
            let mut score = single_depth_search(&bp, depth.unwrap());

            let pv = collect_pv(&score.0);

            if score.1 > 4000000 || score.1 < -4000000 {

                let mate = score_to_mate( score.1, depth.unwrap());

                println!("info score mate {} depth {} nodes {} pv {}", mate, depth.unwrap(), score.2, pv);
            }
            else {
                println!("info score cp {} depth {} nodes {} pv {}", score.1, depth.unwrap(), score.2, pv);
            }
           //println!("Movelist: {:?}", score.0);
            println!("bestmove {}", move_to_alg(&score.0.pop().unwrap().unwrap()));
            
        }).unwrap();
        handler.join().unwrap();
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
        let bp = board_position.clone();
        let mut score = single_depth_search(board_position, 3);
        depth = 4;
        let handler = builder.spawn(move || {
            while now.elapsed().unwrap().as_millis() < time_avail as u128 {
                unsafe {
                    MAX_DEPTH = depth;
                    KILLER_MOVE = [[0; 256]; 2];
                    HISTORY_MOVE = [[0; 64]; 12];
                    //previter_bestmove = score.0.pop().unwrap().unwrap().mv;
                }
                
                score = single_depth_search_aspirated(&bp, depth, score.1);

                let pv = collect_pv(&score.0);

                if score.1 > 4000000 || score.1 < -4000000 {

                    let mate = score_to_mate( score.1, depth);

                    println!("info score mate {} depth {} nodes {} pv {}", mate, depth, score.2, pv);
                }
                else {
                    println!("info score cp {} depth {} nodes {} pv {}", score.1, depth, score.2, pv);
                }
                
                depth = depth + 1;
            }


            println!("bestmove {}", move_to_alg(&score.0.pop().unwrap().unwrap()));
        }).unwrap();
        handler.join().unwrap();
        
    }
    
}



