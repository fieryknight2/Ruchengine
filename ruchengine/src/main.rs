use board::moves::get_moves;
use board::board::{create_board_from_string, create_default_board, print_bitboard, square_from_algebraic, square_to_algebraic};

fn main() {
    println!("Hello, world!");
    println!("{}, {}", square_to_algebraic(31), square_to_algebraic(22));
    print_bitboard(33619968, 'X', '.');
    print_bitboard(7706239435266, 'X', '.');
    println!("f7 {} a5 {} h4 {} h5 {}", square_from_algebraic("f7"),
             square_from_algebraic("a5"), square_from_algebraic("h4"), square_from_algebraic("h5"));

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
