use crate::{Board, BISHOP, CAMEL, KING, KNIGHT, OBSTACLE, PAWN, ROOK, WALL, ZEBRA};

impl Board {
  /// Generates all legal moves from a position.
  #[must_use]
  pub fn generate_legal(&self) -> Vec<Self> {
    let mut boards = Vec::new();
    let king_safe = self.attacked_kings().is_empty();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          let mut skip_legality = match piece.abs() {
            KING | BISHOP | PAWN => Some(false),
            _ => {
              if king_safe {
                None
              } else {
                Some(false)
              }
            }
          };
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    if let Some(mut board) = self.get_legal((i, j), (k, l)) {
                      if board.promotion_available() {
                        for piece in self.promotion_options.as_ref() {
                          let mut promotion = board.clone();
                          promotion.promote(*piece);
                          boards.push(promotion);
                        }
                      } else {
                        board.update();
                        boards.push(board);
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_legal(&mut boards, (i, j), (k, j), &mut skip_legality);
              }
              for l in 0..self.width() {
                self.add_if_legal(&mut boards, (i, j), (i, l), &mut skip_legality);
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
          }
        }
      }
    }
    boards
  }

  // inlining gives approx 3-4% speed improvement
  #[inline(always)]
  fn add_if_legal(
    &self,
    boards: &mut Vec<Self>,
    start: (usize, usize),
    end: (usize, usize),
    skip_legality: &mut Option<bool>,
  ) {
    if self.check_pseudolegal(start, end) {
      let skip_legality = skip_legality.unwrap_or_else(|| {
        let bool = !self.is_attacked(start, !self.to_move);
        *skip_legality = Some(bool);
        bool
      });
      if skip_legality {
        let mut board = self.clone();
        board.make_move(start, end);
        board.update();
        boards.push(board);
      } else if let Some(mut board) = self.get_legal(start, end) {
        board.update();
        boards.push(board);
      }
    }
  }

  /// Generates all legal moves from a position.
  ///
  /// Buckets the moves into enemy captures/promotions and other moves.
  #[must_use]
  pub fn generate_legal_buckets(&self) -> (Vec<Self>, Vec<Self>) {
    let mut enemy_captures = Vec::new();
    let mut boards = Vec::new();
    let king_safe = self.attacked_kings().is_empty();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          let mut skip_legality = match piece.abs() {
            KING | BISHOP | PAWN => Some(false),
            _ => {
              if king_safe {
                None
              } else {
                Some(false)
              }
            }
          };
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    if let Some(mut board) = self.get_legal((i, j), (k, l)) {
                      if board.promotion_available() {
                        for piece in self.promotion_options.as_ref() {
                          let mut promotion = board.clone();
                          promotion.promote(*piece);
                          enemy_captures.push(promotion);
                        }
                      } else {
                        board.update();
                        let target = self.pieces[(k, l)];
                        if target != 0 && (piece > 0) ^ (target > 0) {
                          enemy_captures.push(board);
                        } else {
                          boards.push(board);
                        }
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_legal_buckets(
                  &mut enemy_captures,
                  &mut boards,
                  (i, j),
                  (k, j),
                  &mut skip_legality,
                );
              }
              for l in 0..self.width() {
                self.add_if_legal_buckets(
                  &mut enemy_captures,
                  &mut boards,
                  (i, j),
                  (i, l),
                  &mut skip_legality,
                );
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_buckets(
                    &mut enemy_captures,
                    &mut boards,
                    (i, j),
                    (k, l),
                    &mut skip_legality,
                  );
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_buckets(
                    &mut enemy_captures,
                    &mut boards,
                    (i, j),
                    (k, l),
                    &mut skip_legality,
                  );
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_buckets(
                    &mut enemy_captures,
                    &mut boards,
                    (i, j),
                    (k, l),
                    &mut skip_legality,
                  );
                }
              }
            }
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_legal_buckets(
                    &mut enemy_captures,
                    &mut boards,
                    (i, j),
                    (k, l),
                    &mut skip_legality,
                  );
                }
              }
            }
          }
        }
      }
    }
    (enemy_captures, boards)
  }

  // inlining gives approx 3-4% speed improvement
  #[inline(always)]
  fn add_if_legal_buckets(
    &self,
    enemy_captures: &mut Vec<Self>,
    boards: &mut Vec<Self>,
    start: (usize, usize),
    end: (usize, usize),
    skip_legality: &mut Option<bool>,
  ) {
    if self.check_pseudolegal(start, end) {
      let skip_legality = skip_legality.unwrap_or_else(|| {
        let bool = !self.is_attacked(start, !self.to_move);
        *skip_legality = Some(bool);
        bool
      });
      if skip_legality {
        let mut board = self.clone();
        board.make_move(start, end);
        board.update();
        let target = self.pieces[end];
        if target != 0 && (self.pieces[start] > 0) ^ (target > 0) {
          enemy_captures.push(board);
        } else {
          boards.push(board);
        }
      } else if let Some(mut board) = self.get_legal(start, end) {
        board.update();
        let target = self.pieces[end];
        if target != 0 && (self.pieces[start] > 0) ^ (target > 0) {
          enemy_captures.push(board);
        } else {
          boards.push(board);
        }
      }
    }
  }

  /// Generates all captures of enemy pieces and promotions from a position.
  #[must_use]
  pub fn generate_legal_quiescence(&self) -> Vec<Self> {
    let mut boards = Vec::new();
    let king_safe = self.attacked_kings().is_empty();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          let mut skip_legality = match piece.abs() {
            KING | BISHOP | PAWN => Some(false),
            _ => {
              if king_safe {
                None
              } else {
                Some(false)
              }
            }
          };
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    if let Some(mut board) = self.get_legal((i, j), (k, l)) {
                      if board.promotion_available() {
                        for piece in self.promotion_options.as_ref() {
                          let mut promotion = board.clone();
                          promotion.promote(*piece);
                          boards.push(promotion);
                        }
                      } else {
                        board.update();
                        let target = self.pieces[(k, l)];
                        if target != 0 && (piece > 0) ^ (target > 0) {
                          boards.push(board);
                        }
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_legal_quiescence(&mut boards, (i, j), (k, j), &mut skip_legality);
              }
              for l in 0..self.width() {
                self.add_if_legal_quiescence(&mut boards, (i, j), (i, l), &mut skip_legality);
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_quiescence(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_quiescence(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal_quiescence(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            OBSTACLE | WALL => (),
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_legal_quiescence(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
          }
        }
      }
    }
    boards
  }

  // inlining gives approx 3-4% speed improvement
  #[inline(always)]
  fn add_if_legal_quiescence(
    &self,
    boards: &mut Vec<Self>,
    start: (usize, usize),
    end: (usize, usize),
    skip_legality: &mut Option<bool>,
  ) {
    let target = self.pieces[end];
    if target != 0 && (self.pieces[start] > 0) ^ (target > 0) && self.check_pseudolegal(start, end)
    {
      let skip_legality = skip_legality.unwrap_or_else(|| {
        let bool = !self.is_attacked(start, !self.to_move);
        *skip_legality = Some(bool);
        bool
      });
      if skip_legality {
        let mut board = self.clone();
        board.make_move(start, end);
        board.update();
        boards.push(board);
      } else if let Some(mut board) = self.get_legal(start, end) {
        board.update();
        boards.push(board);
      }
    }
  }
}
