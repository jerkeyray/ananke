use crate::bitboard::{Bitboard, Square};
use crate::board::Board;
use crate::magic;
use crate::types::{Color, Move, MoveList, PieceType};

/// All squares a knight on `sq` can attack (ignoring blockers).
pub fn generate_knight_attacks(sq: Square) -> Bitboard {
    let mut attacks = 0u64;
    let b = 1u64 << (sq as u8);

    // Knight moves are fixed offsets. Masks prevent wrapping off the board.
    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
    const NOT_AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC;
    const NOT_GH_FILE: u64 = 0x3F3F3F3F3F3F3F3F;

    // Two up and right
    if (b & NOT_H_FILE) != 0 {
        attacks |= b << 17;
    }
    // Two up and left
    if (b & NOT_A_FILE) != 0 {
        attacks |= b << 15;
    }
    // Two down and right
    if (b & NOT_H_FILE) != 0 {
        attacks |= b >> 15;
    }
    // Two down and left
    if (b & NOT_A_FILE) != 0 {
        attacks |= b >> 17;
    }

    // One up, two right
    if (b & NOT_GH_FILE) != 0 {
        attacks |= b << 10;
    }
    // One up, two left
    if (b & NOT_AB_FILE) != 0 {
        attacks |= b << 6;
    }
    // One down, two right
    if (b & NOT_GH_FILE) != 0 {
        attacks |= b >> 6;
    }
    // One down, two left
    if (b & NOT_AB_FILE) != 0 {
        attacks |= b >> 10;
    }

    Bitboard::new(attacks)
}

/// All squares a king on `sq` can attack (ignoring blockers).
pub fn generate_king_attacks(sq: Square) -> Bitboard {
    let mut attacks = 0u64;
    let b = 1u64 << (sq as u8);

    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;

    // Straight moves (always valid)
    attacks |= b << 8;
    attacks |= b >> 8;

    // East and diagonals (avoid A file wrapping)
    if (b & NOT_H_FILE) != 0 {
        attacks |= b << 1;
        attacks |= b << 9;
        attacks |= b >> 7;
    }

    // West and diagonals (avoid H file wrapping)
    if (b & NOT_A_FILE) != 0 {
        attacks |= b >> 1;
        attacks |= b << 7;
        attacks |= b >> 9;
    }

    Bitboard::new(attacks)
}

/// Slow but correct bishop attack generation. Used to build magic tables.
pub fn generate_bishop_attacks_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;

    let target_rank = sq.rank() as i8;
    let target_file = sq.file() as i8;

    // Four diagonal directions
    let directions = [(1, 1), (1, -1), (-1, 1), (-1, -1)];

    for (dr, df) in directions {
        for i in 1..8 {
            let r = target_rank + (dr * i);
            let f = target_file + (df * i);

            // Stop if we go off board
            if !(0..=7).contains(&r) || !(0..=7).contains(&f) {
                break;
            }

            let current_sq = Square::new((r * 8 + f) as u8);
            attacks.set_bit(current_sq);

            // Blockers stop the ray (but we include the blocker square)
            if blockers.get_bit(current_sq) {
                break;
            }
        }
    }
    attacks
}

/// Slow but correct rook attack generation. Used to build magic tables.
pub fn generate_rook_attacks_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;

    let target_rank = sq.rank() as i8;
    let target_file = sq.file() as i8;

    // Four orthogonal directions
    let directions = [(1, 0), (-1, 0), (0, 1), (0, -1)];

    for (dr, df) in directions {
        for i in 1..8 {
            let r = target_rank + (dr * i);
            let f = target_file + (df * i);

            if !(0..=7).contains(&r) || !(0..=7).contains(&f) {
                break;
            }

            let current_sq = Square::new((r * 8 + f) as u8);
            attacks.set_bit(current_sq);

            if blockers.get_bit(current_sq) {
                break;
            }
        }
    }
    attacks
}

pub struct MoveGenerator<'a> {
    board: &'a Board,
    moves: MoveList,
}

impl<'a> MoveGenerator<'a> {
    pub fn new(board: &'a Board) -> Self {
        MoveGenerator {
            board,
            moves: MoveList::new(),
        }
    }

    /// Generate all pseudo-legal moves for the current position.
    pub fn generate_all(mut self) -> MoveList {
        self.generate_pawn_moves();
        self.generate_knight_moves();
        self.generate_king_moves();
        self.generate_slider_moves();
        self.moves
    }

    // pawn moves
    fn generate_pawn_moves(&mut self) {
        let white = self.board.side_to_move == Color::White;
        let (pawns, enemies) = if white {
            (
                self.board.white_pieces[PieceType::Pawn as usize],
                self.board.black_occupancy,
            )
        } else {
            (
                self.board.black_pieces[PieceType::Pawn as usize],
                self.board.white_occupancy,
            )
        };

        let empty = !self.board.all_occupancy;

        // Single square push
        let single_push = if white {
            (pawns.0 << 8) & empty.0
        } else {
            (pawns.0 >> 8) & empty.0
        };
        let mut bb = Bitboard::new(single_push);
        while let Some(to_sq) = bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 8
            } else {
                to_sq as u8 + 8
            });
            self.moves.push(Move::new(from_sq, to_sq, Move::QUIET));
        }

        // Double push from starting rank
        let double_push = if white {
            ((single_push << 8) & empty.0) & 0x000000FF00000000
        } else {
            ((single_push >> 8) & empty.0) & 0x000000FF00000000
        };
        let mut bb = Bitboard::new(double_push);
        while let Some(to_sq) = bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 16
            } else {
                to_sq as u8 + 16
            });
            self.moves
                .push(Move::new(from_sq, to_sq, Move::DOUBLE_PAWN_PUSH));
        }

        // Captures - shift diagonally and intersect with enemy pieces
        let (left_attack, right_attack) = if white {
            (
                (pawns.0 << 7) & 0x7F7F7F7F7F7F7F7F,
                (pawns.0 << 9) & 0xFEFEFEFEFEFEFEFE,
            )
        } else {
            (
                (pawns.0 >> 9) & 0x7F7F7F7F7F7F7F7F,
                (pawns.0 >> 7) & 0xFEFEFEFEFEFEFEFE,
            )
        };

        let mut left_bb = Bitboard::new(left_attack & enemies.0);
        while let Some(to_sq) = left_bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 7
            } else {
                to_sq as u8 + 9
            });
            self.moves.push(Move::new(from_sq, to_sq, Move::CAPTURE));
        }

        let mut right_bb = Bitboard::new(right_attack & enemies.0);
        while let Some(to_sq) = right_bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 9
            } else {
                to_sq as u8 + 7
            });
            self.moves.push(Move::new(from_sq, to_sq, Move::CAPTURE));
        }
    }

    // knight moves
    fn generate_knight_moves(&mut self) {
        let white = self.board.side_to_move == Color::White;
        let mut knights = if white {
            self.board.white_pieces[PieceType::Knight as usize]
        } else {
            self.board.black_pieces[PieceType::Knight as usize]
        };

        let friends = if white {
            self.board.white_occupancy
        } else {
            self.board.black_occupancy
        };
        let enemies = if white {
            self.board.black_occupancy
        } else {
            self.board.white_occupancy
        };

        while let Some(from_sq) = knights.pop_lsb() {
            let attacks = generate_knight_attacks(from_sq) & !friends;

            let mut moves_bb = attacks;
            while let Some(to_sq) = moves_bb.pop_lsb() {
                let flag = if enemies.get_bit(to_sq) {
                    Move::CAPTURE
                } else {
                    Move::QUIET
                };
                self.moves.push(Move::new(from_sq, to_sq, flag));
            }
        }
    }

    // king moves
    fn generate_king_moves(&mut self) {
        let white = self.board.side_to_move == Color::White;
        let mut kings = if white {
            self.board.white_pieces[PieceType::King as usize]
        } else {
            self.board.black_pieces[PieceType::King as usize]
        };

        let friends = if white {
            self.board.white_occupancy
        } else {
            self.board.black_occupancy
        };
        let enemies = if white {
            self.board.black_occupancy
        } else {
            self.board.white_occupancy
        };

        if let Some(from_sq) = kings.pop_lsb() {
            let attacks = generate_king_attacks(from_sq) & !friends;

            let mut moves_bb = attacks;
            while let Some(to_sq) = moves_bb.pop_lsb() {
                let flag = if enemies.get_bit(to_sq) {
                    Move::CAPTURE
                } else {
                    Move::QUIET
                };
                self.moves.push(Move::new(from_sq, to_sq, flag));
            }
        }
    }

    // slider moves (rook, bishop, queen)
    fn generate_slider_moves(&mut self) {
        let white = self.board.side_to_move == Color::White;
        let friends = if white {
            self.board.white_occupancy
        } else {
            self.board.black_occupancy
        };
        let enemies = if white {
            self.board.black_occupancy
        } else {
            self.board.white_occupancy
        };

        // Generic helper for rooks, bishops, and queens
        let mut generate = |piece_type: PieceType, rook_like: bool, bishop_like: bool| {
            let mut pieces = if white {
                self.board.white_pieces[piece_type as usize]
            } else {
                self.board.black_pieces[piece_type as usize]
            };

            while let Some(from_sq) = pieces.pop_lsb() {
                let mut attacks = Bitboard::EMPTY;

                if rook_like {
                    attacks |= magic::get_rook_attacks(from_sq, self.board.all_occupancy);
                }
                if bishop_like {
                    attacks |= magic::get_bishop_attacks(from_sq, self.board.all_occupancy);
                }

                attacks &= !friends;

                while let Some(to_sq) = attacks.pop_lsb() {
                    let flag = if enemies.get_bit(to_sq) {
                        Move::CAPTURE
                    } else {
                        Move::QUIET
                    };
                    self.moves.push(Move::new(from_sq, to_sq, flag));
                }
            }
        };

        generate(PieceType::Rook, true, false);
        generate(PieceType::Bishop, false, true);
        generate(PieceType::Queen, true, true);
    }
}
