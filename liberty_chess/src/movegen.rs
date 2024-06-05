use crate::moves::Move;
use crate::{
  Board, BISHOP, CAMEL, CENTAUR, CHAMPION, ELEPHANT, KING, KNIGHT, MANN, OBSTACLE, PAWN, ROOK,
  WALL, ZEBRA,
};

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
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let move_range = if self.to_move {
                let max_row = usize::min(self.height() - 1, i + self.shared_data.pawn_moves);
                let min_row = usize::min(self.height(), i + 1);
                min_row..=max_row
              } else {
                let min_row = i.saturating_sub(self.shared_data.pawn_moves);
                min_row..=(i.saturating_sub(1))
              };
              for k in move_range {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    if let Some(mut board) = self.get_legal((i, j), (k, l)) {
                      if board.promotion_available() {
                        for piece in &self.shared_data.promotion_options {
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
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i, j), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i, j), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            MANN | ELEPHANT => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            CHAMPION => {
              let left_column = j.saturating_sub(2);
              let right_column = usize::min(j + 2, self.width() - 1);
              let left_row = i.saturating_sub(2);
              let right_row = usize::min(i + 2, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            CENTAUR => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
            }
            KING => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_legal(&mut boards, (i, j), (k, l), &mut skip_legality);
                }
              }
              // Castling
              if j >= 2 {
                self.add_if_legal(&mut boards, (i, j), (i, j - 2), &mut skip_legality);
              }
              if j + 2 < self.width() {
                self.add_if_legal(&mut boards, (i, j), (i, j + 2), &mut skip_legality);
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

  /// Generates all pseudolegal moves from a position.
  ///
  /// Buckets the moves into enemy captures/promotions and other moves.
  pub fn generate_pseudolegal(&self, captures: &mut Vec<(Move, u8, u8)>, quiets: &mut Vec<Move>) {
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          match piece.abs() {
            PAWN => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let move_range = if self.to_move {
                let max_row = usize::min(self.height() - 1, i + self.shared_data.pawn_moves);
                let min_row = usize::min(self.height(), i + 1);
                min_row..=max_row
              } else {
                let min_row = i.saturating_sub(self.shared_data.pawn_moves);
                min_row..=(i.saturating_sub(1))
              };
              for k in move_range {
                for l in left_column..=right_column {
                  if self.check_pseudolegal((i, j), (k, l)) {
                    let mv = Move::new((i, j), (k, l));
                    if k == (if self.to_move { self.height() - 1 } else { 0 }) {
                      for piece in &self.shared_data.promotion_options {
                        let mut promotion = mv;
                        promotion.add_promotion(*piece);
                        captures.push((promotion, PAWN as u8, piece.unsigned_abs()));
                      }
                    } else {
                      let target = self.pieces[(k, l)];
                      if target != 0 && (piece > 0) ^ (target > 0) {
                        captures.push((mv, PAWN as u8, target.unsigned_abs()));
                      } else {
                        quiets.push(mv);
                      }
                    }
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                self.add_if_pseudolegal(captures, quiets, (i, j), (k, j));
              }
              for l in 0..self.width() {
                self.add_if_pseudolegal(captures, quiets, (i, j), (i, l));
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i, j), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i, j), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            MANN | ELEPHANT => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            CHAMPION => {
              let left_column = j.saturating_sub(2);
              let right_column = usize::min(j + 2, self.width() - 1);
              let left_row = i.saturating_sub(2);
              let right_row = usize::min(i + 2, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            CENTAUR => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
            KING => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
              // Castling
              if j >= 2 {
                self.add_if_pseudolegal(captures, quiets, (i, j), (i, j - 2));
              }
              if j + 2 < self.width() {
                self.add_if_pseudolegal(captures, quiets, (i, j), (i, j + 2));
              }
            }
            OBSTACLE | WALL => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  let target = self.pieces[(k, l)];
                  if target == 0 {
                    quiets.push(Move::new((i, j), (k, l)));
                  }
                }
              }
            }
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  self.add_if_pseudolegal(captures, quiets, (i, j), (k, l));
                }
              }
            }
          }
        }
      }
    }
  }

  // inlining gives approx 6% speed improvement
  #[inline(always)]
  fn add_if_pseudolegal(
    &self,
    captures: &mut Vec<(Move, u8, u8)>,
    quiets: &mut Vec<Move>,
    start: (usize, usize),
    end: (usize, usize),
  ) {
    if self.check_pseudolegal(start, end) {
      let piece = self.pieces[start];
      let target = self.pieces[end];
      let mv = Move::new(start, end);
      if target != 0 && (piece > 0) ^ (target > 0) {
        captures.push((mv, piece.unsigned_abs(), target.unsigned_abs()));
      } else {
        quiets.push(mv);
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
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let k = if self.to_move {
                usize::min(self.height(), i + 1)
              } else {
                i.saturating_sub(1)
              };
              for l in left_column..=right_column {
                if self.check_pseudolegal((i, j), (k, l)) {
                  let mv = Move::new((i, j), (k, l));
                  if k == (if self.to_move { self.height() - 1 } else { 0 }) {
                    for piece in &self.shared_data.promotion_options {
                      let mut promotion = mv;
                      promotion.add_promotion(*piece);
                      moves.push((promotion, PAWN as u8, piece.unsigned_abs()));
                    }
                  } else {
                    let target = self.pieces[(k, l)];
                    if target != 0 && (piece > 0) ^ (target > 0) {
                      moves.push((mv, PAWN as u8, target.unsigned_abs()));
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
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            CAMEL => {
              for (k, l) in Self::jump_coords((i, j), 3, 1) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            ZEBRA => {
              for (k, l) in Self::jump_coords((i, j), 3, 2) {
                if k < self.height() && l < self.width() {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            KING | MANN | ELEPHANT => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            CHAMPION => {
              let left_column = j.saturating_sub(2);
              let right_column = usize::min(j + 2, self.width() - 1);
              let left_row = i.saturating_sub(2);
              let right_row = usize::min(i + 2, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
            }
            CENTAUR => {
              let left_column = j.saturating_sub(1);
              let right_column = usize::min(j + 1, self.width() - 1);
              let left_row = i.saturating_sub(1);
              let right_row = usize::min(i + 1, self.height() - 1);
              for k in left_row..=right_row {
                for l in left_column..=right_column {
                  self.add_if_pseudolegal_qsearch(&mut moves, (i, j), (k, l));
                }
              }
              for (k, l) in Self::jump_coords((i, j), 2, 1) {
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

  // inlining gives approx 2% speed improvement
  #[inline(always)]
  fn add_if_pseudolegal_qsearch(
    &self,
    moves: &mut Vec<(Move, u8, u8)>,
    start: (usize, usize),
    end: (usize, usize),
  ) {
    let piece = self.pieces[start];
    let target = self.pieces[end];
    if target != 0 && (piece > 0) ^ (target > 0) && self.check_pseudolegal(start, end) {
      let mv = Move::new(start, end);
      moves.push((mv, piece.unsigned_abs(), target.unsigned_abs()));
    }
  }

  /// Generates all recaptures of enemy pieces from a position.
  #[must_use]
  pub fn generate_recaptures(&self, target: (usize, usize)) -> Vec<(Move, u8)> {
    let mut moves = Vec::new();
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) && self.check_pseudolegal((i, j), target) {
          let mv = Move::new((i, j), target);
          moves.push((mv, piece.unsigned_abs()));
        }
      }
    }
    moves
  }
}
