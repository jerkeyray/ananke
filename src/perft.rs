use crate::board::Board;
use crate::movegen::MoveGenerator;

pub fn perft(board: &Board, depth: u8) -> u64 {
    if depth == 0 {
        return 1;
    }

    let mut nodes = 0;
    let generator = MoveGenerator::new(board);
    let moves = generator.generate_all();

    for m in moves.iter() {
        let next_board = board.make_move(*m);

        let us = board.side_to_move;
        let king_sq = next_board.get_king_square(us);

        if next_board.is_square_attacked(king_sq, next_board.side_to_move) {
            continue;
        }

        nodes += perft(&next_board, depth - 1);
    }

    nodes
}

pub fn perft_driver(board: &Board, depth: u8) {
    println!("\n--- running perft depth {} ---", depth);
    let start = std::time::Instant::now();

    let generator = MoveGenerator::new(board);
    let moves = generator.generate_all();
    let mut total_nodes = 0;

    for m in moves.iter() {
        let next_board = board.make_move(*m);

        let us = board.side_to_move;
        let king_sq = next_board.get_king_square(us);

        // filter illegal moves at root level
        if next_board.is_square_attacked(king_sq, next_board.side_to_move) {
            continue;
        }

        let count = perft(&next_board, depth - 1);
        println!("{:?}: {}", m, count);
        total_nodes += count;
    }

    let duration = start.elapsed();
    println!("\ntotal nodes: {}", total_nodes);
    println!("time: {:.3}s", duration.as_secs_f64());
    println!("nps: {:.0}", total_nodes as f64 / duration.as_secs_f64());
}
