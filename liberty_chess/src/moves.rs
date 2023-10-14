use crate::Piece;

/// A struct to represent a move
#[derive(Clone, Copy)]
pub struct Move {
  start: (usize, usize),
  end: (usize, usize),
  promotion: Option<Piece>,
}

impl Move {
  /// Initialise a new move based on the start and end points
  pub fn new(start: (usize, usize), end: (usize, usize)) -> Self {
    Self {
      start,
      end,
      promotion: None,
    }
  }

  /// Make the move include a promotion
  pub fn add_promotion(&mut self, piece: Piece) {
    self.promotion = Some(piece);
  }

  /// Get the start position of the move
  pub fn start(&self) -> (usize, usize) {
    self.start
  }

  /// Get the end position of the move
  pub fn end(&self) -> (usize, usize) {
    self.end
  }

  /// Get the promotion involved in the move if there is one
  pub fn promotion(&self) -> Option<Piece> {
    self.promotion
  }
}
