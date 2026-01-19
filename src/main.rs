use ananke::board::Board;
use ananke::bitboard::Square; 

fn main() {
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    
    println!("Loading FEN: {}", start_fen);
    let board = Board::from_fen(start_fen).expect("Failed to parse FEN");

    println!("--- Board State ---");
    println!("White Pawns: \n{}", board.white_pieces[0]); 
    println!("All Occupancy: \n{}", board.all_occupancy);
}