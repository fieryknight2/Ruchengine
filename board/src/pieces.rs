use crate::board::Board;

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

#[derive(Clone, Copy)]
struct GeneratorBoard<'a> {
    pub board: &'a Board,
    pub white_attacks: u64,
    pub black_attacks: u64,
    pub white_king: u64,
    pub black_king: u64,
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

pub fn get_moves(board: &Board, en_passant: u64, castle_rights: u32) -> Vec<Move> {
    let mut moves: Vec<Move> = Vec::new();

    let mut att_board = get_attacks(board);
    att_board.white_king = find_king(board, true);
    att_board.black_king = find_king(board, false);

    // Normal moves
    for square in 0..64 {
        if board.all_pieces() & (1u64 << square) == 0 { continue; }
        let possible_moves = get_piece_moves_wa(&att_board, square, en_passant);

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
        return piece_on_top_left(board, square, 1u64 << king);
    }

    false
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
        return piece_on_top_right(board, square, 1u64 << king);
    }

    false
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
        return piece_on_bottom_left(board, square, 1u64 << king);
    }

    false
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
        return piece_on_bottom_right(board, square, 1u64 << king);
    }

    false
}

fn is_king_left(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    false
}

fn is_king_right(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    false
}

fn is_king_top(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    false
}

fn is_king_bottom(board: &GeneratorBoard, square: u64, white: bool) -> bool {
    false
}

fn all_pawn_moves(board: GeneratorBoard, square: u64, en_passant: u64) -> u64 {
    let mut moves: u64 = 0;

    if board.board.white_pieces() & (1u64 << square) != 0 {
        // Get captures
        moves |= get_pawn_attacks(board.board, square) & (board.board.black_pieces() & (1u64 << en_passant));

        if (1u64 << (square + 8)) & board.board.all_pieces() == 0 {
            moves |= 1u64 << (square + 8);
            if square % 8 == 1 && board.board.all_pieces() & (1u64 << (square + 16)) == 0 {
                moves |= 1u64 << (square + 16); // can move double
            }
        }
    } else {
        moves |= get_pawn_attacks(board.board, square) & (board.board.white_pieces() & (1u64 << en_passant));

        if (1u64 << (square - 8)) & board.board.all_pieces() == 0 {
            moves |= 1u64 << (square - 8);
            if square % 8 == 7 && board.board.all_pieces() & (1u64 << (square - 16)) == 0 {
                moves |= 1u64 << (square - 16); // can move double
            }
        }
    }

    moves
}

fn all_knight_moves(board: GeneratorBoard, square: u64) -> u64 {
    if board.board.white_pieces() & (1u64 << square) != 0 {
        get_knight_attacks(square) & (!board.board.white_pieces()) // Don't move onto own pieces
    } else {
        get_knight_attacks(square) & (!board.board.black_pieces())
    }
}

fn all_bishop_moves(board: GeneratorBoard, square: u64) -> u64 {
    if board.board.white_pieces() & (1u64 << square) != 0 {
        get_bishop_attacks(board.board, square) & (!board.board.white_pieces()) // Don't move onto own pieces
    } else {
        get_bishop_attacks(board.board, square) & (!board.board.black_pieces())
    }
}

fn all_rook_moves(board: GeneratorBoard, square: u64) -> u64 {
    if board.board.white_pieces() & (1u64 << square) != 0 {
        get_rook_attacks(board.board, square) & (!board.board.white_pieces()) // Don't move onto own pieces
    } else {
        get_rook_attacks(board.board, square) & (!board.board.black_pieces())
    }
}

fn all_queen_moves(board: GeneratorBoard, square: u64) -> u64 {
    all_bishop_moves(board, square) | all_rook_moves(board, square)
}

fn get_pawn_moves(board: GeneratorBoard, square: u64, en_passant: u64) -> u64 {
    let mut moves: u64 = 0;

    all_pawn_moves(board, square, en_passant)
}

fn get_knight_moves(board: GeneratorBoard, square: u64) -> u64 {
    let mut moves: u64 = 0;

    all_knight_moves(board, square) // moves
}

fn get_bishop_moves(board: GeneratorBoard, square: u64) -> u64 {
    let mut moves: u64 = 0;

    all_bishop_moves(board, square) // moves
}

fn get_rook_moves(board: GeneratorBoard, square: u64) -> u64 {
    let mut moves: u64 = 0;

    all_rook_moves(board, square) // moves
}

fn get_queen_moves(board: GeneratorBoard, square: u64) -> u64 {
    let mut moves: u64 = 0;

    all_queen_moves(board, square) // moves
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
        attacks |= (if square % 8 > 0 { 1u64 << (square - 17) } else { 0 } |
            if square % 8 < 7 { 1u64 << (square - 15) } else { 0 })
    }

    if square / 8 > 0 {
        attacks |= (if square % 8 > 1 { 1u64 << (square - 10) } else { 0 } |
            if square % 8 < 6 { 1u64 << (square - 6) } else { 0 })
    }

    if square / 8 < 7 {
        attacks |= (if square % 8 > 1 { 1u64 << (square + 6) } else { 0 } |
            if square % 8 < 6 { 1u64 << (square + 10) } else { 0 })
    }

    if square / 8 < 6 {
        attacks |= (if square % 8 > 0 { 1u64 << (square + 15) } else { 0 } |
            if square % 8 < 7 { 1u64 << (square + 17) } else { 0 })
    }

    attacks
}

fn get_bishop_attacks(board: &Board, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc > 7 && loc % 8 > 0 { // Bottom Right
        loc -= 9;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 && loc % 8 < 7 { // Bottom Left
        loc -= 7;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 > 0 { // Top Left
        loc += 7;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc < 57 && loc % 8 < 7 { // Top Right
        loc += 9;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    attacks
}

fn get_rook_attacks(board: &Board, square: u64) -> u64 {
    let mut attacks: u64 = 0;
    let mut loc: u64 = square;

    while loc < 56 { // Top
        loc += 8;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc > 7 { // Bottom
        loc -= 8;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 > 0 { // Right
        loc -= 1;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    loc = square;
    while loc % 8 < 7 { // Left
        loc += 1;
        attacks |= 1u64 << loc;

        if board.white_pieces() & board.black_pieces() & (1u64 << loc) != 0 { break; }
    }

    attacks
}

fn get_queen_attacks(board: &Board, square: u64) -> u64 {
    get_bishop_attacks(board, square) | get_rook_attacks(board, square) // ez
}

fn get_king_attacks(square: u64) -> u64 {
    let mut attacks: u64 = (1u64 << (square + 8)) | (1u64 << (square - 8)); // Top and Bottom

    if square % 8 < 7 { // Left
        attacks |= 1u64 << (square + 9);
        attacks |= 1u64 << (square + 1);
        attacks |= 1u64 << (square - 7);
    }
    if square % 8 > 0 { // Right
        attacks |= 1u64 << (square + 7);
        attacks |= 1u64 << (square - 1);
        attacks |= 1u64 << (square - 9);
    }

    attacks
}