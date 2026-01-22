use crate::bitboard::Square;
use std::fmt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Color {
    White,
    Black,
}

impl Color {
    pub fn opposite(&self) -> Self {
        match self {
            Color::White => Color::Black,
            Color::Black => Color::White,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum PieceType {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

/// A compact chess move stored in 16 bits.
/// Layout: [4 flag bits][6 from square][6 to square]
#[derive(Copy, Clone, PartialEq, Eq, Default)]
pub struct Move(u16);

impl Move {
    // Move type flags stored in the top 4 bits
    pub const QUIET: u16 = 0b0000;
    pub const DOUBLE_PAWN_PUSH: u16 = 0b0001;
    pub const K_CASTLE: u16 = 0b0010;
    pub const Q_CASTLE: u16 = 0b0011;
    pub const CAPTURE: u16 = 0b0100;
    pub const EP_CAPTURE: u16 = 0b0101;

    // Promotions (knight, bishop, rook, queen)
    pub const N_PROMO: u16 = 0b1000;
    pub const B_PROMO: u16 = 0b1001;
    pub const R_PROMO: u16 = 0b1010;
    pub const Q_PROMO: u16 = 0b1011;

    // Captures with promotion
    pub const N_PROMO_CAP: u16 = 0b1100;
    pub const B_PROMO_CAP: u16 = 0b1101;
    pub const R_PROMO_CAP: u16 = 0b1110;
    pub const Q_PROMO_CAP: u16 = 0b1111;

    /// Pack from, to, and flags into 16 bits.
    #[inline]
    pub fn new(from: Square, to: Square, flag: u16) -> Self {
        let from_bits = (from as u16) << 6;
        let to_bits = to as u16;
        Move(flag << 12 | from_bits | to_bits)
    }

    /// Extract the source square.
    #[inline]
    pub fn from(&self) -> Square {
        Square::new(((self.0 >> 6) & 0x3F) as u8)
    }

    /// Extract the destination square.
    #[inline]
    pub fn to(&self) -> Square {
        Square::new((self.0 & 0x3F) as u8)
    }

    /// Get the move type flag.
    #[inline]
    pub fn flag(&self) -> u16 {
        self.0 >> 12
    }

    /// Is this a capture?
    #[inline]
    pub fn is_capture(&self) -> bool {
        (self.0 & 0b0100_0000_0000_0000) != 0
    }

    /// Does this involve promotion?
    #[inline]
    pub fn is_promotion(&self) -> bool {
        (self.0 & 0b1000_0000_0000_0000) != 0
    }
}

impl fmt::Debug for Move {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}{:?}", self.from(), self.to())?;
        match self.flag() {
            Self::N_PROMO | Self::N_PROMO_CAP => write!(f, "n"),
            Self::B_PROMO | Self::B_PROMO_CAP => write!(f, "b"),
            Self::R_PROMO | Self::R_PROMO_CAP => write!(f, "r"),
            Self::Q_PROMO | Self::Q_PROMO_CAP => write!(f, "q"),
            _ => Ok(()),
        }
    }
}

/// A stack-allocated move list. Much faster than Vec for perft.
pub struct MoveList {
    pub moves: [Move; 256],
    pub count: usize,
}

impl MoveList {
    pub fn new() -> Self {
        MoveList {
            moves: [Move::default(); 256],
            count: 0,
        }
    }

    #[inline]
    pub fn push(&mut self, m: Move) {
        self.moves[self.count] = m;
        self.count += 1;
    }

    /// Iterate over only the filled moves
    #[inline]
    pub fn iter(&self) -> std::slice::Iter<'_, Move> {
        self.moves[0..self.count].iter()
    }
}
