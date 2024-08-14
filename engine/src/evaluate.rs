use board::board::Board;
use board::moves::{get_moves, in_check, Move};
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
    const FOUR: u64 = 0x000018180000;
    const CENTER: u64 = 0x00003c3c0000 | FOUR;
    const SIDES: u64 = 0x007e42427e00 | CENTER;
    const EDGES: u64 = 0xff81818181ff | SIDES; // 0xffffffffffff

    let val = 1u64 << square;

    (if (square & EDGES) != 0 { CENTIPAWN } else { 0 }) +
        if (val & SIDES) != 0 { CENTIPAWN } else { 0 } +
        if (val & CENTER) != 0 { CENTIPAWN } else { 0 } +
        if (val & FOUR) != 0 { CENTIPAWN } else { 0 }
}

fn evaluate_king_safety(board: &mut BoardState, my_king: u64, other_attacks: u64) -> i32 {
    if my_king & other_attacks != 0 {
        -CENTIPAWN * 2 // Prefer king to not be in check
    } else {
        0 // 20
    }
}

fn evaluate_attacks(attacks: u64) -> i32 {
    attacks.count_ones() as i32
}

pub fn evaluate(board: &mut BoardState) -> i32 {
    let mut score = 0;
    let white_attacks = get_attacks(&board.board, board.white);
    let black_attacks = get_attacks(&board.board, !board.white);

    score += evaluate_king_safety(board, 1u64 << find_king(&board.board, true), black_attacks);
    score -= evaluate_king_safety(board, 1u64 << find_king(&board.board, false), white_attacks);
    score += evaluate_attacks(white_attacks);
    score -= evaluate_attacks(black_attacks);

    score += (white_attacks & board.board.white_pieces()).count_ones() as i32; // prefer defended pieces
    score -= (black_attacks & board.board.black_pieces()).count_ones() as i32;

    score -= ((black_attacks & board.board.white_pieces()).count_ones() / 2) as i32; // prefer pieces that aren't attacked
    score += ((white_attacks & board.board.black_pieces()).count_ones() / 2) as i32;

    // Per piece evaluation
    for square in 0..64 {
        if board.board.all_pieces() & (1u64 << square) != 0 {
            let piece_type = find_piece_type(&board.board, square);
            score += if piece_type.is_uppercase() { 1 } else { -1 } * match piece_type.to_lowercase().next().unwrap() {
                'p' => 100,
                'n' => 300,
                'b' => 300,
                'r' => 500,
                'q' => 900,
                'k' => 2000,
                _ => {
                    unreachable!("Invalid board");
                }
            };

            let val = if board.full_move_count < 40 {
                distance_from_center(square) * match piece_type.to_lowercase().next().unwrap() {
                    'p' => 3,
                    'n' => 4,
                    'b' => 4,
                    'r' => 2,
                    'q' => 0,
                    'k' => -2,
                    _ => {
                        unreachable!("Invalid board");
                    }
                }
            } else {
                distance_from_center(square) * match piece_type.to_lowercase().next().unwrap() {
                    'p' => 3,
                    'n' => 3,
                    'b' => 5,
                    'r' => 5,
                    'q' => 5,
                    'k' => 2,
                    _ => {
                        unreachable!("Invalid board");
                    }
                }
            };
            // println!("{}:{}:{}:{}", distance_from_center(square), piece_type, piece_type.to_lowercase().next().unwrap(), val);
            score += (if piece_type.is_uppercase() { 1 } else { -1 }) * val;
        }
    }

    let moves = get_moves(&board.board, board.en_passant, board.castle_rights, !board.white);
    if moves.is_empty() {
        if board.white {
            if in_check(&board.board, true) {
                return -1000000;
            }
        } else if in_check(&board.board, false) {
            return 1000000;
        }

        return 0; // stalemate
    }
    let factor = 0.3;
    score += ((if board.white { moves.len() as i32 } else { -(moves.len() as i32) } as f64) * factor) as i32;

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