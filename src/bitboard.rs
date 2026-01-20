use std::fmt;
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, Not};

/// represents a square on the board (0-63).
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Square {
    A1 = 0, B1, C1, D1, E1, F1, G1, H1,
    A2, B2, C2, D2, E2, F2, G2, H2,
    A3, B3, C3, D3, E3, F3, G3, H3,
    A4, B4, C4, D4, E4, F4, G4, H4,
    A5, B5, C5, D5, E5, F5, G5, H5,
    A6, B6, C6, D6, E6, F6, G6, H6,
    A7, B7, C7, D7, E7, F7, G7, H7,
    A8, B8, C8, D8, E8, F8, G8, H8,
}

impl Square {
    /// convert a raw index (0-63) to a Square. 
    /// panics in debug mode if index is out of bounds.
    #[inline]
    pub fn new(index: u8) -> Self {
        debug_assert!(index < 64, "Square index out of bounds: {}", index);
        unsafe { std::mem::transmute(index) }
    }
}

/// wraps a u64 to represent a set of squares.
#[derive(PartialEq, Eq, PartialOrd, Clone, Copy, Default)]
pub struct Bitboard(pub u64);

impl Bitboard {
    pub const EMPTY: Bitboard = Bitboard(0); // all bits set to zero
    pub const UNIVERSE: Bitboard = Bitboard(!0); // all bits set to one

    /// create a bitboard from a raw u64.
    #[inline]
    pub fn new(bb: u64) -> Self {
        Bitboard(bb)
    }

    /// set the bit at the given square to 1.
    #[inline]
    pub fn set_bit(&mut self, sq: Square) {
        self.0 |= 1u64 << (sq as u8);
    }

    /// get the state of the bit at the given square.
    #[inline]
    pub fn get_bit(&self, sq: Square) -> bool {
        (self.0 & (1u64 << (sq as u8))) != 0
    }

    /// set the bit at the given square to 0.
    #[inline]
    pub fn clear_bit(&mut self, sq: Square) {
        self.0 &= !(1u64 << (sq as u8));
    }

    /// number of set bits (population count).
    #[inline]
    pub fn count(&self) -> u32 {
        self.0.count_ones()
    }

    /// returns the index of the Least Significant Bit (LSB).
    /// used to iterate over pieces.
    #[inline]
    pub fn lsb_index(&self) -> Option<Square> {
        if self.0 == 0 {
            None
        } else {
            Some(Square::new(self.0.trailing_zeros() as u8))
        }
    }

    /// removes and returns the LSB.
    /// this is the standard way to loop through a bitboard:
    /// `while let Some(sq) = bb.pop_lsb() { ... }`
    #[inline]
    pub fn pop_lsb(&mut self) -> Option<Square> {
        let lsb = self.lsb_index()?;
        self.0 &= self.0 - 1; // standard bit trick to clear LSB
        Some(lsb)
    }
}

// bitwise Operators Implementation
// this allows us to do `bb1 | bb2` or `bb1 & bb2` naturally.

impl BitOr for Bitboard {
    type Output = Self;
    #[inline]
    fn bitor(self, rhs: Self) -> Self { Bitboard(self.0 | rhs.0) }
}

impl BitAnd for Bitboard {
    type Output = Self;
    #[inline]
    fn bitand(self, rhs: Self) -> Self { Bitboard(self.0 & rhs.0) }
}

impl BitXor for Bitboard {
    type Output = Self;
    #[inline]
    fn bitxor(self, rhs: Self) -> Self { Bitboard(self.0 ^ rhs.0) }
}

impl Not for Bitboard {
    type Output = Self;
    #[inline]
    fn not(self) -> Self { Bitboard(!self.0) }
}

impl BitOrAssign for Bitboard {
    #[inline]
    fn bitor_assign(&mut self, rhs: Self) { self.0 |= rhs.0; }
}

impl BitAndAssign for Bitboard {
    #[inline]
    fn bitand_assign(&mut self, rhs: Self) { self.0 &= rhs.0; }
}

// Display trait for visualizing the bitboard in the terminal
impl fmt::Display for Bitboard {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f)?;
        for rank in (0..8).rev() {
            write!(f, " {} ", rank + 1)?;
            for file in 0..8 {
                let sq = Square::new(rank * 8 + file);
                let symbol = if self.get_bit(sq) { "X" } else { "." };
                write!(f, " {} ", symbol)?;
            }
            writeln!(f)?;
        }
        writeln!(f, "    a  b  c  d  e  f  g  h")
    }
}