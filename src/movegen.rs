use crate::bitboard::{Bitboard, Square};

/// Generate all knight moves from a specific square.
pub fn generate_knight_attacks(sq: Square) -> Bitboard {
  let mut attacks = 0u64;
  let b = 1u64 << (sq as u8);

    // knight jumps defined by bit shifts
    // we mask with specific shifts to prevent wrapping around the board
    const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE; // All bits except File A
    const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F; // All bits except File H
    const NOT_AB_FILE: u64 = 0xFCFCFCFCFCFCFCFC; // All bits except Files A, B
    const NOT_GH_FILE: u64 = 0x3F3F3F3F3F3F3F3F; // All bits except Files G, H

    // North-to-North-East (+17)
    if (b & NOT_H_FILE) != 0 { attacks |= b << 17; }
    // North-North-West (+15)
    if (b & NOT_A_FILE) != 0 { attacks |= b << 15; }
    // South-South-East (-15)
    if (b & NOT_H_FILE) != 0 { attacks |= b >> 15; }
    // South-South-West (-17)
    if (b & NOT_A_FILE) != 0 { attacks |= b >> 17; }

    // North-East-East (+10)
    if (b & NOT_GH_FILE) != 0 { attacks |= b << 10; }
    // North-West-West (+6)
    if (b & NOT_AB_FILE) != 0 { attacks |= b << 6; }
    // South-East-East (-6)
    if (b & NOT_GH_FILE) != 0 { attacks |= b >> 6; }
    // South-West-West (-10)
    if (b & NOT_AB_FILE) != 0 { attacks |= b >> 10; }
    
    Bitboard::new(attacks)
}

/// Generate all king moves from a specific square.
pub fn generate_king_attacks(sq: Square) -> Bitboard {
  let mut attacks = 0u64;
  let b = 1u64 << (sq as u8);
  
  const NOT_A_FILE: u64 = 0xFEFEFEFEFEFEFEFE;
  const NOT_H_FILE: u64 = 0x7F7F7F7F7F7F7F7F;

  // North (+8), South (-8)
  attacks |= b << 8;
  attacks |= b >> 8;

  // East (+1), North-East (+9), South-East (-7) -> Mask Not H
  if (b & NOT_H_FILE) != 0 {
      attacks |= b << 1;
      attacks |= b << 9;
      attacks |= b >> 7;
  }

  // West (-1), North-West (+7), South-West (-9) -> Mask Not A
  if (b & NOT_A_FILE) != 0 {
      attacks |= b >> 1;
      attacks |= b << 7;
      attacks |= b >> 9;
  }

  Bitboard::new(attacks)
}