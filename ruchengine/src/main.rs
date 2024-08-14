use std::io;
use std::io::Write;
use std::time::Instant;
use board::board::{create_default_board, print_bitboard, square_from_algebraic, square_to_algebraic};
use board::{count_moves};
use board::count_moves_no_threads;
use board::moves::{get_moves, in_check, Move};
use engine::evaluate::{BoardState, evaluate};
// use engine::get_best_move;
use engine::get_best_moves;

const DEPTH: u32 = 300;

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

fn _testing() {
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

fn main() {
    // testing();
    let mut board = BoardState {
        board: create_default_board(),
        en_passant: 0,
        castle_rights: 0xf,
        white: true,
        half_move_clock: 0,
        full_move_count: 0,
    };

    println!("Evaluation at base: {}", evaluate(&mut board));

    loop {
        board.board.print_board();

        let possible_moves = get_moves(&board.board, board.en_passant, board.castle_rights, board.white);
        if possible_moves.is_empty() {
            println!("Game Over");
            if in_check(&board.board, true) {
                println!("Checkmate! You lose!");
            } else {
                println!("Stalemate! Draw!");
            }
            break;
        }

        print!("Enter move (e.g. e2e4=z): ");
        io::stdout().flush().expect("TODO: panic message");

        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Read line failed");
        input = input.trim().to_string();
        if input == "quit" {
            break;
        }
        if input == "moves" {
            for move_ in possible_moves {
                println!("{}{}={}", square_to_algebraic(move_.from), square_to_algebraic(move_.to), move_.promotion_type);
            }
            println!();
            continue;
        }

        if input.len() != 4 && input.len() != 6 {
            println!("Invalid input");
            continue;
        }

        let (from, mut to) = input.split_at(2);
        let mut promotion = "z";
        if input.len() == 6 {
            let promo;
            (to, promo) = to.split_at(2);
            promotion = promo.get(1..2).unwrap_or("");
        }
        let from = square_from_algebraic(from);
        let to = square_from_algebraic(to);


        let mut move_ = Move {
            from,
            to,
            promotion_type: promotion.parse().unwrap(),
            en_passant: 0,
            capture: 'z',
            castle_rights: 0xff,
        };
        for p_move in possible_moves {
            if p_move.from == from && p_move.to == to && p_move.promotion_type == promotion.parse().unwrap() {
                move_ = p_move;
                break;
            }
        }
        if move_.castle_rights == 0xff {
            println!("Invalid move");
            continue;
        }

        let (en_passant, castle_rights) = move_.make_move(&mut board.board);
        board.en_passant = en_passant;
        board.castle_rights = castle_rights;
        board.white = !board.white;

        // Respond
        println!("Move made");
        let start_time = Instant::now();
        let best_moves = get_best_moves(&mut board, DEPTH);
        println!("Time elapsed: {:?}", start_time.elapsed());

        if best_moves.is_empty() {
            println!("Game Over");
            if in_check(&board.board, false) {
                println!("Checkmate! You win!");
            } else {
                println!("Stalemate! Draw!");
            }

            break;
        }

        let best_move = best_moves[0].0;
        println!("Best move: {}{}={}", square_to_algebraic(best_move.from), square_to_algebraic(best_move.to), best_move.promotion_type);

        println!("\nBest move list:");
        for eval in best_moves {
            println!("{}{}={}: {}", square_to_algebraic(eval.0.from), square_to_algebraic(eval.0.to), eval.0.promotion_type, eval.1);
        }
        println!();

        // Make engine move
        best_move.make_move(&mut board.board);
        board.white = !board.white;
    }
}

#[cfg(test)]
mod test2 {
    #[test]
    fn it_works() {}
}
