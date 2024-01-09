use crate::Board;

/// Run perft on the specified position
pub fn perft(board: &Board, depth: usize) -> usize {
  if depth > 0 {
    let mut result = 0;
    for position in board.generate_legal() {
      result += perft(&position, depth - 1);
    }
    result
  } else {
    1
  }
}
