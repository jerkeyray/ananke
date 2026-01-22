use ananke::board::Board;
use ananke::magic;
use ananke::movegen::MoveGenerator;

fn main() {
    magic::initialize();

    // 1. Start Position
    let fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    let board = Board::from_fen(fen).unwrap();

    println!("\nGenerating moves for start position...");
    let movegen = MoveGenerator::new(&board);
    let list = movegen.generate_all();

    println!("Found {} moves:", list.count);
    for m in list.iter() {
        print!("{:?} ", m);
    }
    println!();
}
