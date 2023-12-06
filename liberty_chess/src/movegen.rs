use crate::moves::Move;
use crate::{Board, BISHOP, CAMEL, KING, KNIGHT, OBSTACLE, PAWN, ROOK, WALL, ZEBRA};

impl Board {
  /// Generates all legal moves from a position.
  #[must_use]
  pub fn generate_legal(&self) -> Vec<Self> {
    let mut boards = Vec::new();
    let king_safe = !self.in_check();
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
                        for piece in &self.shared_data.1 {
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
  pub fn generate_pseudolegal(&self) -> (Vec<(Move, u8, u8)>, Vec<Move>) {
    let mut enemy_captures = Vec::new();
    let mut moves = Vec::new();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    let r#move = Move::new((i, j), (k, l));
                    if k == (if self.to_move { self.height() - 1 } else { 0 }) {
                      for piece in &self.shared_data.1 {
                        let mut promotion = r#move;
                        promotion.add_promotion(*piece);
                        enemy_captures.push((promotion, PAWN as u8, piece.unsigned_abs()));
                      }
                    } else {
                      let target = self.pieces[(k, l)];
                      if target != 0 && (piece > 0) ^ (target > 0) {
                        enemy_captures.push((r#move, PAWN as u8, target.unsigned_abs()));
                      } else {
                        moves.push(r#move);
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (k, j));
              }
              for l in 0..self.width() {
                self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (i, l));
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (k, l));
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (k, l));
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (k, l));
                }
              }
            }
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_pseudolegal(&mut enemy_captures, &mut moves, (i, j), (k, l));
                }
              }
            }
          }
        }
      }
    }
    (enemy_captures, moves)
  }

  // inlining gives approx 3-4% speed improvement
  #[inline(always)]
  fn add_if_pseudolegal(
    &self,
    enemy_captures: &mut Vec<(Move, u8, u8)>,
    moves: &mut Vec<Move>,
    start: (usize, usize),
    end: (usize, usize),
  ) {
    if self.check_pseudolegal(start, end) {
      let piece = self.pieces[start];
      let target = self.pieces[end];
      let r#move = Move::new(start, end);
      if target != 0 && (piece > 0) ^ (target > 0) {
        enemy_captures.push((r#move, piece.unsigned_abs(), target.unsigned_abs()));
      } else {
        moves.push(r#move);
      }
    }
  }

  /// Generates all captures of enemy pieces and promotions from a position.
  #[must_use]
  pub fn generate_qsearch(&self) -> Vec<(Move, u8, u8)> {
    let mut moves = Vec::new();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    let r#move = Move::new((i, j), (k, l));
                    if k == (if self.to_move { self.height() - 1 } else { 0 }) {
                      for piece in &self.shared_data.1 {
                        let mut promotion = r#move;
                        promotion.add_promotion(*piece);
                        moves.push((promotion, PAWN as u8, piece.unsigned_abs()));
                      }
                    } else {
                      let target = self.pieces[(k, l)];
                      if target != 0 && (piece > 0) ^ (target > 0) {
                        moves.push((r#move, PAWN as u8, target.unsigned_abs()));
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, j));
              }
              for l in 0..self.width() {
                self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (i, l));
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            OBSTACLE | WALL => (),
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
          }
        }
      }
    }
    moves
  }

  // inlining gives approx 3-4% speed improvement
  #[inline(always)]
  fn add_if_pseudolegal_qsearch(
    &self,
    moves: &mut Vec<(Move, u8, u8)>,
    start: (usize, usize),
    end: (usize, usize),
  ) {
    if self.check_pseudolegal(start, end) {
      let piece = self.pieces[start];
      let target = self.pieces[end];
      let r#move = Move::new(start, end);
      if target != 0 && (piece > 0) ^ (target > 0) {
        moves.push((r#move, piece.unsigned_abs(), target.unsigned_abs()));
      }
    }
  }
}
