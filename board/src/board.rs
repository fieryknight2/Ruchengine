//   PawnLocation = 0,
//   KnightLocation = 1,
//   BishopLocation = 2,
//   RookLocation = 3,
//   QueenLocation = 4,
//   KingLocation = 5,
const WHITE_PIECES: usize = 6;
const BLACK_PIECES: usize = 7;

pub const PAWN: usize = 0;
pub const KNIGHT: usize = 1;
pub const BISHOP: usize = 2;
pub const ROOK: usize = 3;
pub const QUEEN: usize = 4;
pub const KING: usize = 5;

pub struct Board {
    pub bitboards: [u64; 8],
}

pub fn print_bitboard(bitboard: u64, on: char, off: char) {
    for r in 0..8 {
        for c in 0..8 {
            if (bitboard & (1u64 << (r * 8 + (7 - c)))) != 0 {
                print!("{} ", on);
            } else {
                print!("{} ", off);
            }
        }
        println!();
    }
    println!();
}

pub fn square_from_algebraic(sqr: &str) -> u64 {
    let mut file: u64 = 0;
    let mut rank: u64 = 0;

    assert_eq!(sqr.len(), 2, "Invalid algebraic square format");
    assert!(sqr.chars().next().unwrap_or('z').is_alphabetic(), "Invalid algebraic square format");
    assert!(sqr.chars().nth(1).unwrap_or('z').is_numeric(), "Invalid algebraic square format");

    // Parse the square
    for c in sqr.chars() {
        match c {
            'a' => file = 0,
            'b' => file = 1,
            'c' => file = 2,
            'd' => file = 3,
            'e' => file = 4,
            'f' => file = 5,
            'g' => file = 6,
            'h' => file = 7,
            '1' => rank = 0,
            '2' => rank = 1,
            '3' => rank = 2,
            '4' => rank = 3,
            '5' => rank = 4,
            '6' => rank = 5,
            '7' => rank = 6,
            '8' => rank = 7,
            _ => { assert!(false, "Invalid square"); }
        }
    }

    rank * 8 + file
}

pub fn square_to_algebraic(square: u64) -> String {
    assert!(0 < square && square < 64, "Invalid square");

    format!("{}{}", match square % 8 { // square % 8 is the file
        0 => 'a',
        1 => 'b',
        2 => 'c',
        3 => 'd',
        4 => 'e',
        5 => 'f',
        6 => 'g',
        7 => 'h',
        _ => {
            assert!(false, "Invalid square");
            'z'
        }
    }, square / 8 + 1) // square / 8 is the rank
}

pub fn create_board() -> Board {
    Board {
        bitboards: [0x0; 8],
    }
}

pub fn create_board_from_string(string: &str) -> Board {
    let mut board = Board {
        bitboards: [0x0; 8],
    };

    // Parse the fen
    let ranks = string.split('/');
    assert_eq!(string.split('/').count(), 8, "Invalid fen format");

    let mut current_rank: u32 = 8;
    for rank in ranks {
        let mut current_file: u32 = 0;

        current_rank -= 1;
        for file in rank.chars() {
            if file.is_numeric() {
                current_file += file.to_digit(10).unwrap();
                continue;
            }
            assert!(current_file <= 8, "Invalid fen format");
            let square = current_rank * 8 + current_file;

            board.bitboards[match file {
                'p' | 'P' => 0,
                'n' | 'N' => 1,
                'b' | 'B' => 2,
                'r' | 'R' => 3,
                'q' | 'Q' => 4,
                'k' | 'K' => 5,
                _ => { panic!("Invalid fen format"); }
            }] |= 1u64 << square;
            board.bitboards[if file.is_uppercase() { WHITE_PIECES } else { BLACK_PIECES }] |= 1u64 << square;
            current_file += 1;
        }
    }

    board
}

pub fn create_default_board() -> Board {
    create_board_from_string("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR")
}

impl Board {
    pub fn all_pieces(&self) -> u64 {
        self.bitboards[WHITE_PIECES] | self.bitboards[BLACK_PIECES]
    }

    pub fn white_pieces(&self) -> u64 {
        self.bitboards[WHITE_PIECES]
    }

    pub fn black_pieces(&self) -> u64 {
        self.bitboards[BLACK_PIECES]
    }

    pub fn print_board(&self) {
        const PIECE_REP: [char; 6] = ['P', 'N', 'B', 'R', 'Q', 'K'];
        let mut board: [char; 64] = ['.'; 64];
        // Generate the visible board
        for (piece_type, bitboard) in self.bitboards.iter().enumerate() {
            if piece_type == 6 || piece_type == 7 { continue; }

            for square in 0..64 {
                if (bitboard & (1u64 << square)) != 0 {
                    board[square] = if self.bitboards[WHITE_PIECES] & (1u64 << square) != 0 {
                        PIECE_REP[piece_type]
                    } else {
                        PIECE_REP[piece_type].to_lowercase().next().unwrap()
                    };
                }
            }
        }

        // Print the board
        println!("  A B C D E F G H");
        for i in 0..8 {
            print!("{}", 8 - i);
            for j in 0..8 {
                print!(" {}", board[(7 - i) * 8 + j]);
            }
            println!();
        }
    }
}