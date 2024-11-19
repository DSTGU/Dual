use crate::evaluate::evaluate;
use crate::moveGen::{generate_moves, is_square_attacked, make_move};
use crate::shared::{coordinates_to_squares, moveToAlg, BoardPosition, Move};

pub fn rand_search(board_position: &BoardPosition) {

    let mut moves = generate_moves(board_position);
    
    let mut mv = moves.pop();
    
    while mv.is_none() {
        mv = moves.pop();
    }
    
    println!("bestmove {}", moveToAlg(&mv.unwrap()))
}

pub fn negamax(board_position: &BoardPosition, alpha: i32, beta: i32, depth: usize) -> (Vec<Option<Move>>, i32, i32) {
    if depth == 0 {
        return (vec![], evaluate(board_position), 0);
    }

    let mut new_alpha = alpha;

    let moveList = generate_moves(&board_position);
    
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
                return (vec![], -500000, 1)
            }
            else {
                return (vec![], 0, 1)
            }
    }

    bestMoveList.push(bestMove);
    (bestMoveList, new_alpha, nodes)
}

pub fn search(board_position: &BoardPosition, depth: usize) {
    let mut score = negamax(&board_position, -500000, 500000, depth);
    

    println!("info debug eval {} nodes {}", score.1, score.2);
    println!("Movelist: {:?}", score.0);
    println!("bestmove {}", moveToAlg(&score.0.pop().unwrap().unwrap()))
}



