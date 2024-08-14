use board::moves::get_moves;
use board::moves::Move;
use evaluate::{evaluate, order_moves};
use evaluate::BoardState;

pub mod evaluate;

const MASSIVE_NUMBER: i32 = 100000000;

pub fn alpha_beta_max(board: &mut BoardState, depth: u32, mut alpha: i32, beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut best_value = -MASSIVE_NUMBER;
    let mut move_list = get_moves(&board.board, board.en_passant, board.castle_rights, board.white);
    order_moves(board, &mut move_list); // Order the moves to make alpha beta search faster

    for move_ in move_list {
        // Make the move
        let (en_passant, castle_rights) = move_.make_move(&mut board.board);
        board.en_passant = en_passant; // Change the new values for the next move
        board.castle_rights = castle_rights;
        board.white = !board.white;

        let score = alpha_beta_min(board, depth - 1, alpha, beta);

        // Unmake the move
        move_.unmake_move(&mut board.board);
        board.en_passant = move_.en_passant as u64; // Reset old values
        board.castle_rights = move_.castle_rights;
        board.white = !board.white;

        if score > best_value {
            best_value = score;
            if score > alpha {
                alpha = score;
            }
        }
        if score >= beta {
            return score; // fail soft
        }
    }

    best_value
}

pub fn alpha_beta_min(board: &mut BoardState, depth: u32, alpha: i32, mut beta: i32) -> i32 {
    if depth == 0 {
        return evaluate(board);
    }

    let mut best_value = MASSIVE_NUMBER;
    let mut move_list = get_moves(&board.board, board.en_passant, board.castle_rights, board.white);
    order_moves(board, &mut move_list); // Order the moves to make alpha beta search faster

    for move_ in move_list {
        // Make the move
        let (en_passant, castle_rights) = move_.make_move(&mut board.board);
        board.en_passant = en_passant; // Change the new values for the next move
        board.castle_rights = castle_rights;
        board.white = !board.white;

        let score = alpha_beta_max(board, depth - 1, alpha, beta);

        // Unmake the move
        move_.unmake_move(&mut board.board);
        board.en_passant = move_.en_passant as u64; // Reset old values
        board.castle_rights = move_.castle_rights;
        board.white = !board.white;

        if score < best_value {
            best_value = score;
            if score < beta {
                beta = score;
            }
        }
        if score <= alpha {
            return score; // fail soft
        }
    }

    best_value
}

// pub fn alpha_beta_negamax_evaluate(board: &mut BoardState, depth: u32, mut alpha: i32, beta: i32) -> i32 { // Negamax
//     if depth == 0 {
//         return evaluate(board);
//     }
//
//     let mut move_list = get_moves(&board.board, board.en_passant, board.castle_rights, board.white);
//     order_moves(board, &mut move_list); // Order the moves to make alpha beta search faster
//     for move_ in move_list {
//         // Make the move
//         let (en_passant, castle_rights) = move_.make_move(&mut board.board);
//         board.en_passant = en_passant; // Change the new values for the next move
//         board.castle_rights = castle_rights;
//         board.white = !board.white;
//
//         // Recursively search the next move
//         let value = -alpha_beta_negamax_evaluate(board, depth - 1, -beta, -alpha);
//
//         // Unmake the move
//         move_.unmake_move(&mut board.board);
//         board.en_passant = move_.en_passant as u64; // Reset old values
//         board.castle_rights = move_.castle_rights;
//         board.white = !board.white;
//
//         if value >= beta {
//             return beta;
//         }
//         if value > alpha {
//             alpha = value;
//         }
//     }
//
//     alpha
// }

pub fn get_best_move(board: &mut BoardState, depth: u32) -> Move {
    get_best_moves(board, depth)[0].0
}

pub fn get_best_moves(board: &mut BoardState, depth: u32) -> Vec<(Move, i32)> {
    if depth == 0 { assert_ne!(depth, 0, "Depth can't be 0"); }

    let mut move_list = get_moves(&board.board, board.en_passant, board.castle_rights, board.white);
    order_moves(board, &mut move_list); // Order the moves to make alpha beta search faster

    let mut eval_list: Vec<(Move, i32)> = Vec::new();
    for move_ in move_list.iter_mut() {
        let (en_passant, castle_rights) = move_.make_move(&mut board.board);
        board.en_passant = en_passant; // Change the new values for the next move
        board.castle_rights = castle_rights;
        board.white = !board.white;

        eval_list.push(
            if board.white {
                (*move_, alpha_beta_max(board, depth - 1, -MASSIVE_NUMBER, MASSIVE_NUMBER))
            } else {
                (*move_, alpha_beta_min(board, depth - 1, MASSIVE_NUMBER, -MASSIVE_NUMBER))
            }
        );

        move_.unmake_move(&mut board.board);
        board.en_passant = move_.en_passant as u64; // Reset old values
        board.castle_rights = move_.castle_rights;
        board.white = !board.white;
    }


    eval_list.sort_by(
        |a: &(Move, i32), b: &(Move, i32)| {
            if a.1 > b.1 {
                std::cmp::Ordering::Greater
            } else {
                std::cmp::Ordering::Less
            }
        }
    );

    eval_list
}

#[cfg(test)]
mod tests {
    use board::board::{create_default_board, square_to_algebraic};
    use super::*;

    #[test]
    fn test_quick_eval() {
        let mut board = BoardState {
            board: create_default_board(),
            en_passant: 0,
            castle_rights: 0xf,
            white: true,
            half_move_clock: 0,
            full_move_count: 0,
        };

        // println!("{}", alpha_beta_negamax_evaluate(&mut board, 20, -1000000, 10000000));

        let best_move = get_best_move(&mut board, 10);
        println!("Best move: {}{}", square_to_algebraic(best_move.from), square_to_algebraic(best_move.to));
        println!("\nBest move list:");
        for eval in get_best_moves(&mut board, 10) {
            println!("{}{}: {}", square_to_algebraic(eval.0.from), square_to_algebraic(eval.0.to), eval.1);
        }
    }
}