use ananke::board::Board;
use ananke::magic;
use ananke::perft;

fn main() {
    magic::initialize();

    // test starting position
    let start_fen = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
    println!("\nloading start position: {}", start_fen);
    let board = Board::from_fen(start_fen).unwrap();
    perft::perft_driver(&board, 1);

    // test position with castling
    let castling_fen = "r3k2r/pppp1ppp/8/4p3/8/8/PPPP1PPP/R3K1NR w KQkq - 0 1";
    println!("\nloading castling test: {}", castling_fen);
    let board = Board::from_fen(castling_fen).unwrap();
    perft::perft_driver(&board, 1);

    // "KiwiPete" - A famous position for debugging move generators.
    let fen = "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1";

    println!("\nloading kiwi pete: {}", fen);
    let board = Board::from_fen(fen).unwrap();

    perft::perft_driver(&board, 2);
}
