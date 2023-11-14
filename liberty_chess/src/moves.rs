use crate::parsing::{to_char, to_indices, to_piece, update_column, update_row};
use crate::{Board, Piece};
use std::str::FromStr;

enum Stage {
  StartCol,
  StartRow,
  EndCol,
  EndRow,
}

/// A struct to represent a move
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Move {
  start: (usize, usize),
  end: (usize, usize),
  promotion: Option<Piece>,
}

// Long algebraic notation for ULCI
impl ToString for Move {
  fn to_string(&self) -> String {
    let mut result = format!(
      "{}{}",
      to_indices(self.start.1, self.start.0, self.start.0),
      to_indices(self.end.1, self.end.0, self.end.0),
    );
    if let Some(piece) = self.promotion {
      result.push(to_char(piece).to_ascii_lowercase());
    }
    result
  }
}

impl FromStr for Move {
  type Err = ();

  fn from_str(string: &str) -> Result<Self, Self::Err> {
    if !string.is_empty() && string.parse::<u32>() != Ok(0) {
      let mut start_col = 0;
      let mut start_row = 0;
      let mut end_col = 0;
      let mut end_row = 0;
      let mut stage = Stage::StartCol;
      for c in string.chars() {
        if c.is_ascii_lowercase() {
          match stage {
            Stage::StartCol => update_column(&mut start_col, c),
            Stage::StartRow => {
              stage = Stage::EndCol;
              update_column(&mut end_col, c);
            }
            Stage::EndCol => update_column(&mut end_col, c),
            Stage::EndRow => {
              let promotion = to_piece(c).ok().map(i8::abs);
              return if start_row == 0 || start_col == 0 || end_row == 0 || end_col == 0 {
                Err(())
              } else {
                Ok(Self {
                  start: (start_row - 1, start_col - 1),
                  end: (end_row - 1, end_col - 1),
                  promotion,
                })
              };
            }
          }
        } else if c.is_ascii_digit() {
          match stage {
            Stage::StartCol => {
              stage = Stage::StartRow;
              update_row(&mut start_row, c);
            }
            Stage::StartRow => update_row(&mut start_row, c),
            Stage::EndCol => {
              stage = Stage::EndRow;
              update_row(&mut end_row, c);
            }
            Stage::EndRow => update_row(&mut end_row, c),
          }
        }
      }
      match stage {
        Stage::StartCol | Stage::StartRow | Stage::EndCol => Err(()),
        Stage::EndRow => {
          if start_row == 0 || start_col == 0 || end_row == 0 || end_col == 0 {
            Err(())
          } else {
            Ok(Self {
              start: (start_row - 1, start_col - 1),
              end: (end_row - 1, end_col - 1),
              promotion: None,
            })
          }
        }
      }
    } else {
      Err(())
    }
  }
}

impl Move {
  /// Initialise a new move based on the start and end points
  #[must_use]
  pub const fn new(start: (usize, usize), end: (usize, usize)) -> Self {
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
  #[must_use]
  pub const fn start(&self) -> (usize, usize) {
    self.start
  }

  /// Get the end position of the move
  #[must_use]
  pub const fn end(&self) -> (usize, usize) {
    self.end
  }

  /// Get the promotion involved in the move if there is one
  #[must_use]
  pub const fn promotion(&self) -> Option<Piece> {
    self.promotion
  }
}

impl Board {
  /// Play a move from a move object
  pub fn play_move(&mut self, played_move: Move) {
    self.make_move(played_move.start(), played_move.end());
    if let Some(piece) = played_move.promotion() {
      self.promote(piece);
    }
  }

  /// Return a new board if the move is legal
  #[must_use]
  pub fn move_if_legal(&self, test_move: Move) -> Option<Self> {
    let start = test_move.start();
    let end = test_move.end();
    if start.0 < self.height()
      && start.1 < self.width()
      && end.0 < self.height()
      && end.1 < self.width()
      && self.check_pseudolegal(start, end)
    {
      if let Some(mut board) = self.get_legal(start, end) {
        match (board.promotion_available(), test_move.promotion()) {
          (true, Some(piece)) => {
            board.promote(piece);
            Some(board)
          }
          (false, None) => {
            board.update();
            Some(board)
          }
          (true, None) | (false, Some(_)) => None,
        }
      } else {
        None
      }
    } else {
      None
    }
  }
}
