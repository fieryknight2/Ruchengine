use board::square_to_algebraic;
use std::sync::mpsc;

pub mod board;
pub mod moves;

use std::thread; // Speed up perft

pub fn print_move(move_: moves::Move) {
    if move_.promotion_type != 'z' {
        println!(
            "{}{}{}",
            square_to_algebraic(move_.from),
            square_to_algebraic(move_.to),
            move_.promotion_type
        );
    } else {
        println!(
            "{}{}",
            square_to_algebraic(move_.from),
            square_to_algebraic(move_.to)
        );
    }
}

fn print_divide(move_: moves::Move, value: u64) {
    if move_.promotion_type != 'z' {
        println!(
            "{}{}{}: {}",
            square_to_algebraic(move_.from),
            square_to_algebraic(move_.to),
            move_.promotion_type,
            value
        );
    } else {
        println!(
            "{}{}: {}",
            square_to_algebraic(move_.from),
            square_to_algebraic(move_.to),
            value
        );
    }
}

pub fn count_moves(
    board: &mut board::Board,
    castle_rights: u32,
    white: bool,
    mut en_passant: u64,
    depth: u32,
) -> u64 {
    let mut perft = 0;
    if en_passant > 63 || en_passant == 0 {
        en_passant = 65;
    }

    let move_list = moves::get_moves(board, en_passant, castle_rights, white);
    let mut handles: Vec<thread::JoinHandle<()>> = Vec::new();
    let mut recievers = Vec::new();
    if depth != 0 {
        for move_ in move_list {
            let mut board = board.clone();
            let (tx, rx) = mpsc::channel();
            recievers.push(rx);
            handles.push(thread::spawn(move || {
                let result = recursive_count_moves(&mut board, move_, !white, depth - 1);
                print_divide(move_, result);
                tx.send(result).unwrap();
            }));
        }

        for (index, handle) in handles.into_iter().enumerate() {
            let val = recievers[index].recv().unwrap();
            perft += val;
            handle.join().unwrap(); // wait for all threads to finish
        }
    } else {
        perft = move_list.len() as u64;
        for move_ in move_list {
            print_divide(move_, 1); // print_move(move_); // print_divide(move_, 1);
        }
    }

    perft
}

fn recursive_count_moves(
    board: &mut board::Board,
    move_: moves::Move,
    white: bool,
    depth: u32,
) -> u64 {
    let mut perft = 0;

    // Make the move
    let (en_passant, castle_rights) = move_.make_move(board);
    // Discover the possible moves
    let move_list = moves::get_moves(board, en_passant, castle_rights, white);

    for n_move in move_list {
        if depth == 0 {
            perft += 1;
        } else {
            perft += recursive_count_moves(board, n_move, !white, depth - 1);
        }
    }
    // Unmake the move
    move_.unmake_move(board);

    perft
}

pub fn count_moves_no_threads(
    board: &mut board::Board,
    castle_rights: u32,
    white: bool,
    mut en_passant: u64,
    depth: u32,
) -> u64 {
    let mut perft = 0;

    if en_passant > 63 || en_passant == 0 {
        en_passant = 65;
    }

    let move_list = moves::get_moves(board, en_passant, castle_rights, white);
    if depth != 0 {
        for move_ in move_list {
            let result = recursive_count_moves(board, move_, !white, depth - 1);
            print_divide(move_, result);
            perft += result;
        }
    } else {
        perft = move_list.len() as u64;
        for move_ in move_list {
            print_move(move_); // print_divide(move_, 1);
        }
    }

    perft
}

#[cfg(test)]
mod tests {
    use crate::{board, count_moves};

    fn test(
        string: &str,
        depth: u32,
        expected: u64,
        castle_rights: u32,
        white: bool,
        en_passant: &str,
    ) {
        let mut board = board::create_board_from_string(string);
        let perft = count_moves(
            &mut board,
            castle_rights,
            white,
            if en_passant.len() == 2 {
                board::square_from_algebraic(en_passant)
            } else {
                0
            },
            depth,
        );
        println!("Total Count: {}", perft);
        if expected != 0 {
            assert_eq!(perft, expected);
        }
    }

    #[test]
    fn test_3() {
        // test("8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8", 4, 674_624, 0, true, "");
        test(
            "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8",
            5,
            11_030_083,
            0,
            true,
            "",
        );
    }

    #[test]
    fn test_2() {
        test(
            "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R",
            3,
            4_085_603,
            0b1111,
            true,
            "",
        );
        // test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R", 5, 8_031_647_685, 0b1111, true, "");
    }

    #[test]
    fn test_4() {
        test(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1",
            3,
            422_333,
            0b1100,
            true,
            "",
        );
        test(
            "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1",
            5,
            706_045_033,
            0b1100,
            true,
            "",
        );
    }

    #[test]
    fn test_board() {
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            0,
            20,
            0b1111,
            true,
            "",
        );
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            1,
            400,
            0b1111,
            true,
            "",
        );
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            2,
            8_902,
            0b1111,
            true,
            "",
        );
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            3,
            197_281,
            0b1111,
            true,
            "",
        );
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            4,
            4_865_609,
            0b1111,
            true,
            "",
        );
        test(
            "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR",
            5,
            119_060_324,
            0b1111,
            true,
            "",
        );
        // test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 6, 3_195_901_860, 0b1111, true, "");
        // test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 7, 84_998_978_956, 0b1111, true, "");
    }

    #[test]
    fn test_perft_5() {
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 0, 44, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 1, 1_486, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 2, 62_379, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        // test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 3, 2_103_487, CASTLE_WHITE_KING_SIDE | CASTLE_WHITE_QUEEN_SIDE, true, "");
        test(
            "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R",
            4,
            89_941_194,
            0b0011,
            true,
            "",
        );
    }

    #[test]
    fn test_board_temp() {
        test(
            "8/2p5/3p4/KP5r/5p1k/8/4P1P1/1R6",
            4,
            1_160_678,
            0,
            false,
            "",
        );
        // test("8/2p5/3p4/KP5r/7k/5p2/4P1P1/1R6", 3, 83_090, 0, true, "");
        // test("8/2p5/3p4/KP5r/7k/5p2/4P1P1/6R1", 2, 4_546, 0, false, "");
        // test("8/2p5/3p4/KP5r/6k1/5p2/4P1P1/6R1", 1, 313, 0, true, "");
        // test("8/2p5/3p4/KP5r/6k1/5P2/4P3/6R1", 0, 4, 0, false, "");
    }

    #[test]
    fn test_broken() {
        test("2Q3nr/pp1pk3/7p/1q2B1p1/8/2P3P1/P1P3BP/1K1R4", 3, 53491, 0, true, "");
        // test("2Q3nr/pp1pk3/7p/1q2B1p1/8/2P3P1/P1P3BP/K2R4", 2, 27113, 0, false, "");
        // test("2Q3nr/pp1p4/4k2p/1q2B1p1/8/2P3P1/P1P3BP/K2R4", 1, 1094, 0, true, "");
        // test("2Q3nr/pp1p4/2B1k2p/1q2B1p1/8/2P3P1/P1P4P/K2R4", 0, 29, 0, false, "");
    }
}
