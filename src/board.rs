use crate::bitboard::{Bitboard, Square};
use crate::types::{Color, PieceType};

pub struct Board {
    /// White pieces, indexed by PieceType (0=Pawn, 1=Knight, etc.)
    pub white_pieces: [Bitboard; 6],
    pub black_pieces: [Bitboard; 6],

    /// Aggregate bitboards - computed from the piece bitboards
    pub white_occupancy: Bitboard,
    pub black_occupancy: Bitboard,
    pub all_occupancy: Bitboard,

    pub side_to_move: Color,
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            white_pieces: [Bitboard::EMPTY; 6],
            black_pieces: [Bitboard::EMPTY; 6],
            white_occupancy: Bitboard::EMPTY,
            black_occupancy: Bitboard::EMPTY,
            all_occupancy: Bitboard::EMPTY,
            side_to_move: Color::White,
        }
    }

    /// Refresh all occupancy bitboards. Call this after making moves.
    pub fn update_occupancies(&mut self) {
        self.white_occupancy = Bitboard::EMPTY;
        self.black_occupancy = Bitboard::EMPTY;

        for bb in self.white_pieces.iter() {
            self.white_occupancy |= *bb;
        }
        for bb in self.black_pieces.iter() {
            self.black_occupancy |= *bb;
        }

        self.all_occupancy = self.white_occupancy | self.black_occupancy;
    }

    /// Set up the board from a FEN string.
    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        // Parse the piece placement part (the first FEN field)
        let rows: Vec<&str> = parts[0].split('/').collect();
        if rows.len() != 8 {
            return Err("Invalid FEN: expected 8 rows".to_string());
        }

        for (rank_idx, row) in rows.iter().enumerate() {
            let rank = 7 - rank_idx as u8;
            let mut file = 0;

            for char in row.chars() {
                if char.is_ascii_digit() {
                    file += char.to_digit(10).unwrap() as u8;
                } else {
                    let piece_type = match char.to_ascii_lowercase() {
                        'p' => PieceType::Pawn,
                        'n' => PieceType::Knight,
                        'b' => PieceType::Bishop,
                        'r' => PieceType::Rook,
                        'q' => PieceType::Queen,
                        'k' => PieceType::King,
                        _ => return Err(format!("Unknown piece: {}", char)),
                    };

                    let color = if char.is_uppercase() {
                        Color::White
                    } else {
                        Color::Black
                    };
                    let square = Square::new(rank * 8 + file);

                    let idx = piece_type as usize;
                    if color == Color::White {
                        board.white_pieces[idx].set_bit(square);
                    } else {
                        board.black_pieces[idx].set_bit(square);
                    }

                    file += 1;
                }
            }
        }

        // Side to move
        if parts.len() > 1 {
            board.side_to_move = if parts[1] == "w" {
                Color::White
            } else {
                Color::Black
            };
        }

        board.update_occupancies();
        Ok(board)
    }
}
