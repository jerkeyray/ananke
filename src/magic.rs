use crate::bitboard::{Bitboard, Square};
use crate::movegen::{generate_bishop_attacks_slow, generate_rook_attacks_slow};

// Simple Xorshift32 random number generator
struct Rng(u32);
impl Rng {
    fn next(&mut self) -> u32 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 17;
        self.0 ^= self.0 << 5;
        self.0
    }
    fn rand_u64(&mut self) -> u64 {
        let n1 = (self.next() as u64) & 0xFFFF;
        let n2 = (self.next() as u64) & 0xFFFF;
        let n3 = (self.next() as u64) & 0xFFFF;
        let n4 = (self.next() as u64) & 0xFFFF;
        n1 | (n2 << 16) | (n3 << 32) | (n4 << 48)
    }
    // Generate a sparse random number (fewer bits set = more magic-like)
    fn rand_sparse(&mut self) -> u64 {
        self.rand_u64() & self.rand_u64() & self.rand_u64()
    }
}

// How many blocker bits each square has (determines table size)
const ROOK_BITS: [u32; 64] = [
    12, 11, 11, 11, 11, 11, 11, 12, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11, 11, 10, 10, 10, 10, 10, 10, 11,
    11, 10, 10, 10, 10, 10, 10, 11, 12, 11, 11, 11, 11, 11, 11, 12,
];

const BISHOP_BITS: [u32; 64] = [
    6, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 7, 9, 9, 7, 5, 5,
    5, 5, 7, 9, 9, 7, 5, 5, 5, 5, 7, 7, 7, 7, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 6, 5, 5, 5, 5, 5, 5, 6,
];

// Precomputed magic attack tables
pub static mut ROOK_MAGICS: [MagicEntry; 64] = [MagicEntry {
    mask: Bitboard(0),
    magic: 0,
    shift: 0,
    offset: 0,
}; 64];
pub static mut BISHOP_MAGICS: [MagicEntry; 64] = [MagicEntry {
    mask: Bitboard(0),
    magic: 0,
    shift: 0,
    offset: 0,
}; 64];

pub static mut ROOK_TABLE: [Bitboard; 102400] = [Bitboard(0); 102400];
pub static mut BISHOP_TABLE: [Bitboard; 5248] = [Bitboard(0); 5248];

#[derive(Copy, Clone, Debug)]
pub struct MagicEntry {
    pub mask: Bitboard, // Relevant occupancy squares
    pub magic: u64,     // Magic multiplier
    pub shift: u32,     // Bits to shift after multiplication
    pub offset: u32,    // Where this square's table starts
}

// fast lookups
pub fn get_rook_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    unsafe {
        let entry = &ROOK_MAGICS[sq as usize];
        let idx = ((blockers.0 & entry.mask.0).wrapping_mul(entry.magic)) >> entry.shift;
        ROOK_TABLE[(entry.offset as usize) + (idx as usize)]
    }
}

pub fn get_bishop_attacks(sq: Square, blockers: Bitboard) -> Bitboard {
    unsafe {
        let entry = &BISHOP_MAGICS[sq as usize];
        let idx = ((blockers.0 & entry.mask.0).wrapping_mul(entry.magic)) >> entry.shift;
        BISHOP_TABLE[(entry.offset as usize) + (idx as usize)]
    }
}

// table generation
fn mask_rook(sq: Square) -> Bitboard {
    let mut mask = Bitboard::EMPTY;
    let (tr, tf) = (sq.rank() as i8, sq.file() as i8);
    for r in (tr + 1)..7 {
        mask.set_bit(Square::new((r * 8 + tf) as u8));
    }
    for r in 1..tr {
        mask.set_bit(Square::new((r * 8 + tf) as u8));
    }
    for f in (tf + 1)..7 {
        mask.set_bit(Square::new((tr * 8 + f) as u8));
    }
    for f in 1..tf {
        mask.set_bit(Square::new((tr * 8 + f) as u8));
    }
    mask
}

fn mask_bishop(sq: Square) -> Bitboard {
    let mut mask = Bitboard::EMPTY;
    let (tr, tf) = (sq.rank() as i8, sq.file() as i8);
    for (dr, df) in [(1, 1), (1, -1), (-1, 1), (-1, -1)] {
        let mut r = tr + dr;
        let mut f = tf + df;
        while r > 0 && r < 7 && f > 0 && f < 7 {
            mask.set_bit(Square::new((r * 8 + f) as u8));
            r += dr;
            f += df;
        }
    }
    mask
}

// Turn an index into an occupancy pattern (which bits are set)
fn get_occupancy_variation(index: usize, bits_in_mask: i32, mask: Bitboard) -> Bitboard {
    let mut occupancy = Bitboard::EMPTY;
    let mut m = mask;
    for i in 0..bits_in_mask {
        let bit_sq = m.pop_lsb().unwrap();
        if (index & (1 << i)) != 0 {
            occupancy.set_bit(bit_sq);
        }
    }
    occupancy
}

// Find a magic number that maps all occupancies to unique attacks
fn find_magic(sq: Square, bits: u32, is_rook: bool) -> (u64, Vec<Bitboard>) {
    let mask = if is_rook {
        mask_rook(sq)
    } else {
        mask_bishop(sq)
    };
    let n = mask.count();
    let num_occupancies = 1 << n;

    // Precompute all occupancy variations and their attacks
    let mut occupancies = vec![Bitboard::EMPTY; num_occupancies];
    let mut attacks = vec![Bitboard::EMPTY; num_occupancies];

    for i in 0..num_occupancies {
        occupancies[i] = get_occupancy_variation(i, n as i32, mask);
        attacks[i] = if is_rook {
            generate_rook_attacks_slow(sq, occupancies[i])
        } else {
            generate_bishop_attacks_slow(sq, occupancies[i])
        };
    }

    let mut rng = Rng(1804289383);
    let size = 1 << bits;
    let mut table = vec![Bitboard::EMPTY; size];

    // Keep trying random numbers until we find one that works
    loop {
        let magic = rng.rand_sparse();

        // Quick filter: good magics spread bits around
        if (mask.0.wrapping_mul(magic) & 0xFF00000000000000).count_ones() < 6 {
            if n >= 6 {
                continue;
            }
        }

        let shift = 64 - bits;
        let mut fail = false;

        for x in table.iter_mut() {
            *x = Bitboard::EMPTY;
        }

        // Try to fill the table
        for i in 0..num_occupancies {
            let idx = (occupancies[i].0.wrapping_mul(magic) >> shift) as usize;
            if table[idx] == Bitboard::EMPTY {
                table[idx] = attacks[i];
            } else if table[idx] != attacks[i] {
                fail = true;
                break;
            }
        }
        if !fail {
            return (magic, table);
        }
    }
}

// initialization
pub fn initialize() {
    println!("Initializing Magic Bitboards...");

    // Build rook tables
    let mut rook_offset = 0;
    for i in 0..64 {
        let sq = Square::new(i);
        let bits = ROOK_BITS[i as usize];
        let (magic, table) = find_magic(sq, bits, true);
        unsafe {
            ROOK_MAGICS[i as usize] = MagicEntry {
                mask: mask_rook(sq),
                magic,
                shift: 64 - bits,
                offset: rook_offset,
            };
            for (j, &att) in table.iter().enumerate() {
                ROOK_TABLE[(rook_offset as usize) + j] = att;
            }
            rook_offset += 1 << bits;
        }
    }

    // Build bishop tables
    let mut bishop_offset = 0;
    for i in 0..64 {
        let sq = Square::new(i);
        let bits = BISHOP_BITS[i as usize];
        let (magic, table) = find_magic(sq, bits, false);
        unsafe {
            BISHOP_MAGICS[i as usize] = MagicEntry {
                mask: mask_bishop(sq),
                magic,
                shift: 64 - bits,
                offset: bishop_offset,
            };
            for (j, &att) in table.iter().enumerate() {
                BISHOP_TABLE[(bishop_offset as usize) + j] = att;
            }
            bishop_offset += 1 << bits;
        }
    }
    println!("Magic initialization complete.");
}
