use liberty_chess::moves::Move;
use liberty_chess::Board;
use rand::seq::SliceRandom;
use rand::thread_rng;

pub trait Engine {
  const NAME: &'static str;

  fn best_move(&self, board: &Board) -> Option<Move>;
}

pub fn random_move(board: &Board) -> Option<Move> {
  let moves = board.generate_legal();
  moves.choose(&mut thread_rng())?.last_move
}
