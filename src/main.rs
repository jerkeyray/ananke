use ananke::movegen;
use ananke::bitboard::Square;

fn main() {
    // Test Knight on E4
    let e4 = Square::new(28); // E4
    let attacks = movegen::generate_knight_attacks(e4);
    
    println!("\nKnight on E4 attacks:");
    println!("{}", attacks);
    
    // Test King on A1 (Corner Case)
    let a1 = Square::new(0);
    let king_attacks = movegen::generate_king_attacks(a1);
    
    println!("King on A1 attacks:");
    println!("{}", king_attacks);
}