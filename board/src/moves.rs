use crate::board::Board;
use crate::board::{BISHOP, ROOK, QUEEN};
use crate::board::print_bitboard;

const CASTLE_WHITE_KING_SIDE: u32 = 0b0001;
const CASTLE_WHITE_QUEEN_SIDE: u32 = 0b0010;
const CASTLE_BLACK_QUEEN_SIDE: u32 = 0b0100;
const CASTLE_BLACK_KING_SIDE: u32 = 0b1000;

const WHITE_KING_SQUARE: u64 = 4;
const BLACK_KING_SQUARE: u64 = 60;

pub struct Move {
    pub from: u64,
    pub to: u64,
    pub promotion_type: char,
    pub capture: char,
    pub castle_rights: u32,
}

#[derive(Clone, Copy)]
struct GeneratorBoard<'a> {
    pub board: &'a Board,
    pub white_attacks: u64,
    pub black_attacks: u64,
    pub white_king: u64,
    pub black_king: u64,
}

fn find_king(board: &Board, white: bool) -> u64 {
    let mut index = 0;
    for square in 0..64 {
        if board.bitboards[5] & (1u64 << square) != 0 {
            if white && (board.white_pieces() & (1u64 << square)) == 0 {
                index = square;
                break;
            }
            if !white && (board.black_pieces() & (1u64 << square)) == 0 {
                index = square;
                break;
            }
        }
    }

    index
}

fn get_attacks(board: &Board) -> GeneratorBoard {
    let mut att_board = GeneratorBoard {
        board,
        white_attacks: 0,
        black_attacks: 0,
        white_king: 0,
        black_king: 0,
    };

    for square in 0..64 {
        for piece in 0..6 {
            if board.bitboards[piece] & (1u64 << square) != 0 {
                let val = match piece {
                    0 => get_pawn_attacks(board, square),
                    1 => get_knight_attacks(square),
                    2 => get_bishop_attacks(board, square),
                    3 => get_rook_attacks(board, square),
                    4 => get_queen_attacks(board, square),
                    5 => get_king_attacks(square),
                    _ => { unreachable!("Invalid board"); }
                };

                if board.white_pieces() & (1u64 << square) != 0 {
                    att_board.white_attacks |= val;
                } else {
                    att_board.black_attacks |= val;
                }

                break; // Piece found, no need to check other types
            }
        }
    }

    att_board
}

pub fn get_piece_moves(board: &Board, square: u64, en_passant: u64) -> u64 {
    get_piece_moves_wa(&get_attacks(board), square, en_passant)
}

fn get_piece_moves_wa(attacks: &GeneratorBoard, square: u64, en_passant: u64) -> u64 {
    for (i, bitboard) in attacks.board.bitboards.iter().enumerate() {
        if i == 6 || i == 7 { continue; }

        if bitboard & (1u64 << square) != 0 {
            return match i {
                0 => get_pawn_moves(*attacks, square, en_passant),
                1 => get_knight_moves(*attacks, square),
                2 => get_bishop_moves(*attacks, square),
                3 => get_rook_moves(*attacks, square),
                4 => get_queen_moves(*attacks, square),
                5 => get_king_moves(*attacks, square),
                _ => { unreachable!("Invalid board"); }
            };
        }
    }
    panic!("No piece on the given square");
}

fn find_piece_type(board: &Board, square: u64) -> char {
    for (piece, bitboard) in board.bitboards.iter().enumerate() {
        if piece == 6 || piece == 7 { continue; }

        if bitboard & (1u64 << square) != 0 {
            return if board.white_pieces() & (1u64 << square) != 0 {
                match piece {
                    0 => 'P',
                    1 => 'N',
                    2 => 'B',
                    3 => 'R',
                    4 => 'Q',
                    5 => 'K',
                    _ => { unreachable!("Invalid board"); }
                }
            } else {
                match piece {
                    0 => 'p',
                    1 => 'n',
                    2 => 'b',
                    3 => 'r',
                    4 => 'q',
                    5 => 'k',
                    _ => { unreachable!("Invalid board"); }
                }
            };
        }
    }
    panic!("No piece on the given square");
}

fn count_check(board: &GeneratorBoard, white: bool) -> (bool, u64) {
    let mut possible_blocks: u64 = 0;
    let mut count: u64 = 0;
    for square in 0..64 {
        for piece in 0..6 {
            if board.board.bitboards[piece] & (1u64 << square) != 0 {
                let val = match piece {
                    0 => get_pawn_attacks(board.board, square),
                    1 => get_knight_attacks(square),
                    2 => get_bishop_attacks(board.board, square),
                    3 => get_rook_attacks(board.board, square),
                    4 => get_queen_attacks(board.board, square),
                    5 => get_king_attacks(square),
                    _ => { unreachable!("Invalid board"); }
                };

                if (1u64 << if white { board.white_king } else { board.black_king }) & val != 0 {
                    if count == 1 {
                        return (true, 0); // double check
                    } else {
                        count = 1;

                        possible_blocks |= 1u64 << square; // allow capture of the piece
                        possible_blocks |= match piece {
                            0 => 0,
                            1 => 0,
                            2 => get_between(board, piece as u64, if white { board.white_king } else { board.black_king }),
                            3 => get_between(board, piece as u64, if white { board.white_king } else { board.black_king }),
                            4 => get_between(board, piece as u64, if white { board.white_king } else { board.black_king }),
                            _ => { unreachable!("Invalid board"); }
                        };
                    }
                }
                break;
            }
        }
    }

    (false, possible_blocks)
}

pub fn get_moves(board: &Board, en_passant: u64, castle_rights: u32, white: bool) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    let mut att_board = get_attacks(board);
    att_board.white_king = find_king(board, true);
    att_board.black_king = find_king(board, false);

    print_bitboard(att_board.white_attacks, 'X', '.');
    print_bitboard(att_board.black_attacks, 'X', '.');

    let my_pieces = if white { board.white_pieces() } else { board.black_pieces() };
    let mut possible_squares = !0u64;
    let mut double_check = false;
    if (white && (att_board.black_attacks & (1u64 << att_board.white_king) != 0)) ||
        (!white && (att_board.white_attacks & (1u64 << att_board.black_king) != 0)) {
        // in check
        (double_check, possible_squares) = count_check(&att_board, white);
    }

    if double_check {
        // Only king can move
        let possible_moves = get_piece_moves_wa(&att_board, att_board.white_king, en_passant);
        for pm_square in 0..64 {
            if possible_moves & (1u64 << pm_square) != 0 {
                let mut capture = 'z';
                if board.all_pieces() & (1u64 << pm_square) != 0 {
                    // find the piece that is being captured
                    capture = find_piece_type(board, pm_square);
                }

                moves.push(Move {
                    from: att_board.white_king,
                    to: pm_square,
                    promotion_type: 'z',
                    capture,
                    castle_rights,
                });
            }
        }

        return moves;
    }

    // Normal moves
    let king_type = if white { 'K' } else { 'k' };
    for square in 0..64 {
        if my_pieces & (1u64 << square) == 0 { continue; }
        let possible_moves = get_piece_moves_wa(&att_board, square, en_passant) &
            (if find_piece_type(board, square) == king_type { possible_squares } else { !0u64 });

        for pm_square in 0..64 {
            if possible_moves & (1u64 << pm_square) != 0 {
                let move_piece_type = find_piece_type(board, square);
                let mut capture = 'z';
                if board.all_pieces() & (1u64 << pm_square) != 0 {
                    // find the piece that is being captured
                    capture = find_piece_type(board, pm_square);
                }

                // Deal with promotion
                if (move_piece_type == 'p' && pm_square / 8 == 0) || (move_piece_type == 'P' && pm_square / 8 == 7) {
                    for promotion in ['q', 'r', 'b', 'n'] {
                        moves.push(Move {
                            from: square,
                            to: pm_square,
                            promotion_type: promotion,
                            capture,
                            castle_rights,
                        });
                    }
                    break;
                } else {
                    moves.push(Move {
                        from: square,
                        to: pm_square,
                        promotion_type: 'z',
                        capture,
                        castle_rights,
                    });
                }
            }
        }
    }

    fn check_castling(castle_rights: u32, pieces: u64, side: u32, king_square: i32, direction: i32, attacks: u64) -> bool {
        ((castle_rights & side) != 0) &&
            ((pieces & (1u64 << (king_square + direction))) == 0) && // No pieces in between
            ((pieces & (1u64 << (king_square + direction * 2))) == 0) &&
            ((attacks & (1u64 << king_square)) == 0) && // No checks in between
            ((attacks & (1u64 << (king_square + direction))) == 0) &&
            ((attacks & (1u64 << (king_square + direction * 2))) == 0)
    }

    // Castling
    if check_castling(castle_rights, board.all_pieces(), CASTLE_WHITE_KING_SIDE,
                      WHITE_KING_SQUARE as i32, -1, att_board.black_attacks) {
        moves.push(Move {
            from: WHITE_KING_SQUARE,
            to: WHITE_KING_SQUARE - 2,
            promotion_type: 'z',
            capture: 'z',
            castle_rights,
        });
    }
    if check_castling(castle_rights, board.all_pieces(), CASTLE_WHITE_QUEEN_SIDE,
                      WHITE_KING_SQUARE as i32, 1, att_board.black_attacks) {
        moves.push(Move {
            from: WHITE_KING_SQUARE,
            to: WHITE_KING_SQUARE + 2,
            promotion_type: 'z',
            capture: 'z',
            castle_rights,
        });
    }
    if check_castling(castle_rights, board.all_pieces(), CASTLE_BLACK_KING_SIDE,
                      BLACK_KING_SQUARE as i32, -1, att_board.white_attacks) {
        moves.push(Move {
            from: BLACK_KING_SQUARE,
            to: BLACK_KING_SQUARE - 2,
            promotion_type: 'z',
            capture: 'z',
            castle_rights,
        });
    }
    if check_castling(castle_rights, board.all_pieces(), CASTLE_BLACK_QUEEN_SIDE,
                      BLACK_KING_SQUARE as i32, 1, att_board.white_attacks) {
        moves.push(Move {
            from: BLACK_KING_SQUARE,
            to: BLACK_KING_SQUARE + 2,
            promotion_type: 'z',
            capture: 'z',
            castle_rights,
        });
    }

    moves
}

fn piece_on_top_left(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 > 0) && (index / 8 < 8) {
        index += 7;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_top_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if (square / 8 < king / 8 && square % 8 > king % 8) &&
        (king / 8 - square / 8 == king % 8 - square % 8) {
        piece_on_top_left(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_top_right(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 < 8) && (index / 8 > 0) {
        index += 9;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_top_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if (square / 8 < king / 8 && square % 8 < king % 8) &&
        (king / 8 - square / 8 == king % 8 - square % 8) {
        piece_on_top_right(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_bottom_left(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 > 0) && (index / 8 > 0) {
        index -= 7;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_bottom_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if (square / 8 > king / 8 && square % 8 > king % 8) &&
        (king / 8 - square / 8 == king % 8 - square % 8) {
        piece_on_bottom_left(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_bottom_right(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 < 8) && (index / 8 > 0) {
        index -= 9;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_bottom_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if (square / 8 > king / 8 && square % 8 < king % 8) &&
        (king / 8 - square / 8 == king % 8 - square % 8) {
        piece_on_bottom_right(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_left(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while index % 8 < 7 {
        index += 1;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if square / 8 == king / 8 && square % 8 < king % 8 {
        piece_on_left(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_right(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while index % 8 > 0 {
        index -= 1;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if square / 8 == king / 8 && square % 8 > king % 8 {
        piece_on_right(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_top(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while index / 8 < 7 {
        index += 8;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_top(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if square % 8 == king % 8 && square / 8 < king / 8 {
        piece_on_top(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_bottom(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while index / 8 > 0 {
        index -= 8;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_bottom(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king };

    if square % 8 == king % 8 && square / 8 > king / 8 {
        piece_on_bottom(board, square, 1u64 << king)
    } else {
        false
    }
}

fn top_left_bottom_right(board: &Board, square: u64) -> u64 {
    let mut moves: u64 = 0;
    let mut loc: u64 = square;
    let my_pieces = if board.white_pieces() & (1u64 << square) != 0 {
        board.white_pieces()
    } else {
        board.black_pieces()
    };

    while loc > 7 && loc % 8 > 0 { // Bottom Right
        loc -= 9;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 > 0 { // Top Left
        loc += 7;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    moves
}

fn top_right_bottom_left(board: &Board, square: u64) -> u64 {
    let mut moves: u64 = 0;
    let mut loc: u64 = square;
    let my_pieces = if board.white_pieces() & (1u64 << square) != 0 {
        board.white_pieces()
    } else {
        board.black_pieces()
    };

    while loc < 57 && loc % 8 < 7 { // Top Right
        loc += 9;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 && loc % 8 < 7 { // Bottom Left
        loc -= 7;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    moves
}

fn all_bishop_moves(board: GeneratorBoard, square: u64) -> u64 {
    top_left_bottom_right(board.board, square) | top_right_bottom_left(board.board, square)
}

fn top_bottom(board: &Board, square: u64) -> u64 {
    let mut moves: u64 = 0;
    let mut loc: u64 = square;
    let my_pieces = if board.white_pieces() & (1u64 << square) != 0 {
        board.white_pieces()
    } else {
        board.black_pieces()
    };

    while loc < 56 { // Top
        loc += 8;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 { // Bottom
        loc -= 8;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    moves
}

fn left_right(board: &Board, square: u64) -> u64 {
    let mut moves: u64 = 0;
    let mut loc: u64 = square;
    let my_pieces = if board.white_pieces() & (1u64 << square) != 0 {
        board.white_pieces()
    } else {
        board.black_pieces()
    };

    while loc % 8 > 0 { // Right
        loc -= 1;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 < 7 { // Left
        loc += 1;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    moves
}

fn all_rook_moves(board: GeneratorBoard, square: u64) -> u64 {
    left_right(board.board, square) | top_bottom(board.board, square)
}

fn get_pawn_moves(board: GeneratorBoard, square: u64, en_passant: u64) -> u64 {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_pieces = if white { board.board.black_pieces() } else { board.board.white_pieces() };
    let target_bishop = (board.board.bitboards[BISHOP] | board.board.bitboards[QUEEN]) & target_pieces;
    let target_rook = (board.board.bitboards[ROOK] | board.board.bitboards[QUEEN]) & target_pieces;

    fn right_attacks(board: &Board, square: u64, en_passant: u64) -> u64 {
        if board.white_pieces() & (1u64 << square) != 0 {
            if square % 8 > 0 {
                (1u64 << (square + 7)) & (board.black_pieces() & (1u64 << en_passant))
            } else { 0 }
        } else if square % 8 > 0 {
            (1u64 << (square - 9)) & (board.white_pieces() & (1u64 << en_passant))
        } else { 0 }
    }

    fn left_attacks(board: &Board, square: u64, en_passant: u64) -> u64 {
        if board.white_pieces() & (1u64 << square) != 0 {
            if square % 8 < 7 {
                (1u64 << (square + 9)) & (board.black_pieces() & (1u64 << en_passant))
            } else { 0 }
        } else if square % 8 < 7 {
            (1u64 << (square - 7)) & (board.white_pieces() & (1u64 << en_passant))
        } else { 0 }
    }

    fn forward_moves(board: &Board, square: u64) -> u64 {
        let mut moves: u64 = 0;
        if board.white_pieces() & (1u64 << square) != 0 {
            if (1u64 << (square + 8)) & board.all_pieces() == 0 {
                moves |= 1u64 << (square + 8);
                if square % 8 == 1 && board.all_pieces() & (1u64 << (square + 16)) == 0 {
                    moves |= 1u64 << (square + 16); // can move double
                }
            }
        } else if (1u64 << (square - 8)) & board.all_pieces() == 0 {
            moves |= 1u64 << (square - 8);
            if square % 8 == 7 && board.all_pieces() & (1u64 << (square - 16)) == 0 {
                moves |= 1u64 << (square - 16); // can move double
            }
        }

        moves
    }

    fn all_pawn_moves(board: GeneratorBoard, square: u64, en_passant: u64) -> u64 {
        right_attacks(board.board, square, en_passant) |
            left_attacks(board.board, square, en_passant) |
            forward_moves(board.board, square)
    }

    if is_king_top_left(&board, square, white) {
        if piece_on_bottom_right(&board, square, target_bishop) {
            left_attacks(board.board, square, en_passant)
        } else { all_pawn_moves(board, square, en_passant) }
    } else if is_king_top_right(&board, square, white) {
        if piece_on_bottom_left(&board, square, target_bishop) {
            right_attacks(board.board, square, en_passant)
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_bottom_left(&board, square, white) {
        if piece_on_top_right(&board, square, target_bishop) {
            right_attacks(board.board, square, en_passant)
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_bottom_right(&board, square, white) {
        if piece_on_top_left(&board, square, target_bishop) {
            left_attacks(board.board, square, en_passant)
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_left(&board, square, white) {
        if piece_on_right(&board, square, target_rook) {
            0 // no moves
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_right(&board, square, white) {
        if piece_on_left(&board, square, target_rook) {
            0 // no moves
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_top(&board, square, white) {
        if piece_on_bottom(&board, square, target_rook) {
            forward_moves(board.board, square)
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else if is_king_bottom(&board, square, white) {
        if piece_on_top(&board, square, target_rook) {
            forward_moves(board.board, square)
        } else {
            all_pawn_moves(board, square, en_passant)
        }
    } else {
        all_pawn_moves(board, square, en_passant)
    }
}

fn king_diag_pinned(board: &GeneratorBoard, square: u64) -> bool {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_bishop = (board.board.bitboards[BISHOP] | board.board.bitboards[QUEEN]) &
        (if white { board.board.black_pieces() } else { board.board.white_pieces() });

    (is_king_top_left(board, square, white) && piece_on_bottom_right(board, square, target_bishop)) ||
        (is_king_top_right(board, square, white) && piece_on_bottom_left(board, square, target_bishop)) ||
        (is_king_bottom_left(board, square, white) && piece_on_top_right(board, square, target_bishop)) ||
        (is_king_bottom_right(board, square, white) && piece_on_top_left(board, square, target_bishop))
}

//noinspection DuplicatedCode
fn get_knight_moves(board: GeneratorBoard, square: u64) -> u64 {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_pieces = if white { board.board.black_pieces() } else { board.board.white_pieces() };
    let target_rook = (board.board.bitboards[ROOK] | board.board.bitboards[QUEEN]) & target_pieces;

    if king_diag_pinned(&board, square) ||
        (is_king_left(&board, square, white) && piece_on_right(&board, square, target_rook)) ||
        (is_king_right(&board, square, white) && piece_on_left(&board, square, target_rook)) ||
        (is_king_top(&board, square, white) && piece_on_bottom(&board, square, target_rook)) ||
        (is_king_bottom(&board, square, white) && piece_on_top(&board, square, target_rook))
    {
        0 // knight can't move when pinned
    } else if white {
        get_knight_attacks(square) & (!board.board.white_pieces()) // Don't move onto own pieces
    } else {
        get_knight_attacks(square) & (!board.board.black_pieces())
    }
}

fn get_bishop_moves(board: GeneratorBoard, square: u64) -> u64 {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_pieces = if white { board.board.black_pieces() } else { board.board.white_pieces() };
    let target_bishop = (board.board.bitboards[BISHOP] | board.board.bitboards[QUEEN]) & target_pieces;
    let target_rook = (board.board.bitboards[ROOK] | board.board.bitboards[QUEEN]) & target_pieces;

    if (is_king_left(&board, square, white) && piece_on_right(&board, square, target_rook)) ||
        (is_king_right(&board, square, white) && piece_on_left(&board, square, target_rook)) ||
        (is_king_top(&board, square, white) && piece_on_bottom(&board, square, target_rook)) ||
        (is_king_bottom(&board, square, white) && piece_on_top(&board, square, target_rook)) {
        0 // bishop can't move when pinned from the side
    } else if is_king_top_left(&board, square, white) {
        if piece_on_bottom_right(&board, square, target_bishop) {
            top_left_bottom_right(board.board, square)
        } else {
            all_bishop_moves(board, square)
        }
    } else if is_king_top_right(&board, square, white) {
        if piece_on_bottom_left(&board, square, target_bishop) {
            top_right_bottom_left(board.board, square)
        } else {
            all_bishop_moves(board, square)
        }
    } else if is_king_bottom_left(&board, square, white) {
        if piece_on_top_right(&board, square, target_bishop) {
            top_right_bottom_left(board.board, square)
        } else {
            all_bishop_moves(board, square)
        }
    } else if is_king_bottom_right(&board, square, white) {
        if piece_on_top_left(&board, square, target_bishop) {
            top_left_bottom_right(board.board, square)
        } else {
            all_bishop_moves(board, square)
        }
    } else {
        all_bishop_moves(board, square)
    }
}

fn get_rook_moves(board: GeneratorBoard, square: u64) -> u64 {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_pieces = if white { board.board.black_pieces() } else { board.board.white_pieces() };
    let target_rook = (board.board.bitboards[ROOK] | board.board.bitboards[QUEEN]) & target_pieces;

    if king_diag_pinned(&board, square)
    {
        0 // rook can't move when pinned diagonally
    } else if is_king_left(&board, square, white) {
        if piece_on_right(&board, square, target_rook) {
            left_right(board.board, square)
        } else {
            all_rook_moves(board, square)
        }
    } else if is_king_right(&board, square, white) {
        if piece_on_left(&board, square, target_rook) {
            left_right(board.board, square)
        } else {
            all_rook_moves(board, square)
        }
    } else if is_king_top(&board, square, white) {
        if piece_on_bottom(&board, square, target_rook) {
            top_bottom(board.board, square)
        } else {
            all_rook_moves(board, square)
        }
    } else if is_king_bottom(&board, square, white) {
        if piece_on_top(&board, square, target_rook) {
            top_bottom(board.board, square)
        } else {
            all_rook_moves(board, square)
        }
    } else {
        all_rook_moves(board, square) // moves
    }
}

fn get_queen_moves(board: GeneratorBoard, square: u64) -> u64 {
    get_bishop_moves(board, square) | get_rook_moves(board, square)
}

fn get_king_moves(board: GeneratorBoard, square: u64) -> u64 {
    if board.board.white_pieces() & (1u64 << square) != 0 {
        get_king_attacks(square) & (!board.board.white_pieces()) & (!board.black_attacks)
    } else {
        get_king_attacks(square) & (!board.board.black_pieces()) & (!board.white_attacks)
    }
}

fn get_pawn_attacks(board: &Board, square: u64) -> u64 {
    if board.white_pieces() & (1u64 << square) != 0 {
        (if square % 8 > 0 { 1u64 << (square + 7) } else { 0 }) |
            (if square % 8 < 7 { 1u64 << (square + 9) } else { 0 })
    } else {
        (if square % 8 < 7 { 1u64 << (square - 7) } else { 0 }) |
            (if square % 8 > 0 { 1u64 << (square - 9) } else { 0 })
    }
}

fn get_knight_attacks(square: u64) -> u64 {
    let mut attacks: u64 = 0;

    if square / 8 > 1 {
        attacks |= if square % 8 > 0 { 1u64 << (square - 17) } else { 0 } |
            if square % 8 < 7 { 1u64 << (square - 15) } else { 0 }
    }

    if square / 8 > 0 {
        attacks |= if square % 8 > 1 { 1u64 << (square - 10) } else { 0 } |
            if square % 8 < 6 { 1u64 << (square - 6) } else { 0 }
    }

    if square / 8 < 7 {
        attacks |= if square % 8 > 1 { 1u64 << (square + 6) } else { 0 } |
            if square % 8 < 6 { 1u64 << (square + 10) } else { 0 }
    }

    if square / 8 < 6 {
        attacks |= if square % 8 > 0 { 1u64 << (square + 15) } else { 0 } |
            if square % 8 < 7 { 1u64 << (square + 17) } else { 0 }
    }

    attacks
}

fn get_bishop_attacks(board: &Board, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc > 7 && loc % 8 > 0 { // Bottom Right
        loc -= 9;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 && loc % 8 < 7 { // Bottom Left
        loc -= 7;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 > 0 { // Top Left
        loc += 7;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 < 7 { // Top Right
        loc += 9;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    attacks
}

fn get_rook_attacks(board: &Board, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc < 56 { // Top
        loc += 8;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 { // Bottom
        loc -= 8;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 > 0 { // Right
        loc -= 1;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 < 7 { // Left
        loc += 1;
        attacks |= 1u64 << loc;

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    attacks
}

fn get_queen_attacks(board: &Board, square: u64) -> u64 {
    get_bishop_attacks(board, square) | get_rook_attacks(board, square) // ez
}

fn get_king_attacks(square: u64) -> u64 {
    let mut attacks: u64 = if square < 56 { 1u64 << (square + 8) } else { 0 } |
        if square > 7 { 1u64 << (square - 8) } else { 0 }; // Top and Bottom

    if square % 8 < 7 { // Left
        attacks |= 1u64 << if square < 55 { square + 9 } else { 0 };
        attacks |= 1u64 << if square < 64 { square + 1 } else { 0 };
        attacks |= 1u64 << if square > 6 { square - 7 } else { 0 };
    }
    if square % 8 > 0 { // Right
        attacks |= 1u64 << if square < 57 { square + 7 } else { 0 };
        attacks |= 1u64 << if square > 0 { square - 1 } else { 0 };
        attacks |= 1u64 << if square > 8 { square - 9 } else { 0 };
    }

    attacks
}

fn get_between(board: &GeneratorBoard, start: u64, end: u64) -> u64 {
    let mut between: u64 = 0;
    let mut loc: u64 = start;
    let target = 1u64 << end;

    if piece_on_top_left(board, start, target) {
        while loc < 57 && loc % 8 > 0 { // Top Left
            loc += 7;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_top_right(board, start, target) {
        while loc < 57 && loc % 8 < 7 { // Top Right
            loc += 9;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_bottom_left(board, start, target) {
        while loc > 7 && loc % 8 < 7 { // Bottom Left
            loc -= 7;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_bottom_right(board, start, target) {
        while loc > 7 && loc % 8 > 0 { // Bottom Right
            loc -= 9;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_left(board, start, target) {
        while loc % 8 < 7 { // Left
            loc += 1;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_right(board, start, target) {
        while loc % 8 > 0 { // Right
            loc -= 1;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_top(board, start, target) {
        while loc < 56 { // Top
            loc += 8;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_bottom(board, start, target) {
        while loc > 7 { // Bottom
            loc -= 8;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else {
        panic!("No way to get between squares");
    }

    between
}