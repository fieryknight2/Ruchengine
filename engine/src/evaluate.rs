use board::board::Board;
use board::moves::Move;
use board::moves::{get_attacks, find_king, find_piece_type};

const CENTIPAWN: i32 = 1; // Value doesn't really matter

pub struct BoardState {
    pub board: Board,
    pub en_passant: u64,
    pub castle_rights: u32,
    pub white: bool,
    pub half_move_clock: u32,
    pub full_move_count: u32,
}

fn distance_from_center(square: u64) -> i32 {
    let mut score = 0;

    let r = square / 8;
    let c = square % 8;
    for val in 0..4 {
        if r > val && c > val {
            score += CENTIPAWN;
        }
    }

    score
}

fn evaluate_king_safety(board: &mut BoardState, my_king: u64, other_attacks: u64) -> i32 {
    if my_king & other_attacks != 0 {
        -20 // Prefer king to not be in check
    } else {
        20
    }
}

fn evaluate_attacks(attacks: u64) -> i32 {
    attacks.count_ones() as i32
}

pub fn evaluate(board: &mut BoardState) -> i32 {
    let mut score = 0;
    let my_pieces = if board.white { board.board.white_pieces() } else { board.board.black_pieces() };
    let my_attacks = get_attacks(&board.board, board.white);
    // let other_pieces = if !board.white { board.board.white_pieces() } else { board.board.black_pieces() };
    let other_attacks = get_attacks(&board.board, !board.white);

    score += evaluate_king_safety(board, find_king(&board.board, board.white), other_attacks);
    score += evaluate_attacks(my_attacks);

    // Per piece evaluation
    for square in 0..64 {
        if my_pieces & (1u64 << square) != 0 {
            let piece_type = find_piece_type(&board.board, square);

            score += match piece_type.to_lowercase().next().unwrap() {
                'p' | 'n' | 'q' | 'r' | 'b' => {
                    distance_from_center(square) * match piece_type.to_lowercase().next().unwrap() {
                        'p' => 1,
                        'n' => 3,
                        'b' => 3,
                        'q' => 9,
                        'r' => 5,
                        _ => {
                            unreachable!("Invalid board");
                        }
                    }
                }
                'k' => {
                    if board.full_move_count < 40 {
                        let val = distance_from_center(square);
                        if val != 0 { 1 / distance_from_center(square) } else { 0 }
                    } else {
                        4 * distance_from_center(square)
                    }
                }
                _ => {
                    unreachable!("Invalid board");
                }
            }
        }
    }

    score
}

fn quick_eval_move(board: &mut BoardState, move_: &Move) -> i32 {
    let mut score = 0;

    score += distance_from_center(move_.to);

    score
}

pub fn order_moves(board: &mut BoardState, move_list: &mut [Move]) {
    move_list.sort_by(|a, b| {
        if quick_eval_move(board, a) > quick_eval_move(board, b) {
            std::cmp::Ordering::Greater
        } else {
            std::cmp::Ordering::Less
        }
    });
}