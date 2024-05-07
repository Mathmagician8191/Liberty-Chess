use crate::Board;

/// Run perft on the specified position
#[must_use]
pub fn perft(board: &Board, depth: usize) -> usize {
  match depth {
    0 => 1,
    1 => board.generate_legal().len(),
    _ => {
      let mut result = 0;
      for position in board.generate_legal() {
        result += perft(&position, depth - 1);
      }
      result
    }
  }
}
