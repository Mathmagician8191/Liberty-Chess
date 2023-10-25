use crate::{to_char, to_indices, to_piece, update_column, update_row, Piece};

enum Stage {
  StartCol,
  StartRow,
  EndCol,
  EndRow,
}

/// A struct to represent a move
#[derive(Clone, Copy)]
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

impl Move {
  /// Initialise a new move based on the start and end points
  pub const fn new(start: (usize, usize), end: (usize, usize)) -> Self {
    Self {
      start,
      end,
      promotion: None,
    }
  }

  /// Initialise a move from a string representation
  pub fn load(string: &str) -> Option<Self> {
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
              update_row(&mut end_col, c);
            }
            Stage::EndCol => update_row(&mut end_col, c),
            Stage::EndRow => {
              let promotion = to_piece(c).ok();
              return Some(Self {
                start: (start_col, start_row),
                end: (end_col, end_row),
                promotion,
              });
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
        Stage::StartCol | Stage::StartRow | Stage::EndCol => None,
        Stage::EndRow => Some(Self {
          start: (start_col, start_row),
          end: (end_col, end_row),
          promotion: None,
        }),
      }
    } else {
      None
    }
  }

  /// Make the move include a promotion
  pub fn add_promotion(&mut self, piece: Piece) {
    self.promotion = Some(piece);
  }

  /// Get the start position of the move
  pub const fn start(&self) -> (usize, usize) {
    self.start
  }

  /// Get the end position of the move
  pub const fn end(&self) -> (usize, usize) {
    self.end
  }

  /// Get the promotion involved in the move if there is one
  pub const fn promotion(&self) -> Option<Piece> {
    self.promotion
  }
}
