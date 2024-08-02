use crate::board::{Board, square_to_algebraic}; // , square_to_algebraic;
use crate::board::{BISHOP, ROOK, QUEEN};
// use crate::board::print_bitboard;

pub const CASTLE_WHITE_KING_SIDE: u32 = 0b0001;
pub const CASTLE_WHITE_QUEEN_SIDE: u32 = 0b0010;
pub const CASTLE_BLACK_QUEEN_SIDE: u32 = 0b0100;
pub const CASTLE_BLACK_KING_SIDE: u32 = 0b1000;

pub const WHITE_KING_SQUARE: u64 = 4;
pub const BLACK_KING_SQUARE: u64 = 60;

#[derive(Clone, Copy)]
pub struct Move {
    pub from: u64,
    pub to: u64,
    pub promotion_type: char,
    pub capture: char,
    pub en_passant: i32,
    pub castle_rights: u32,
}

impl Move {
    pub fn is_castle(&self) -> bool {
        (self.from == 60 && (self.to == 62 || self.to == 58)) ||
            (self.from == 4 && (self.to == 6 || self.to == 2))
    }

    fn get_castle_rook_pos(&self) -> (u64, u64, usize) {
        let rook_pos: u64;
        let rook_n_pos: u64;
        let color: usize;
        if self.from == 60 && self.to == 62 {
            rook_pos = 63;
            rook_n_pos = 61;
            color = 7;
        } else if self.from == 60 && self.to == 58 {
            rook_pos = 56;
            rook_n_pos = 59;
            color = 7;
        } else if self.from == 4 && self.to == 6 {
            rook_pos = 7;
            rook_n_pos = 5;
            color = 6;
        } else {
            rook_pos = 0;
            rook_n_pos = 3;
            color = 6;
        }

        (rook_pos, rook_n_pos, color)
    }

    pub fn make_move(&self, board: &mut Board) -> u64 { // returns an en passant square
        // println!("Before move {}{}={}", square_to_algebraic(self.from), square_to_algebraic(self.to), self.promotion_type);
        // board.print_board();
        let piece_type = find_piece_type(board, self.from);
        let piece_bitboard = get_bitboard_val(piece_type);

        board.bitboards[piece_bitboard] &= !(1u64 << self.from); // clear the piece from the old square
        board.bitboards[piece_bitboard] |= 1u64 << self.to; // set the piece to the new square

        let capture_bitboard = match self.capture {
            'p' | 'P' => 0,
            'n' | 'N' => 1,
            'b' | 'B' => 2,
            'r' | 'R' => 3,
            'q' | 'Q' => 4,
            'k' | 'K' => 5,
            _ => 6,
        };

        if capture_bitboard != 6 {
            if capture_bitboard != piece_bitboard {
                board.bitboards[capture_bitboard] &= !(1u64 << self.to);
            }

            if self.capture.is_uppercase() {
                board.bitboards[6] &= !(1u64 << self.to);
            } else {
                board.bitboards[7] &= !(1u64 << self.to);
            }
        }

        if self.promotion_type != 'z' {
            board.bitboards[match self.promotion_type {
                'n' => 1,
                'b' => 2,
                'r' => 3,
                'q' => 4,
                _ => { unreachable!("Invalid promotion type {}", self.promotion_type); }
            }] |= 1u64 << self.to; // set the promoted piece

            board.bitboards[0] &= !(1u64 << self.to); // clear the promoted pawn
        }

        if self.is_castle() {
            let (rook_pos, rook_n_pos, color) = self.get_castle_rook_pos();

            board.bitboards[color] |= 1u64 << rook_n_pos; // Set the rook
            board.bitboards[color] &= !(1u64 << rook_pos); // Remove the rook
            board.bitboards[ROOK] |= 1u64 << rook_n_pos; // Set the rook
            board.bitboards[ROOK] &= !(1u64 << rook_pos); // Remove the rook
        }

        if self.en_passant != 0 {
            if piece_type.is_uppercase() {
                let clear = !(1u64 << (4 * 8 + (self.from as i32 % 8) - self.en_passant));
                board.bitboards[7] &= clear; // clear the taken pawn
                board.bitboards[0] &= clear;
            } else {
                let clear = !(1u64 << (3 * 8 + (self.from as i32 % 8) - self.en_passant));
                board.bitboards[6] &= clear;
                board.bitboards[0] &= clear;
            }
        }

        // Set the color bitboards
        if piece_type.is_uppercase() {
            board.bitboards[6] &= !(1u64 << self.from);
            board.bitboards[6] |= 1u64 << self.to;
        } else {
            board.bitboards[7] &= !(1u64 << self.from);
            board.bitboards[7] |= 1u64 << self.to;
        }

        // board.print_board();
        // println!("Move made");

        if piece_type == 'p' {
            if self.from / 8 == 6 && self.to / 8 == 4 {
                self.from - 8
            } else {
                65
            }
        } else if piece_type == 'P' {
            if self.from / 8 == 1 && self.to / 8 == 3 {
                self.from + 8
            } else {
                65
            }
        } else {
            65
        }
    }

    pub fn unmake_move(&self, board: &mut Board) {
        let mut piece_type = find_piece_type(board, self.to);
        let mut piece_bitboard = get_bitboard_val(piece_type);
        // println!("Unmaking move");
        // board.print_board();

        if self.is_castle() {
            let (rook_pos, rook_n_pos, color) = self.get_castle_rook_pos();

            board.bitboards[ROOK] |= 1u64 << rook_pos; // Set the rook
            board.bitboards[color] |= 1u64 << rook_pos; // Set the rook
            board.bitboards[color] &= !(1u64 << rook_n_pos); // Remove the rook
            board.bitboards[ROOK] &= !(1u64 << rook_n_pos); // Remove the rook
        }

        if self.promotion_type != 'z' {
            board.bitboards[piece_bitboard] &= !(1u64 << self.to); // clear the promoted piece

            piece_type = if piece_type.is_uppercase() { 'P' } else { 'p' };
            piece_bitboard = 0;
        }


        if self.en_passant != 0 {
            if piece_type.is_uppercase() {
                let set = 1u64 << (4 * 8 + (self.from as i32 % 8) - self.en_passant);
                board.bitboards[7] |= set; // Add the taken pawn back in
                board.bitboards[0] |= set;
            } else {
                let set = 1u64 << (3 * 8 + (self.from as i32 % 8) - self.en_passant);
                board.bitboards[6] |= set;
                board.bitboards[0] |= set;
            }
        }

        board.bitboards[piece_bitboard] &= !(1u64 << self.to);
        board.bitboards[piece_bitboard] |= 1u64 << self.from;

        if piece_type.is_uppercase() {
            board.bitboards[6] &= !(1u64 << self.to);
            board.bitboards[6] |= 1u64 << self.from;
        } else {
            board.bitboards[7] &= !(1u64 << self.to);
            board.bitboards[7] |= 1u64 << self.from;
        }

        if self.capture != 'z' {
            board.bitboards[get_bitboard_val(self.capture)] |= 1u64 << self.to; // set the captured piece
            if self.capture.is_uppercase() {
                board.bitboards[6] |= 1u64 << self.to;
            } else {
                board.bitboards[7] |= 1u64 << self.to;
            }
        }
        // println!("After move {}{}", square_to_algebraic(self.from), square_to_algebraic(self.to));
        // board.print_board();
    }
}

#[derive(Clone, Copy)]
struct GeneratorBoard<'a> {
    pub board: &'a Board,
    pub white_attacks: u64,
    pub black_attacks: u64,
    pub white_king: u64,
    pub black_king: u64,
}

fn get_bitboard_val(piece_type: char) -> usize {
    match piece_type {
        'p' | 'P' => 0,
        'n' | 'N' => 1,
        'b' | 'B' => 2,
        'r' | 'R' => 3,
        'q' | 'Q' => 4,
        'k' | 'K' => 5,
        _ => { unreachable!("Invalid board"); }
    }
}

fn find_king(board: &Board, white: bool) -> u64 {
    let mut index = 0;
    for square in 0..64 {
        if board.bitboards[5] & (1u64 << square) != 0 {
            if white && (board.white_pieces() & (1u64 << square)) != 0 {
                index = square;
                break;
            }
            if !white && (board.black_pieces() & (1u64 << square)) != 0 {
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
                    2 => get_bishop_attacks(board.all_pieces(), square),
                    3 => get_rook_attacks(board.all_pieces(), square),
                    4 => get_queen_attacks(board.all_pieces(), square),
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
    // attacks.board.print_board();
    // print_bitboard(attacks.board.white_pieces(), 'W', '.');
    // print_bitboard(attacks.board.black_pieces(), 'B', '.');
    panic!("No piece on the given square {}", square_to_algebraic(square));
}

pub fn find_piece_type(board: &Board, square: u64) -> char {
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

fn count_check(board: &GeneratorBoard, white: bool, en_passant: u64) -> (bool, u64) {
    let mut possible_blocks: u64 = 0;
    let mut count: u64 = 0;
    for square in 0..64 {
        for piece in 0..6 {
            if (if white { board.board.black_pieces() } else { board.board.white_pieces() } & (1u64 << square)) != 0 &&
                board.board.bitboards[piece] & (1u64 << square) != 0 {
                let val = match piece {
                    0 => get_pawn_attacks(board.board, square),
                    1 => get_knight_attacks(square),
                    2 => get_bishop_attacks(board.board.all_pieces(), square),
                    3 => get_rook_attacks(board.board.all_pieces(), square),
                    4 => get_queen_attacks(board.board.all_pieces(), square),
                    5 => get_king_attacks(square),
                    _ => { unreachable!("Invalid board"); }
                };

                if ((1u64 << if white { board.white_king } else { board.black_king }) & val) != 0 {
                    if count == 1 {
                        return (true, 0); // double check
                    } else {
                        count = 1;

                        // Allow en passant capture
                        if piece == 0 && en_passant > 15 && en_passant < 48 {
                            if white {
                                if en_passant == square + 8 {
                                    possible_blocks |= 1u64 << en_passant; // allow en passant
                                }
                            } else if en_passant == square - 8 {
                                possible_blocks |= 1u64 << square;
                            }
                        }

                        possible_blocks |= 1u64 << square; // allow capture of the piece
                        possible_blocks |= match piece {
                            0 => 0,
                            1 => 0,
                            2 => get_between(board, square, if white { board.white_king } else { board.black_king }),
                            3 => get_between(board, square, if white { board.white_king } else { board.black_king }),
                            4 => get_between(board, square, if white { board.white_king } else { board.black_king }),
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

pub fn in_check(board: &Board, white: bool) -> bool {
    let king = if white { find_king(board, true) } else { find_king(board, false) };
    for square in 0..64 {
        for piece in 0..6 {
            if board.bitboards[if white { 6 } else { 7 }] & (1u64 << square) != 0 { continue; }
            if (board.bitboards[piece] & (1u64 << square)) != 0 &&
                ((1u64 << king) & match piece {
                    0 => get_pawn_attacks(board, square),
                    1 => get_knight_attacks(square),
                    2 => get_bishop_attacks(board.all_pieces(), square),
                    3 => get_rook_attacks(board.all_pieces(), square),
                    4 => get_queen_attacks(board.all_pieces(), square),
                    5 => get_king_attacks(square),
                    _ => { unreachable!("Invalid board"); }
                }) != 0 {
                return true;
            }
        }
    }

    false
}

// fn check_move(board: &mut Board, move_: &Move, white: bool) -> bool {
//     move_.make_move(board);
//     let val = in_check(board, white);
//     move_.unmake_move(board);
//
//     val
// }

pub fn get_moves(board: &Board, en_passant: u64, castle_rights: u32, white: bool) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    let mut att_board = get_attacks(board);
    att_board.white_king = find_king(board, true);
    att_board.black_king = find_king(board, false);

    let my_pieces = if white { board.white_pieces() } else { board.black_pieces() };
    let mut possible_squares = !0u64;
    let mut double_check = false;
    if (white && ((att_board.black_attacks & (1u64 << att_board.white_king)) != 0)) ||
        (!white && ((att_board.white_attacks & (1u64 << att_board.black_king)) != 0)) {
        // in check
        (double_check, possible_squares) = count_check(&att_board, white, en_passant);
        // board.print_board();
        // board::print_bitboard(possible_squares, 'X', '.');
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
                    en_passant: 0,
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
            (if find_piece_type(board, square) != king_type { possible_squares } else { !0u64 });

        if possible_moves == 0 { continue; } // No moves, no reason to loop through them
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
                            en_passant: 0,
                            capture,
                            castle_rights,
                        });
                        // if check_move(&mut check_board, &n_move, white) {
                        //     moves.push(n_move);
                        // }
                    }
                    break;
                } else {
                    moves.push(Move {
                        from: square,
                        to: pm_square,
                        promotion_type: 'z',
                        en_passant: is_en_passant(board, square, pm_square),
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
    if white {
        if check_castling(castle_rights, board.all_pieces(), CASTLE_WHITE_KING_SIDE,
                          WHITE_KING_SQUARE as i32, -1, att_board.black_attacks) {
            moves.push(Move {
                from: WHITE_KING_SQUARE,
                to: WHITE_KING_SQUARE - 2,
                promotion_type: 'z',
                en_passant: 0,
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
                en_passant: 0,
                capture: 'z',
                castle_rights,
            });
        }
    }
    if !white {
        if check_castling(castle_rights, board.all_pieces(), CASTLE_BLACK_KING_SIDE,
                          BLACK_KING_SQUARE as i32, -1, att_board.white_attacks) {
            moves.push(Move {
                from: BLACK_KING_SQUARE,
                to: BLACK_KING_SQUARE - 2,
                promotion_type: 'z',
                en_passant: 0,
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
                en_passant: 0,
                capture: 'z',
                castle_rights,
            });
        }
    }

    moves
}

fn is_en_passant(board: &Board, from: u64, to: u64) -> i32 {
    let piece_type = find_piece_type(board, from);
    if piece_type == 'p' {
        if from / 8 == 3 && to / 8 == 2 { // Capture
            if ((1u64 << (3 * 8 + (to % 8))) & board.bitboards[0]) != 0 && // Pawn on same rank
                ((1u64 << to) & board.all_pieces()) == 0 {
                return (from % 8) as i32 - (to % 8) as i32;
            }
        }
    } else if piece_type == 'P' && from / 8 == 4 && to / 8 == 5 &&
        ((1u64 << (4 * 8 + (to % 8))) & board.bitboards[0]) != 0 &&
        ((1u64 << to) & board.all_pieces()) == 0 { // No pieces where taking
        return (from % 8) as i32 - (to % 8) as i32;
    }

    0
}

fn piece_on_top_left(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 > 0) && (index / 8 < 8) && (index < 57) {
        index += 7;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_top_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king } as i64;

    let i_square = square as i64;
    if (i_square / 8 < king / 8 && i_square % 8 > king % 8) &&
        (king / 8 - i_square / 8 == i_square % 8 - king % 8) {
        piece_on_top_left(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_top_right(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 < 7) && (index / 8 < 7) && (index < 55) {
        index += 9;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_top_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king } as i64;
    let i_square = square as i64;
    if (i_square / 8 < king / 8 && i_square % 8 < king % 8) &&
        (king / 8 - i_square / 8 == king % 8 - i_square % 8) {
        piece_on_top_right(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_bottom_left(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 > 0) && (index / 8 > 0) && (index > 8) {
        index -= 9;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_bottom_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king = if white { board.white_king } else { board.black_king } as i64;

    let i_square = square as i64;
    if (i_square / 8 > king / 8 && i_square % 8 > king % 8) &&
        (i_square / 8 - king / 8 == i_square % 8 - king % 8) {
        piece_on_bottom_left(board, square, 1u64 << king)
    } else {
        false
    }
}

fn piece_on_bottom_right(board: &GeneratorBoard, square: u64, target_bitboard: u64) -> bool {
    let mut index = square;
    while (index % 8 < 7) && (index / 8 > 0) && (index > 6) {
        index -= 7;

        if (board.board.all_pieces() & (1u64 << index)) != 0 {
            return target_bitboard & (1u64 << index) != 0;
        }
    }

    false
}

fn is_king_bottom_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    let king: i64 = (if white { board.white_king } else { board.black_king }) as i64;

    let i_square = square as i64;
    if (i_square / 8 > king / 8 && i_square % 8 < king % 8) &&
        (i_square / 8 - king / 8 == king % 8 - i_square % 8) {
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

    while loc / 8 > 0 && loc % 8 < 7 && loc > 6 { // Bottom Right
        loc -= 7;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc / 8 < 7 && loc % 8 > 0 && loc < 58 { // Top Left
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

    while loc / 8 < 7 && loc % 8 < 7 && loc < 56 { // Top Right
        loc += 9;
        moves |= if my_pieces & (1u64 << loc) == 0 { 1u64 << loc } else { 0 };

        if board.all_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc / 8 > 0 && loc % 8 > 0 && loc > 8 { // Bottom Left
        loc -= 9;
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

fn check_en_passant(board: &Board, square: u64, en_passant: u64) -> u64 {
    if en_passant > 63 || en_passant == 0 {
        return 0;
    }
    let mut board_cp: Board = board.clone();
    let white = board.white_pieces() & (1u64 << square) != 0;

    let clear = !(1u64 << (en_passant as i32 + if white { -8 } else { 8 }));
    board_cp.bitboards[if white { 7 } else { 6 }] &= clear; // clear the taken pawn
    board_cp.bitboards[0] &= clear;

    board_cp.bitboards[if white { 6 } else { 7 }] |= 1u64 << en_passant; // set the new square
    board_cp.bitboards[0] |= 1u64 << en_passant;
    // println!("{}", square_to_algebraic(square));
    board_cp.bitboards[if white { 6 } else { 7 }] &= !(1u64 << square); // clear the old square
    board_cp.bitboards[0] &= !(1u64 << square);

    if in_check(&board_cp, white) {
        // println!("In check");
        0
    } else {
        1u64 << en_passant
    }
}

fn get_pawn_moves(board: GeneratorBoard, square: u64, en_passant: u64) -> u64 {
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let target_pieces = if white { board.board.black_pieces() } else { board.board.white_pieces() };
    let target_bishop = (board.board.bitboards[BISHOP] | board.board.bitboards[QUEEN]) & target_pieces;
    let target_rook = (board.board.bitboards[ROOK] | board.board.bitboards[QUEEN]) & target_pieces;

    let passant_square = if square > 15 && square < 48 { check_en_passant(board.board, square, en_passant) } else { 0 };

    fn right_attacks(board: &Board, square: u64, en_passant: u64) -> u64 {
        if board.white_pieces() & (1u64 << square) != 0 {
            if square % 8 < 7 {
                (1u64 << (square + 9)) & (board.black_pieces() | en_passant)
            } else { 0 }
        } else if square % 8 < 7 {
            (1u64 << (square - 9)) & (board.white_pieces() | en_passant)
        } else { 0 }
    }

    fn left_attacks(board: &Board, square: u64, en_passant: u64) -> u64 {
        if board.white_pieces() & (1u64 << square) != 0 {
            if square % 8 > 0 {
                (1u64 << (square + 7)) & (board.black_pieces() | en_passant)
            } else { 0 }
        } else if square % 8 > 0 {
            (1u64 << (square - 7)) & (board.white_pieces() | en_passant)
        } else { 0 }
    }

    fn forward_moves(board: &Board, square: u64) -> u64 {
        let mut moves: u64 = 0;
        if board.white_pieces() & (1u64 << square) != 0 {
            if (1u64 << (square + 8)) & board.all_pieces() == 0 {
                moves |= 1u64 << (square + 8);
                if square / 8 == 1 && board.all_pieces() & (1u64 << (square + 16)) == 0 {
                    moves |= 1u64 << (square + 16); // can move double
                }
            }
        } else if (1u64 << (square - 8)) & board.all_pieces() == 0 {
            moves |= 1u64 << (square - 8);
            if square / 8 == 6 && board.all_pieces() & (1u64 << (square - 16)) == 0 {
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
            left_attacks(board.board, square, passant_square)
        } else { all_pawn_moves(board, square, passant_square) }
    } else if is_king_top_right(&board, square, white) {
        if piece_on_bottom_left(&board, square, target_bishop) {
            right_attacks(board.board, square, passant_square)
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_bottom_left(&board, square, white) {
        if piece_on_top_right(&board, square, target_bishop) {
            right_attacks(board.board, square, passant_square)
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_bottom_right(&board, square, white) {
        if piece_on_top_left(&board, square, target_bishop) {
            left_attacks(board.board, square, passant_square)
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_left(&board, square, white) {
        if piece_on_right(&board, square, target_rook) {
            0 // no moves
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_right(&board, square, white) {
        if piece_on_left(&board, square, target_rook) {
            0 // no moves
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_top(&board, square, white) {
        if piece_on_bottom(&board, square, target_rook) {
            forward_moves(board.board, square)
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else if is_king_bottom(&board, square, white) {
        if piece_on_top(&board, square, target_rook) {
            forward_moves(board.board, square)
        } else {
            all_pawn_moves(board, square, passant_square)
        }
    } else {
        all_pawn_moves(board, square, passant_square)
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
    let white = board.board.white_pieces() & (1u64 << square) != 0;
    let my_pieces = if white { board.board.white_pieces() } else { board.board.black_pieces() };
    let pieces = board.board.all_pieces() & (!(board.board.bitboards[5] & my_pieces));
    let mut attacks = 0;
    for square in 0..64 {
        for piece in 0..6 {
            if board.board.bitboards[piece] & (1u64 << square) != 0 &&
                board.board.bitboards[if white { 7 } else { 6 }] & (1u64 << square) != 0 {
                let val = match piece {
                    0 => get_pawn_attacks(board.board, square),
                    1 => get_knight_attacks(square),
                    2 => get_bishop_attacks(pieces, square),
                    3 => get_rook_attacks(pieces, square),
                    4 => get_queen_attacks(pieces, square),
                    5 => get_king_attacks(square),
                    _ => { unreachable!("Invalid board"); }
                };

                attacks |= val;

                break; // Piece found, no need to check other types
            }
        }
    }

    get_king_attacks(square) & (!if white { board.board.white_pieces() } else { board.board.black_pieces() }) & (!attacks)
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


fn get_rook_attacks(pieces: u64, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc < 56 { // Top
        loc += 8;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 { // Bottom
        loc -= 8;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 > 0 { // Right
        loc -= 1;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 < 7 { // Left
        loc += 1;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    attacks
}

fn get_bishop_attacks(pieces: u64, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc > 7 && loc % 8 > 0 && loc > 8 { // Bottom Right
        loc -= 9;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 && loc % 8 < 7 && loc > 6 { // Bottom Left
        loc -= 7;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 > 0 && loc < 58 { // Top Left
        loc += 7;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 < 7 && loc < 56 { // Top Right
        loc += 9;
        attacks |= 1u64 << loc;

        if pieces & (1u64 << loc) != 0 { break; }
    }

    attacks
}


fn get_queen_attacks(pieces: u64, square: u64) -> u64 {
    get_bishop_attacks(pieces, square) | get_rook_attacks(pieces, square) // ez
}

fn get_king_attacks(square: u64) -> u64 {
    let mut attacks: u64 = if square < 56 { 1u64 << (square + 8) } else { 0 } |
        if square > 7 { 1u64 << (square - 8) } else { 0 }; // Top and Bottom

    if square % 8 < 7 { // Left
        attacks |= if square < 55 { 1u64 << (square + 9) } else { 0 };
        attacks |= if square < 64 { 1u64 << (square + 1) } else { 0 };
        attacks |= if square > 6 { 1u64 << (square - 7) } else { 0 };
    }
    if square % 8 > 0 { // Right
        attacks |= if square < 57 { 1u64 << (square + 7) } else { 0 };
        attacks |= if square > 0 { 1u64 << (square - 1) } else { 0 };
        attacks |= if square > 8 { 1u64 << (square - 9) } else { 0 };
    }

    attacks
}

fn get_between(board: &GeneratorBoard, start: u64, end: u64) -> u64 {
    let mut between: u64 = 0;
    let mut loc: u64 = start;
    let target = 1u64 << end;

    if piece_on_top_left(board, start, target) {
        while loc / 8 < 7 && loc % 8 > 0 && loc < 58 { // Top Left
            loc += 7;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_top_right(board, start, target) {
        while loc / 8 < 7 && loc % 8 < 7 && loc < 56 { // Top Right
            loc += 9;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_bottom_left(board, start, target) {
        while loc / 8 > 0 && loc % 8 > 0 && loc > 8 { // Bottom Left
            loc -= 9;

            if board.board.all_pieces() & (1u64 << loc) != 0 { break; }
            between |= 1u64 << loc;
        }
    } else if piece_on_bottom_right(board, start, target) {
        while loc / 8 > 0 && loc % 8 < 7 && loc > 6 { // Bottom Right
            loc -= 7;

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
        board.board.print_board();
        panic!("No way to get between squares {} and {}", square_to_algebraic(start), square_to_algebraic(end));
    }

    between
}