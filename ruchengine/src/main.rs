use board::moves::get_moves;
use board::board::{create_board_from_string, create_default_board, print_bitboard, square_from_algebraic, square_to_algebraic};

fn main() {
    println!("Hello, world!");
    println!("{}, {}", square_to_algebraic(4), square_to_algebraic(22));
    print_bitboard(18446744073709551615, 'X', '.');
    print_bitboard(8815957245952, 'X', '.');
    println!("g7 {} h6 {} g7 {} h8 {}", square_from_algebraic("g7"),
             square_from_algebraic("h6"), square_from_algebraic("g7"), square_from_algebraic("h8"));
    // let mut i = 63;
    // while i > 0 {
    //     println!("{}", i / 8);
    //     i -= 8;
    // }

    let board = create_default_board();
    board.print_board();
    let moves = get_moves(&board, 0, !0, true);
    println!("Move Count: {}", moves.len());
    assert_eq!(moves.len(), 20);

    println!("a1 {}, a8 {}, h1 {}, h8 {}", square_from_algebraic("a1"),
             square_from_algebraic("a8"), square_from_algebraic("h1"), square_from_algebraic("h8"));
    println!("4 {} 60 {}", square_to_algebraic(4), square_to_algebraic(60));

    println!();

    let second_board = create_board_from_string("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1");
    second_board.print_board();

    // let moves = get_moves(&second_board, 0, !0, true);
    // println!("Move Count: {}", moves.len());
    // for move_ in moves {
    //     println!("{} to {} capture {}", square_to_algebraic(move_.from),
    //              square_to_algebraic(move_.to), move_.capture);
    // }
}
#[cfg(test)]
mod test2 {
    #[test]
    fn it_works() {}
}
