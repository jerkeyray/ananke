use crate::bitboard::{Bitboard, Square};
use crate::board::Board;
use crate::magic;
use crate::types::{Color, Move, MoveList, PieceType};

// leaper attack generators

pub fn generate_knight_attacks(sq: Square) -> Bitboard {
    let mut attacks = 0u64;
    let b = 1u64 << (sq as u8);

    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;
    const NOT_AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC;
    const NOT_GH_FILE: u64 = 0x3F3F3F3F3F3F3F3F;

    if (b & NOT_H_FILE) != 0 {
        attacks |= b << 17;
    }
    if (b & NOT_A_FILE) != 0 {
        attacks |= b << 15;
    }
    if (b & NOT_H_FILE) != 0 {
        attacks |= b >> 15;
    }
    if (b & NOT_A_FILE) != 0 {
        attacks |= b >> 17;
    }
    if (b & NOT_GH_FILE) != 0 {
        attacks |= b << 10;
    }
    if (b & NOT_AB_FILE) != 0 {
        attacks |= b << 6;
    }
    if (b & NOT_GH_FILE) != 0 {
        attacks |= b >> 6;
    }
    if (b & NOT_AB_FILE) != 0 {
        attacks |= b >> 10;
    }

    Bitboard::new(attacks)
}

pub fn generate_king_attacks(sq: Square) -> Bitboard {
    let mut attacks = 0u64;
    let b = 1u64 << (sq as u8);

    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;

    attacks |= b << 8;
    attacks |= b >> 8;

    if (b & NOT_H_FILE) != 0 {
        attacks |= b << 1;
        attacks |= b << 9;
        attacks |= b >> 7;
    }

    if (b & NOT_A_FILE) != 0 {
        attacks |= b >> 1;
        attacks |= b << 7;
        attacks |= b >> 9;
    }

    Bitboard::new(attacks)
}

// these slow functions are kept for magic initialization
pub fn generate_bishop_attacks_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let target_rank = sq.rank() as i8;
    let target_file = sq.file() as i8;

    for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
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

pub fn generate_rook_attacks_slow(sq: Square, blockers: Bitboard) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let target_rank = sq.rank() as i8;
    let target_file = sq.file() as i8;

    for (dr, df) in [(1, 0), (-1, 0), (0, 1), (0, -1)] {
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

// move generator

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

    pub fn generate_all(mut self) -> MoveList {
        self.generate_pawn_moves();
        self.generate_knight_moves();
        self.generate_king_moves();
        self.generate_slider_moves();
        self.moves
    }

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
        let promotion_rank = if white { 7 } else { 0 };

        // single push
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

            // handle promotions
            if to_sq.rank() == promotion_rank {
                self.moves.push(Move::new(from_sq, to_sq, Move::N_PROMO));
                self.moves.push(Move::new(from_sq, to_sq, Move::B_PROMO));
                self.moves.push(Move::new(from_sq, to_sq, Move::R_PROMO));
                self.moves.push(Move::new(from_sq, to_sq, Move::Q_PROMO));
            } else {
                self.moves.push(Move::new(from_sq, to_sq, Move::QUIET));
            }
        }

        // double push
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

        // captures
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

        // regular captures
        let mut left_bb = Bitboard::new(left_attack & enemies.0);
        while let Some(to_sq) = left_bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 7
            } else {
                to_sq as u8 + 9
            });

            if to_sq.rank() == promotion_rank {
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::N_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::B_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::R_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::Q_PROMO_CAP));
            } else {
                self.moves.push(Move::new(from_sq, to_sq, Move::CAPTURE));
            }
        }

        let mut right_bb = Bitboard::new(right_attack & enemies.0);
        while let Some(to_sq) = right_bb.pop_lsb() {
            let from_sq = Square::new(if white {
                to_sq as u8 - 9
            } else {
                to_sq as u8 + 7
            });

            if to_sq.rank() == promotion_rank {
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::N_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::B_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::R_PROMO_CAP));
                self.moves
                    .push(Move::new(from_sq, to_sq, Move::Q_PROMO_CAP));
            } else {
                self.moves.push(Move::new(from_sq, to_sq, Move::CAPTURE));
            }
        }

        // en passant captures
        if let Some(ep_sq) = self.board.en_passant_sq {
            let ep_bitboard = Bitboard::new(1u64 << (ep_sq as u8));

            // check if left capture is possible
            if (left_attack & ep_bitboard.0) != 0 {
                let from_sq = if white {
                    Square::new((ep_sq as u8) - 7)
                } else {
                    Square::new((ep_sq as u8) + 9)
                };
                self.moves.push(Move::new(from_sq, ep_sq, Move::EP_CAPTURE));
            }

            // check if right capture is possible
            if (right_attack & ep_bitboard.0) != 0 {
                let from_sq = if white {
                    Square::new((ep_sq as u8) - 9)
                } else {
                    Square::new((ep_sq as u8) + 7)
                };
                self.moves.push(Move::new(from_sq, ep_sq, Move::EP_CAPTURE));
            }
        }
    }

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

            // castling moves
            self.generate_castling_moves(from_sq, white);
        }
    }

    fn generate_castling_moves(&mut self, king_sq: Square, white: bool) {
        let color = if white { Color::White } else { Color::Black };
        let (king_start, _rook_qs_start, _rook_ks_start, ks_target, qs_target) = if white {
            (Square::E1, Square::A1, Square::H1, Square::G1, Square::C1)
        } else {
            (Square::E8, Square::A8, Square::H8, Square::G8, Square::C8)
        };

        let rights = self.board.castling_rights;

        // check if king is on starting square
        if king_sq != king_start {
            return;
        }

        // kingside castling
        if rights.can_castle_kingside(color) {
            // check squares between king and rook are empty
            let f_sq = if white { Square::F1 } else { Square::F8 };
            let g_sq = if white { Square::G1 } else { Square::G8 };

            if !self.board.all_occupancy.get_bit(f_sq) && !self.board.all_occupancy.get_bit(g_sq) {
                // check king is not in check and doesn't pass through check
                let them = if white { Color::Black } else { Color::White };
                if !self.board.is_square_attacked(king_start, them)
                    && !self.board.is_square_attacked(f_sq, them)
                    && !self.board.is_square_attacked(g_sq, them)
                {
                    self.moves
                        .push(Move::new(king_start, ks_target, Move::K_CASTLE));
                }
            }
        }

        // queenside castling
        if rights.can_castle_queenside(color) {
            // check squares between king and rook are empty
            let d_sq = if white { Square::D1 } else { Square::D8 };
            let c_sq = if white { Square::C1 } else { Square::C8 };
            let b_sq = if white { Square::B1 } else { Square::B8 };

            if !self.board.all_occupancy.get_bit(d_sq)
                && !self.board.all_occupancy.get_bit(c_sq)
                && !self.board.all_occupancy.get_bit(b_sq)
            {
                // check king is not in check and doesn't pass through check
                let them = if white { Color::Black } else { Color::White };
                if !self.board.is_square_attacked(king_start, them)
                    && !self.board.is_square_attacked(d_sq, them)
                    && !self.board.is_square_attacked(c_sq, them)
                {
                    self.moves
                        .push(Move::new(king_start, qs_target, Move::Q_CASTLE));
                }
            }
        }
    }

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

        let mut generate = |piece_type: PieceType, is_rook: bool, is_bishop: bool| {
            let mut pieces = if white {
                self.board.white_pieces[piece_type as usize]
            } else {
                self.board.black_pieces[piece_type as usize]
            };

            while let Some(from_sq) = pieces.pop_lsb() {
                let mut attacks = Bitboard::EMPTY;
                if is_rook {
                    attacks |= magic::get_rook_attacks(from_sq, self.board.all_occupancy);
                }
                if is_bishop {
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

pub fn generate_pawn_attacks(sq: Square, color: Color) -> Bitboard {
    let mut attacks = Bitboard::EMPTY;
    let b = Bitboard::new(1u64 << (sq as u8));

    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;

    if color == Color::White {
        if (b.0 & NOT_H_FILE) != 0 {
            attacks.0 |= b.0 << 9;
        }
        if (b.0 & NOT_A_FILE) != 0 {
            attacks.0 |= b.0 << 7;
        }
    } else {
        if (b.0 & NOT_H_FILE) != 0 {
            attacks.0 |= b.0 >> 7;
        }
        if (b.0 & NOT_A_FILE) != 0 {
            attacks.0 |= b.0 >> 9;
        }
    }
    attacks
}
