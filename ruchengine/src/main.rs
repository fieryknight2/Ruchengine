use board::moves::get_moves;
use board::board::{create_board_from_string, create_default_board, square_to_algebraic};

fn main() {
    println!("Hello, world!");

    let board = create_default_board();
    board.print_board();

    println!();

    let second_board = create_board_from_string("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1");
    second_board.print_board();

    let moves = get_moves(&second_board, 0, !0, true);
    for move_ in moves {
        println!("{} to {} capture {} castle {}", square_to_algebraic(move_.from),
                 square_to_algebraic(move_.to), move_.capture, move_.castle_rights);
    }
}
#[cfg(test)]
mod test2 {
    #[test]
    fn it_works() {}
}
