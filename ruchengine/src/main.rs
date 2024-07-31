use crate::board::Board;

fn main() {
    println!("Hello, world!");

    let board = board::create_default_board();
    board.print_board();

    println!();

    let second_board = board::create_board_from_string("r2q1rk1/2p1bppp/p2p1n2/1p2P3/4P1b1/1nP1BN2/PP3PPP/RN1QR1K1");
    second_board.print_board();

    Ok(())
}
#[cfg(test)]
mod test2 {
    use super::*;

    #[test]
    fn it_works() {
        let result = board::create_board();
    }
}
