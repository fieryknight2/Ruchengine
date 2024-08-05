use std::time::Instant;
use board::board::{print_bitboard, square_from_algebraic, square_to_algebraic};
use board::count_moves;
use board::count_moves_no_threads;

fn test(string: &str, depth: u32, expected: u64, castle_rights: u32, white: bool, en_passant: &str) {
    let mut board = board::board::create_board_from_string(string);
    let perft = count_moves(&mut board, castle_rights, white, if en_passant.len() == 2 { square_from_algebraic(en_passant) } else { 0 }, depth);
    println!("Total Count: {}", perft);
    if expected != 0 {
        assert_eq!(perft, expected);
    }
}

fn test_no_thread(string: &str, depth: u32, expected: u64, castle_rights: u32, white: bool, en_passant: &str) {
    let mut board = board::board::create_board_from_string(string);
    let perft = count_moves_no_threads(&mut board, castle_rights, white, if en_passant.len() == 2 { square_from_algebraic(en_passant) } else { 0 }, depth);
    println!("Total Count: {}", perft);
    if expected != 0 {
        assert_eq!(perft, expected);
    }
}

fn main() {
    println!("Hello, world!");
    println!("{}, {}", square_to_algebraic(4), square_to_algebraic(22));
    print_bitboard(1073741824, 'X', '.');
    print_bitboard(9331893833473007105, 'X', '.');
    println!("b1 {} c1 {} h1 {} h8 {}", square_from_algebraic("b1"),
             square_from_algebraic("c1"), square_from_algebraic("h1"), square_from_algebraic("h8"));
    // let mut i = 63;
    // while i > 0 {
    //     println!("{}", i / 8);
    //     i -= 8;
    // }

    // let board = create_default_board();
    // board.print_board();
    // let moves = get_moves(&board, 0, !0, true);
    // println!("Move Count: {}", moves.len());
    // assert_eq!(moves.len(), 20);

    // println!("a1 {}, a8 {}, h1 {}, h8 {}", square_from_algebraic("a1"),
    //          square_from_algebraic("a8"), square_from_algebraic("h1"), square_from_algebraic("h8"));
    // println!("4 {} 60 {}", square_to_algebraic(4), square_to_algebraic(60));

    // println!();

    // let second_board = create_board_from_string("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1");
    // second_board.print_board();

    // let moves = get_moves(&second_board, 0, !0, true);
    // println!("Move Count: {}", moves.len());
    // for move_ in moves {
    //     println!("{} to {} capture {}", square_to_algebraic(move_.from),
    //              square_to_algebraic(move_.to), move_.capture);
    // }

    let start_time = Instant::now();
    test("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 4, 89_941_194, 0b0011, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());

    let start_time = Instant::now();
    test_no_thread("rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R", 4, 89_941_194, 0b0011, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());

    let start_time = Instant::now();
    test("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R", 5, 8_031_647_685, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());

    // let start_time = Instant::now();
    // test_no_thread("r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R", 5, 8_031_647_685, 0b1111, true, "");
    // println!("Time elapsed: {:?}", start_time.elapsed());

    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 0, 20, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 1, 400, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 2, 8_902, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 3, 197_281, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 4, 4_865_609, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 5, 119_060_324, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 6, 3_195_901_860, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
    let start_time = Instant::now();
    test("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR", 7, 84_998_978_956, 0b1111, true, "");
    println!("Time elapsed: {:?}", start_time.elapsed());
}

#[cfg(test)]
mod test2 {
    #[test]
    fn it_works() {}
}
