use std::sync::mpsc;
use crate::board::square_to_algebraic;
// use crate::moves::{CASTLE_BLACK_KING_SIDE, CASTLE_BLACK_QUEEN_SIDE, CASTLE_WHITE_KING_SIDE, CASTLE_WHITE_QUEEN_SIDE};

pub mod board;
pub mod moves;

use std::thread; // Speed up perft

pub struct PerftStats {
    pub capture_count: u64,
    pub promotion_count: u64,
    pub check_count: u64,
    pub total_count: u64,
    pub castle_count: u64,
    pub en_passant_count: u64,
}

pub fn count_moves(board: &mut board::Board, castle_rights: u32, white: bool, mut en_passant: u64, depth: u32) -> PerftStats {
    let mut perft = PerftStats {
        capture_count: 0,
        promotion_count: 0,
        check_count: 0,
        total_count: 0,
        castle_count: 0,
        en_passant_count: 0,
    };

    if en_passant > 63 || en_passant == 0 {
        en_passant = 65;
    }

    let move_list = moves::get_moves(board, en_passant, castle_rights, white);
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let (tx, rx) = mpsc::channel();
    if depth != 0 {
        for move_ in move_list {
            let mut board = board.clone();
            let tx = tx.clone();
            handles.push(thread::spawn(move || {
                let mut perft = PerftStats {
                    capture_count: 0,
                    promotion_count: 0,
                    check_count: 0,
                    total_count: 0,
                    castle_count: 0,
                    en_passant_count: 0,
                };
                let result = recursive_count_moves(&mut board, move_, !white, depth - 1);
                perft.capture_count += result.capture_count;
                perft.promotion_count += result.promotion_count;
                perft.check_count += result.check_count;
                perft.total_count += result.total_count;
                perft.castle_count += result.castle_count;
                perft.en_passant_count += result.en_passant_count;
                if move_.promotion_type != 'z' {
                    println!("{}{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), move_.promotion_type, result.total_count);
                } else {
                    println!("{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), result.total_count);
                }
                tx.send(perft).unwrap();
            }));
        }

        for handle in handles {
            let val = rx.recv().unwrap();
            perft.capture_count += val.capture_count;
            perft.promotion_count += val.promotion_count;
            perft.check_count += val.check_count;
            perft.total_count += val.total_count;
            perft.castle_count += val.castle_count;
            perft.en_passant_count += val.en_passant_count;
            handle.join().unwrap(); // wait for all threads to finish
        }
    } else {
        perft.total_count = move_list.len() as u64;
        for move_ in move_list {
            let piece_type = moves::find_piece_type(board, move_.from);
            move_.make_move(board);

            // Count information
            if move_.capture != 'z' {
                perft.capture_count += 1;
            }
            if move_.promotion_type != 'z' { perft.promotion_count += 1; }
            perft.check_count += if moves::in_check(board, !white) { 1 } else { 0 };
            if piece_type == 'k' || piece_type == 'K' {
                let val = move_.from as i32 - move_.to as i32;
                if val == 2 || val == -2 { // king moves two squares
                    perft.castle_count += 1;
                }
            }
            perft.en_passant_count += if move_.en_passant != 0 { 1 } else { 0 };

            move_.unmake_move(board);
            if check_overlap(board) {
                println!("Piece overlap caused by move {}{}", square_to_algebraic(move_.from), square_to_algebraic(move_.to));
            }
            if move_.promotion_type != 'z' {
                println!("{}{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), move_.promotion_type, 1);
            } else {
                println!("{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), 1);
            }
        }
    }

    perft
}

fn check_overlap(board: &board::Board) -> bool {
    for piece in 0..6 {
        for alt in 0..6 {
            if piece == alt { continue; }
            if board.bitboards[piece] & board.bitboards[alt] != 0 {
                println!("Piece {} and {} overlap", piece, alt);
                board::print_bitboard(board.bitboards[piece], usize::to_string(&piece).chars().next().unwrap(), '.');
                board::print_bitboard(board.bitboards[alt], usize::to_string(&alt).chars().next().unwrap(), '.');
                return true;
            }
        }
    }

    false
}

// fn check_missing(board: &board::Board) -> bool {
//     for piece in 0..6 {
//         let white = (!board.black_pieces()) & board.bitboards[piece];
//         let black = (!board.white_pieces()) & board.bitboards[piece];
//
//         // board::print_bitboard(white, 'W', '.');
//         // board::print_bitboard(black, 'B', '.');
//
//         if (white & (!board.white_pieces())) | (black & (!board.black_pieces())) != 0 {
//             return true;
//         }
//     }
//
//     if (board.white_pieces() & board.black_pieces()) != 0 {
//         println!("WHITE AND BLACK PIECES ON TOP OF EACH OTHER");
//     }
//
//     false
// }

fn recursive_count_moves(board: &mut board::Board, move_: moves::Move, white: bool, depth: u32) -> PerftStats {
    let mut perft = PerftStats {
        capture_count: 0,
        promotion_count: 0,
        check_count: 0,
        total_count: 0,
        castle_count: 0,
        en_passant_count: 0,
    };

    let piece_type = moves::find_piece_type(board, move_.from);
    // let mut ncastle_rights = castle_rights;

    // match piece_type { // Deal with castling rights
    //     'r' => {
    //         if move_.from == 56 {
    //             ncastle_rights &= !CASTLE_BLACK_QUEEN_SIDE;
    //         } else if move_.from == 63 {
    //             ncastle_rights &= !CASTLE_BLACK_KING_SIDE;
    //         }
    //     }
    //     'R' => {
    //         if move_.from == 0 {
    //             ncastle_rights &= !CASTLE_WHITE_QUEEN_SIDE;
    //         } else if move_.from == 7 {
    //             ncastle_rights &= !CASTLE_WHITE_KING_SIDE;
    //         }
    //     }
    //     'k' => {
    //         ncastle_rights &= !(CASTLE_BLACK_KING_SIDE | CASTLE_BLACK_QUEEN_SIDE);
    //     }
    //     'K' => {
    //         ncastle_rights &= !(CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE);
    //     }
    //     _ => {}
    // }

    let (en_passant, castle_rights) = move_.make_move(board);
    let move_list = moves::get_moves(board, en_passant, castle_rights, white);
    perft.check_count += if moves::in_check(board, !white) { 1 } else { 0 };
    if piece_type == 'k' || piece_type == 'K' {
        let val = move_.from as i32 - move_.to as i32;
        if val == 2 || val == -2 { // king moves two squares
            perft.castle_count += 1;
        }
    }

    // print!("{{");
    for n_move in move_list {
        // print!("{}{}, ", square_to_algebraic(n_move.from), square_to_algebraic(n_move.to));
        if depth == 0 {
            if n_move.capture != 'z' {
                perft.capture_count += 1;
            }
            if n_move.promotion_type != 'z' { perft.promotion_count += 1; }
            perft.total_count += 1;
            perft.en_passant_count += if n_move.en_passant != 0 { 1 } else { 0 };
            perft.check_count += if n_move.en_passant != 0 { 1 } else { 0 };
            // println!("Before {}{} {},{}", square_to_algebraic(n_move.from), square_to_algebraic(n_move.to), n_move.from, n_move.to);
            // if n_move.is_castle() {
            //     board.print_board();
            // }
            n_move.make_move(board);
            perft.check_count += if moves::in_check(board, !white) { 1 } else { 0 };
            n_move.unmake_move(board);
            // println!("After");
            // board.print_board();
            if check_overlap(board) {
                board.print_board();
                println!("Piece overlap caused by move {}{}", square_to_algebraic(n_move.from), square_to_algebraic(n_move.to));
                panic!("Piece overlap emergency quit")
            }
            // if check_missing(board) {
            //     board.print_board();
            //     println!("Piece missing caused by move {}{}", square_to_algebraic(n_move.from), square_to_algebraic(n_move.to));
            //     panic!("Piece missing emergency quit")
            // }
        } else {
            let result = recursive_count_moves(board, n_move, !white, depth - 1);
            perft.capture_count += result.capture_count;
            perft.promotion_count += result.promotion_count;
            perft.total_count += result.total_count;
            perft.check_count += result.check_count;
            perft.castle_count += result.castle_count;
            perft.en_passant_count += result.en_passant_count;
        }
    }
    // println!("}}");
    move_.unmake_move(board);

    perft
}

pub fn count_moves_no_threads(board: &mut board::Board, castle_rights: u32, white: bool, mut en_passant: u64, depth: u32) -> PerftStats {
    let mut perft = PerftStats {
        capture_count: 0,
        promotion_count: 0,
        check_count: 0,
        total_count: 0,
        castle_count: 0,
        en_passant_count: 0,
    };

    if en_passant > 63 || en_passant == 0 {
        en_passant = 65;
    }

    let move_list = moves::get_moves(board, en_passant, castle_rights, white);
    if depth != 0 {
        for move_ in move_list {
            let result = recursive_count_moves(board, move_, !white, depth - 1);
            perft.capture_count += result.capture_count;
            perft.promotion_count += result.promotion_count;
            perft.check_count += result.check_count;
            perft.total_count += result.total_count;
            perft.castle_count += result.castle_count;
            perft.en_passant_count += result.en_passant_count;
            if move_.promotion_type != 'z' {
                println!("{}{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), move_.promotion_type, result.total_count);
            } else {
                println!("{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), result.total_count);
            }
        }
    } else {
        perft.total_count = move_list.len() as u64;
        for move_ in move_list {
            let piece_type = moves::find_piece_type(board, move_.from);
            move_.make_move(board);

            // Count information
            if move_.capture != 'z' {
                perft.capture_count += 1;
            }
            if move_.promotion_type != 'z' { perft.promotion_count += 1; }
            perft.check_count += if moves::in_check(board, !white) { 1 } else { 0 };
            if piece_type == 'k' || piece_type == 'K' {
                let val = move_.from as i32 - move_.to as i32;
                if val == 2 || val == -2 { // king moves two squares
                    perft.castle_count += 1;
                }
            }
            perft.en_passant_count += if move_.en_passant != 0 { 1 } else { 0 };

            move_.unmake_move(board);
            if check_overlap(board) {
                println!("Piece overlap caused by move {}{}", square_to_algebraic(move_.from), square_to_algebraic(move_.to));
            }
            if move_.promotion_type != 'z' {
                println!("{}{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), move_.promotion_type, 1);
            } else {
                println!("{}{}: {}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), 1);
            }
        }
    }

    perft
}

#[cfg(test)]
mod tests {
    use crate::{board, count_moves};

    fn test(string: &str, depth: u32, expected: u64, castle_rights: u32, white: bool, en_passant: &str) {
        let mut board = board::create_board_from_string(string);
        let perft = count_moves(&mut board, castle_rights, white, if en_passant.len() == 2 { board::square_from_algebraic(en_passant) } else { 0 }, depth);
        println!("Promotion Count: {}, Capture Count: {}, Check Count: {}, Castle Count {}, En Passant Count {}",
                 perft.promotion_count, perft.capture_count, perft.check_count, perft.castle_count, perft.en_passant_count);
        println!("Total Count: {}", perft.total_count);
        if expected != 0 {
            assert_eq!(perft.total_count, expected);
        }
    }

    #[test]
    fn test_3() {
        // test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8", 4, 674_624, 0, true, "");
        test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8", 5, 11_030_083, 0, true, "");
    }

    #[test]
    fn test_2() {
        test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R", 3, 4_085_603, 0b1111, true, "");
        test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R", 5, 8_031_647_685, 0b1111, true, "");
    }

    #[test]
    fn test_4() {
        test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1", 3, 422_333, 0b1100, true, "");
        test("r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1", 5, 706_045_033, 0b1100, true, "");
    }

    #[test]
    fn test_board() {
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0, 20, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 1, 400, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 2, 8_902, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 3, 197_281, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 4, 4_865_609, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 5, 119_060_324, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 6, 3_195_901_860, 0b1111, true, "");
        test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 7, 84_998_978_956, 0b1111, true, "");
    }

    #[test]
    fn test_perft_5() {
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 0, 44, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 1, 1_486, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 2, 62_379, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 3, 2_103_487, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 4, 89_941_194, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
    }

    #[test]
    fn test_board_temp() {
        test("8/2p5/3p4/KP5r/5p1k/8/4P1P1/1R6", 4, 1_160_678, 0, false, "");
        // test("8/2p5/3p4/KP5r/7k/5p2/4P1P1/1R6", 3, 83_090, 0, true, "");
        // test("8/2p5/3p4/KP5r/7k/5p2/4P1P1/6R1", 2, 4_546, 0, false, "");
        // test("8/2p5/3p4/KP5r/6k1/5p2/4P1P1/6R1", 1, 313, 0, true, "");
        // test("8/2p5/3p4/KP5r/6k1/5P2/4P3/6R1", 0, 4, 0, false, "");
    }
}