use crate::evaluate::evaluate;
use crate::moveGen::{generate_moves, is_square_attacked, make_move};
use crate::shared::{coordinates_to_squares, get_bit, move_to_alg, BoardPosition, Move, Piece};
use crate::shared::Piece::{p, K};

const MVV_LVA : [usize ; 36] = [
105, 205, 305, 405, 505, 605,
104, 204, 304, 404, 504, 604,
103, 203, 303, 403, 503, 603,
102, 202, 302, 402, 502, 602,
101, 201, 301, 401, 501, 601,
100, 200, 300, 400, 500, 600,
];

static mut max_depth : usize = 0;
static mut KILLER_MOVES : [[usize; 2]; 128 ] = [[0; 2]; 128];
static mut HISTORY_MOVES : [[usize; 12]; 64 ] = [[0; 12]; 64];

pub fn get_MVV_LVA(victim: usize, attacker: usize) -> usize {
    MVV_LVA[victim % 6 + attacker % 6 * 6]
}

pub fn get_victim(board_position: &BoardPosition, mv: &Move) -> usize {
    let sidevar = ((board_position.side + 1) % 2) * 6;

    for i in 0+sidevar..6+sidevar {
        if get_bit(board_position.bitboards[i], mv.target_square as usize) {
            return i;
        }
    }

    0
}
pub fn get_move_score(board_position: &BoardPosition, mv: &Move) -> usize {
    if mv.capture == true {
        let victim = get_victim(board_position, mv);
        return get_MVV_LVA(victim, mv.piece.to_usize());
    }
    else {
        
    }
    
    return 0;

}

pub fn rand_search(board_position: &BoardPosition) {

    let mut moves = generate_moves(board_position);
    
    let mut mv = moves.pop();
    
    while mv.is_none() {
        mv = moves.pop();
    }
    
    println!("bestmove {}", move_to_alg(&mv.unwrap()))
}


pub fn quiescence(board_position: &BoardPosition, alpha: i32, beta: i32) -> (i32, i32) {

    let eval = evaluate(board_position);

    if eval >= beta
    {
        return (beta,0);
    }

    let mut new_alpha = alpha;

    if (eval > alpha)
    {
        new_alpha = eval;
    }

    let move_list = generate_moves(&board_position);
    let mut filtered_move_list : Vec<Move> = move_list.into_iter().filter(|mv| mv.capture == true).collect();
    filtered_move_list.sort_by(|a, b| {
        let score_a = get_move_score(board_position, a);
        let score_b = get_move_score(board_position, b);
        score_b.cmp(&score_a)
    });

    let mut nodes = 1;

    for mv in filtered_move_list {
        let nbp_option = make_move(&board_position, &mv);

        if let Some(nbp) = nbp_option {
            let res = quiescence(&nbp, -beta, -new_alpha);
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
        let score = quiescence(board_position, alpha, beta);
        return (vec![], score.0, score.1)
    }

    let mut new_alpha = alpha;

    let mut moveList = generate_moves(&board_position);
    moveList.sort_by(|a, b| {
        let score_a = get_move_score(board_position, a);
        let score_b = get_move_score(board_position, b);
        score_b.cmp(&score_a)
    });
    
    // Move, eval (alpha), nodes
    let mut nodes = 1;

    let mut bestMove = None;
    let mut bestMoveList = vec![];

    let mut legal_moves = 0;
    for mv in moveList {

        let nbpOption = make_move(&board_position, &mv);

        if let Some(nbp) = nbpOption {
            legal_moves += 1;
            let res = negamax(&nbp, -beta, -new_alpha, depth - 1);
            nodes += res.2;

            if -res.1 >= beta {
                return (vec![], beta, nodes);
            }

            if -res.1 > new_alpha {
                new_alpha = -res.1;
                bestMove = Some(mv);
                bestMoveList = res.0;
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

    bestMoveList.push(bestMove);
    (bestMoveList, new_alpha, nodes)
}

pub fn score_to_mate( score: i32, depth: usize) -> i32 {
    if score > 0 {
        return (- score + 4999901 + depth as i32 ) / 2
    }
    (- score - 4999900  - depth as i32 ) / 2
}

pub fn collectPv(moves: &Vec<Option<Move>>) -> String {
    moves
        .iter()
        .filter_map(|x| x.as_ref().map(move_to_alg))
        .rev()
        .reduce(|a, b| a + " " + &b)
        .unwrap_or_default()
}

pub fn search(board_position: &BoardPosition, depth: usize) {
    unsafe {
        max_depth = depth; 
    }
    

    
    let mut score = negamax(&board_position, -5000000, 5000000, depth);

    let pv = collectPv(&score.0);

    if score.1 > 4000000 || score.1 < -4000000 {
        
        let mate = score_to_mate( score.1, depth);

        println!("info score mate {} depth {} nodes {} pv {}", mate, depth, score.2, pv);
    }
    else {
        println!("info score cp {} depth {} nodes {} pv {}", score.1, depth, score.2, pv);
    }
    println!("Movelist: {:?}", score.0);
    println!("bestmove {}", move_to_alg(&score.0.pop().unwrap().unwrap()))
}



