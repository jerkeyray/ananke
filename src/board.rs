use crate::bitboard::{Bitboard, Square};
use crate::types::{CastlingRights, Color, Move, PieceType};

#[derive(Clone)]
pub struct Board {
    pub white_pieces: [Bitboard; 6],
    pub black_pieces: [Bitboard; 6],
    pub white_occupancy: Bitboard,
    pub black_occupancy: Bitboard,
    pub all_occupancy: Bitboard,
    pub side_to_move: Color,

    // State fields
    pub castling_rights: CastlingRights,
    pub en_passant_sq: Option<Square>,
    pub halfmove_clock: u8,
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
            castling_rights: CastlingRights::new(),
            en_passant_sq: None,
            halfmove_clock: 0,
        }
    }

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

    pub fn from_fen(fen: &str) -> Result<Self, String> {
        let mut board = Board::new();
        let parts: Vec<&str> = fen.split_whitespace().collect();

        if parts.len() < 2 {
            return Err("Invalid FEN: not enough fields".to_string());
        }

        // 1. Piece placement
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

        // 2. Side to move
        board.side_to_move = if parts[1] == "w" {
            Color::White
        } else {
            Color::Black
        };

        // 3. Castling rights
        if parts.len() > 2 && parts[2] != "-" {
            for c in parts[2].chars() {
                match c {
                    'K' => board.castling_rights.add_white_kingside(),
                    'Q' => board.castling_rights.add_white_queenside(),
                    'k' => board.castling_rights.add_black_kingside(),
                    'q' => board.castling_rights.add_black_queenside(),
                    _ => return Err(format!("Invalid castling char: {}", c)),
                }
            }
        }

        // 4. En passant target square
        if parts.len() > 3 && parts[3] != "-" {
            let ep_str = parts[3];
            if ep_str.len() != 2 {
                return Err(format!("Invalid en passant: {}", ep_str));
            }
            let file_char = ep_str.chars().nth(0).unwrap();
            let rank_char = ep_str.chars().nth(1).unwrap();

            let file = file_char as u8 - b'a';
            let rank = rank_char as u8 - b'1';

            if file > 7 || rank > 7 {
                return Err(format!("Invalid en passant square: {}", ep_str));
            }
            board.en_passant_sq = Some(Square::new(rank * 8 + file));
        }

        // 5. Halfmove clock (optional, default 0)
        if parts.len() > 4 {
            board.halfmove_clock = parts[4].parse().unwrap_or(0);
        }

        board.update_occupancies();
        Ok(board)
    }

    // core logic: execute a move
    pub fn make_move(&self, m: Move) -> Board {
        let mut next = self.clone();

        let from = m.from();
        let to = m.to();
        let flag = m.flag();
        let us = self.side_to_move;
        let them = us.opposite();

        // 1. Move the piece
        let piece_type = self
            .get_piece_type_at(from, us)
            .expect("No piece at from square");
        next.remove_piece(piece_type, us, from);
        next.add_piece(piece_type, us, to);

        // 2. Handle Castling
        if piece_type == PieceType::King && (from as i8 - to as i8).abs() == 2 {
            // Kingside castling
            if to as u8 > from as u8 {
                let rook_from = if us == Color::White {
                    Square::H1
                } else {
                    Square::H8
                };
                let rook_to = if us == Color::White {
                    Square::F1
                } else {
                    Square::F8
                };
                next.remove_piece(PieceType::Rook, us, rook_from);
                next.add_piece(PieceType::Rook, us, rook_to);
            }
            // Queenside castling
            else {
                let rook_from = if us == Color::White {
                    Square::A1
                } else {
                    Square::A8
                };
                let rook_to = if us == Color::White {
                    Square::D1
                } else {
                    Square::D8
                };
                next.remove_piece(PieceType::Rook, us, rook_from);
                next.add_piece(PieceType::Rook, us, rook_to);
            }
            // Castling removes all castling rights for this side
            next.castling_rights.remove(match us {
                Color::White => CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE,
                Color::Black => CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE,
            });
        }

        // 3. Handle Captures
        if m.is_capture() {
            if flag == Move::EP_CAPTURE {
                let cap_sq = if us == Color::White {
                    Square::new((to as u8) - 8)
                } else {
                    Square::new((to as u8) + 8)
                };
                next.remove_piece(PieceType::Pawn, them, cap_sq);
            } else {
                let captured_type = self
                    .get_piece_type_at(to, them)
                    .expect("Capture but no enemy");
                next.remove_piece(captured_type, them, to);

                // Capturing a rook removes castling rights for that side
                if captured_type == PieceType::Rook {
                    if them == Color::White {
                        if to == Square::A1 {
                            next.castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
                        }
                        if to == Square::H1 {
                            next.castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
                        }
                    } else {
                        if to == Square::A8 {
                            next.castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
                        }
                        if to == Square::H8 {
                            next.castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
                        }
                    }
                }
            }
        }

        // 4. Handle Promotions
        if m.is_promotion() {
            next.remove_piece(PieceType::Pawn, us, to);
            let promo_type = match flag {
                Move::N_PROMO | Move::N_PROMO_CAP => PieceType::Knight,
                Move::B_PROMO | Move::B_PROMO_CAP => PieceType::Bishop,
                Move::R_PROMO | Move::R_PROMO_CAP => PieceType::Rook,
                Move::Q_PROMO | Move::Q_PROMO_CAP => PieceType::Queen,
                _ => panic!("Invalid promo flag"),
            };
            next.add_piece(promo_type, us, to);
        }

        // 5. Handle Castling Rights (king or rook moved)
        if piece_type == PieceType::King {
            next.castling_rights.remove(match us {
                Color::White => CastlingRights::WHITE_KINGSIDE | CastlingRights::WHITE_QUEENSIDE,
                Color::Black => CastlingRights::BLACK_KINGSIDE | CastlingRights::BLACK_QUEENSIDE,
            });
        }
        if piece_type == PieceType::Rook {
            if from == Square::A1 || to == Square::A1 {
                next.castling_rights.remove(CastlingRights::WHITE_QUEENSIDE);
            }
            if from == Square::H1 || to == Square::H1 {
                next.castling_rights.remove(CastlingRights::WHITE_KINGSIDE);
            }
            if from == Square::A8 || to == Square::A8 {
                next.castling_rights.remove(CastlingRights::BLACK_QUEENSIDE);
            }
            if from == Square::H8 || to == Square::H8 {
                next.castling_rights.remove(CastlingRights::BLACK_KINGSIDE);
            }
        }

        // 6. Update State
        next.side_to_move = them;
        next.en_passant_sq = None;

        if flag == Move::DOUBLE_PAWN_PUSH {
            let ep_sq = if us == Color::White {
                Square::new((from as u8) + 8)
            } else {
                Square::new((from as u8) - 8)
            };
            next.en_passant_sq = Some(ep_sq);
        }

        next.update_occupancies();
        next
    }

    // --- HELPERS ---

    fn get_piece_type_at(&self, sq: Square, color: Color) -> Option<PieceType> {
        let pieces = if color == Color::White {
            &self.white_pieces
        } else {
            &self.black_pieces
        };
        for (i, bb) in pieces.iter().enumerate() {
            if bb.get_bit(sq) {
                return Some(match i {
                    0 => PieceType::Pawn,
                    1 => PieceType::Knight,
                    2 => PieceType::Bishop,
                    3 => PieceType::Rook,
                    4 => PieceType::Queen,
                    5 => PieceType::King,
                    _ => unreachable!(),
                });
            }
        }
        None
    }

    fn remove_piece(&mut self, pt: PieceType, color: Color, sq: Square) {
        if color == Color::White {
            self.white_pieces[pt as usize].clear_bit(sq);
        } else {
            self.black_pieces[pt as usize].clear_bit(sq);
        }
    }

    fn add_piece(&mut self, pt: PieceType, color: Color, sq: Square) {
        if color == Color::White {
            self.white_pieces[pt as usize].set_bit(sq);
        } else {
            self.black_pieces[pt as usize].set_bit(sq);
        }
    }

    pub fn get_king_square(&self, color: Color) -> Square {
        let kings = if color == Color::White {
            self.white_pieces[PieceType::King as usize]
        } else {
            self.black_pieces[PieceType::King as usize]
        };
        kings.lsb_index().expect("Board has no King!")
    }

    pub fn is_square_attacked(&self, sq: Square, attacker: Color) -> bool {
        // 1. Check if an enemy Pawn attacks us
        let is_white_attacker = attacker == Color::White;
        if is_white_attacker {
            let white_pawns = self.white_pieces[PieceType::Pawn as usize];
            // If we pretend to be a Black pawn here, do we hit a White pawn?
            let attacks = crate::movegen::generate_pawn_attacks(sq, Color::Black);
            if (attacks & white_pawns).count() > 0 {
                return true;
            }
        } else {
            let black_pawns = self.black_pieces[PieceType::Pawn as usize];
            let attacks = crate::movegen::generate_pawn_attacks(sq, Color::White);
            if (attacks & black_pawns).count() > 0 {
                return true;
            }
        }

        // 2. Check Knights
        let knights = if is_white_attacker {
            self.white_pieces[PieceType::Knight as usize]
        } else {
            self.black_pieces[PieceType::Knight as usize]
        };
        if (crate::movegen::generate_knight_attacks(sq) & knights).count() > 0 {
            return true;
        }

        // 3. Check King
        let kings = if is_white_attacker {
            self.white_pieces[PieceType::King as usize]
        } else {
            self.black_pieces[PieceType::King as usize]
        };
        if (crate::movegen::generate_king_attacks(sq) & kings).count() > 0 {
            return true;
        }

        // 4. Check Rooks/Queens
        let rooks = if is_white_attacker {
            self.white_pieces[PieceType::Rook as usize]
        } else {
            self.black_pieces[PieceType::Rook as usize]
        };
        let queens = if is_white_attacker {
            self.white_pieces[PieceType::Queen as usize]
        } else {
            self.black_pieces[PieceType::Queen as usize]
        };

        let rook_attacks = crate::magic::get_rook_attacks(sq, self.all_occupancy);
        if (rook_attacks & (rooks | queens)).count() > 0 {
            return true;
        }

        // 5. Check Bishops/Queens
        let bishops = if is_white_attacker {
            self.white_pieces[PieceType::Bishop as usize]
        } else {
            self.black_pieces[PieceType::Bishop as usize]
        };

        let bishop_attacks = crate::magic::get_bishop_attacks(sq, self.all_occupancy);
        if (bishop_attacks & (bishops | queens)).count() > 0 {
            return true;
        }

        false
    }
}
