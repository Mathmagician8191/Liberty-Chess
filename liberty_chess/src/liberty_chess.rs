//! The backend for Liberty Chess

use array2d::Array2D;

pub use crate::clock::{print_secs, Clock, Type};

pub mod clock;

/// A type used for pieces.
/// Positive values indicate a white piece, negative values indicate a black piece and 0 indicates an empty square.
pub type Piece = i8;
type Hash = u64;

pub const SQUARE: Piece = 0;
pub const PAWN: Piece = 1;
pub const KNIGHT: Piece = 2;
pub const BISHOP: Piece = 3;
pub const ROOK: Piece = 4;
pub const QUEEN: Piece = 5;
pub const KING: Piece = 6;
pub const ARCHBISHOP: Piece = 7;
pub const CHANCELLOR: Piece = 8;
pub const CAMEL: Piece = 9;
pub const ZEBRA: Piece = 10;
pub const MANN: Piece = 11;
pub const NIGHTRIDER: Piece = 12;
pub const CHAMPION: Piece = 13;
pub const CENTAUR: Piece = 14;
pub const AMAZON: Piece = 15;
pub const ELEPHANT: Piece = 16;
pub const OBSTACLE: Piece = 17;
pub const WALL: Piece = 18;

// attack and defence values of pieces
// 0 = empty square
// 1 = None
// 2 = Basic
// 3 = Powerful
const ATTACK: [Piece; 19] = [0, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 1, 1];
const DEFENCE: [Piece; 19] = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2];

/// An enum to represent the reasons for an L-FEN to be invalid.
#[derive(Debug)]
pub enum FenError {
  /// An unrecognised piece was encountered
  InvalidPiece(char),
  /// The board has a non-uniform width
  NonRectangular,
  /// The board is less than 2 units wide on 1 axis
  Size,
  /// Required fields are missing
  MissingFields,
}

impl ToString for FenError {
  fn to_string(&self) -> String {
    match self {
      FenError::InvalidPiece(c) => format!("Invalid piece found: {}", c),
      FenError::NonRectangular => "Non-rectangular board found".to_string(),
      FenError::Size => "Board is too small (must be at least 2x2)".to_string(),
      FenError::MissingFields => "Required field (side to move) missing".to_string(),
    }
  }
}

/// Represents a Liberty chess position
#[derive(Clone, Debug)]
pub struct Board {
  pieces: Array2D<Piece>,
  height: usize,
  width: usize,
  to_move: bool,
  castling: [bool; 4],
  en_passant: Option<[usize; 3]>,
  halfmoves: u8,
  moves: u16,
  pawn_moves: isize,
  pawn_row: usize,
  castle_row: usize,
  queen_column: usize,
  king_column: usize,
}

/// Converts a character to a `Piece`
///
/// # Errors
///
/// Will return `Err` if `c` is not a recognised piece
pub fn to_piece(c: char) -> Result<Piece, FenError> {
  let multiplier: Piece = if c.is_ascii_uppercase() { 1 } else { -1 };

  let piece_type = match c.to_ascii_lowercase() {
    'p' => PAWN,
    'n' => KNIGHT,
    'b' => BISHOP,
    'r' => ROOK,
    'q' => QUEEN,
    'k' => KING,
    'a' => ARCHBISHOP,
    'c' => CHANCELLOR,
    'l' => CAMEL,
    'z' => ZEBRA,
    'x' => MANN,
    'i' => NIGHTRIDER,
    'h' => CHAMPION,
    'u' => CENTAUR,
    'm' => AMAZON,
    'e' => ELEPHANT,
    'o' => OBSTACLE,
    'w' => WALL,
    _ => return Err(FenError::InvalidPiece(c.to_ascii_lowercase())),
  };
  Ok(multiplier * piece_type)
}

fn to_char(piece: Piece) -> char {
  let c = match piece.abs() {
    PAWN => 'p',
    KNIGHT => 'n',
    BISHOP => 'b',
    ROOK => 'r',
    QUEEN => 'q',
    KING => 'k',
    ARCHBISHOP => 'a',
    CHANCELLOR => 'c',
    CAMEL => 'l',
    ZEBRA => 'z',
    MANN => 'x',
    NIGHTRIDER => 'i',
    CHAMPION => 'h',
    CENTAUR => 'u',
    AMAZON => 'm',
    ELEPHANT => 'e',
    OBSTACLE => 'o',
    WALL => 'w',
    _ => unreachable!(),
  };
  if piece > 0 {
    c.to_ascii_uppercase()
  } else {
    c
  }
}

fn get_indices(algebraic: &str) -> Option<[usize; 3]> {
  if algebraic == "-" {
    return None;
  }
  let mut column = 0;
  let mut row = 0;
  let mut row_start = 0;
  let mut found_dash = false;
  let iterator = algebraic.chars();
  for c in iterator {
    match c {
      _ if c.is_ascii_lowercase() => {
        column *= 26;
        column += c as usize + 1 - 'a' as usize;
      }
      _ if c.is_ascii_digit() => {
        if found_dash {
          row_start *= 10;
          row_start += c as usize - '0' as usize;
        } else {
          row *= 10;
          row += c as usize - '0' as usize;
        }
      }
      '-' => found_dash = true,
      _ => (),
    }
  }
  if column == 0 || row == 0 {
    None
  } else if row_start == 0 {
    Some([column - 1, row - 1, row - 1])
  } else {
    Some([column - 1, row - 1, row_start - 1])
  }
}

// returns a board with default values for parameters
fn process_board(board: &str) -> Result<Board, FenError> {
  let rows: Vec<&str> = board.split('/').collect();

  let height = rows.len();
  let mut width: Option<usize> = None;
  let mut pieces: Vec<Vec<Piece>> = Vec::new();

  for row in (0..height).rev() {
    let string = rows[row];
    let mut squares = 0;
    let mut vec: Vec<Piece> = Vec::new();

    for c in string.chars() {
      if c.is_ascii_digit() {
        squares *= 10;
        squares += c as usize - '0' as usize;
      } else {
        if squares > 0 {
          vec.append(&mut vec![0; squares]);
          squares = 0;
        }
        vec.push(to_piece(c)?);
      }
    }
    if squares > 0 {
      vec.append(&mut vec![0; squares]);
    }
    if let Some(i) = width {
      if vec.len() != (i) {
        return Err(FenError::NonRectangular);
      }
    } else {
      width = Some(vec.len());
    }
    pieces.push(vec);
  }

  if width < Some(2) || height < 2 {
    return Err(FenError::Size);
  }
  let width = width.unwrap();

  Ok(Board {
    pieces: Array2D::from_rows(&pieces),
    height,
    width,
    to_move: true,
    castling: [false; 4],
    en_passant: None,
    halfmoves: 0,
    moves: 1,
    pawn_moves: 2,
    pawn_row: 2,
    castle_row: 0,
    queen_column: 0,
    king_column: width - 1,
  })
}

impl Board {
  /// Initialise a new `Board` from an L-FEN
  ///
  /// # Errors
  /// Return an `FenError` if one of the invalid input types mentioned applies.
  ///
  /// # Examples
  /// Getting the start position for standard chess:
  /// ```
  /// liberty_chess::Board::new("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1").unwrap();
  /// ```
  pub fn new(fen: &str) -> Result<Self, FenError> {
    let fields: Vec<&str> = fen.split(' ').collect();
    if fields.len() < 2 {
      return Err(FenError::MissingFields);
    }

    let mut board = process_board(fields[0])?;

    board.to_move = fields[1] == "w";

    if fields.len() > 2 {
      let mut castling = [false; 4];
      for c in fields[2].chars() {
        match c {
          'K' => castling[0] = true,
          'Q' => castling[1] = true,
          'k' => castling[2] = true,
          'q' => castling[3] = true,
          _ => (),
        }
      }
      board.castling = castling;
    }

    if fields.len() > 3 {
      board.en_passant = get_indices(fields[3]);
    }

    if fields.len() > 4 {
      if let Ok(value) = fields[4].parse::<u8>() {
        board.halfmoves = value;
      }
    }

    if fields.len() > 5 {
      if let Ok(value) = fields[5].parse::<u16>() {
        board.moves = value;
      }
    }

    if fields.len() > 6 {
      let data: Vec<&str> = fields[6].split(',').collect();
      if !data.is_empty() {
        if let Ok(pawn_moves) = data[0].parse::<isize>() {
          board.pawn_moves = pawn_moves;
        }
      }
      if data.len() > 1 {
        if let Ok(pawn_row) = data[1].parse::<usize>() {
          board.pawn_row = pawn_row;
        }
      }
      if data.len() > 2 {
        if let Ok(castle_row) = data[2].parse::<usize>() {
          if castle_row > 0 {
            board.castle_row = castle_row - 1;
          }
        }
      }
      if data.len() > 3 {
        if let Ok(queen_column) = data[3].parse::<usize>() {
          if queen_column < board.width {
            board.queen_column = queen_column;
          }
        }
      }
      if data.len() > 4 {
        if let Ok(king_column) = data[4].parse::<usize>() {
          if king_column > 0 && king_column <= board.width {
            board.king_column = king_column - 1;
          }
        }
      }
    }

    Ok(board)
  }

  /// Returns the piece at the given coordinates.
  #[must_use]
  pub fn get_piece(&self, coords: (usize, usize)) -> Piece {
    self.pieces[coords]
  }

  /// The number of ranks the board has
  #[must_use]
  pub fn height(&self) -> usize {
    self.height
  }

  /// The number of columns the board has
  #[must_use]
  pub fn width(&self) -> usize {
    self.width
  }

  /// The side currently to move. `true` indicates white, `false` indicates black.
  #[must_use]
  pub fn to_move(&self) -> bool {
    self.to_move
  }

  /// Checks if a move is psuedo-legal.
  /// Pseudo-legal moves may expose the king to attack but are otherwise legal.
  #[must_use]
  pub fn check_pseudolegal(&self, start: (usize, usize), end: (usize, usize)) -> bool {
    let piece = self.pieces[start];
    if start == end || self.to_move == (piece < 0) {
      return false;
    }
    let destination = self.pieces[end];
    if (destination != 0 && (piece > 0) == (destination > 0))
      || DEFENCE[destination.unsigned_abs() as usize] >= ATTACK[piece.unsigned_abs() as usize]
    {
      return false;
    }
    let istart = (start.0 as isize, start.1 as isize);
    let iend = (end.0 as isize, end.1 as isize);
    let rows = (istart.0 - iend.0).abs();
    let cols = (istart.1 - iend.1).abs();
    match piece.abs() {
      //Teleporting pieces
      OBSTACLE | WALL => true,

      //Jumping pieces
      KNIGHT => (rows == 2 && cols == 1) || (rows == 1 && cols == 2),
      CAMEL => (rows == 3 && cols == 1) || (rows == 1 && cols == 3),
      ZEBRA => (rows == 3 && cols == 2) || (rows == 2 && cols == 3),
      MANN | ELEPHANT => rows <= 1 && cols <= 1,
      CHAMPION => rows <= 2 && cols <= 2 && (rows == 0 || cols == 0 || rows == cols),
      CENTAUR => (rows <= 1 && cols <= 1) || (rows == 2 && cols == 1) || (rows == 1 && cols == 2),

      // Leaping pieces
      BISHOP => rows == cols && self.ray_is_valid(istart, iend, rows),
      ROOK => (rows == 0 || cols == 0) && self.ray_is_valid(istart, iend, isize::max(rows, cols)),
      QUEEN => {
        if rows == 0 || cols == 0 {
          self.ray_is_valid(istart, iend, isize::max(rows, cols))
        } else {
          rows == cols && self.ray_is_valid(istart, iend, rows)
        }
      }
      ARCHBISHOP => {
        (rows == 2 && cols == 1)
          || (rows == 1 && cols == 2)
          || rows == cols && self.ray_is_valid(istart, iend, rows)
      }
      CHANCELLOR => {
        (rows == 2 && cols == 1)
          || (rows == 1 && cols == 2)
          || ((rows == 0 || cols == 0) && self.ray_is_valid(istart, iend, isize::max(rows, cols)))
      }
      NIGHTRIDER => {
        if rows == 2 * cols {
          self.ray_is_valid(istart, iend, cols)
        } else {
          (cols == 2 * rows) && self.ray_is_valid(istart, iend, rows)
        }
      }
      AMAZON => {
        (rows == 2 && cols == 1) || (rows == 1 && cols == 2) || {
          if rows == 0 || cols == 0 {
            self.ray_is_valid(istart, iend, isize::max(rows, cols))
          } else {
            rows == cols && self.ray_is_valid(istart, iend, rows)
          }
        }
      }

      // Special cases - TODO
      PAWN => {
        (end.0 > start.0) == (piece > 0) && {
          match cols {
            0 => {
              destination == 0
                && (rows == 1
                  || (rows <= self.pawn_moves && {
                    let (valid, iter) = if piece > 0 {
                      (start.0 < self.pawn_row, start.0 + 1..end.0 - 1)
                    } else {
                      (
                        self.height - start.0 <= self.pawn_row,
                        end.0 + 1..start.0 - 1,
                      )
                    };
                    if valid {
                      let mut valid = true;
                      for i in iter {
                        if self.pieces[(i, start.1)] != 0 {
                          valid = false;
                          break;
                        }
                      }
                      valid
                    } else {
                      false
                    }
                  }))
            }
            1 => {
              rows == 1
                && (destination != 0 || {
                  if let Some(coords) = self.en_passant {
                    end.1 == coords[0] && coords[1] <= end.0 && end.0 <= coords[2]
                  } else {
                    false
                  }
                })
            }
            _ => false,
          }
        }
      }
      KING => {
        (rows <= 1 && cols <= 1)
          || (start.0 == self.castle_row() && rows == 0 && cols == 2 && {
            let offset = self.castle_offset();
            let (iter, offset) = if start.1 > end.1 {
              // Queenside Castling
              (self.queen_column + 1..start.1, offset + 1)
            } else {
              //Kingside Castling
              (start.1 + 1..self.king_column, offset)
            };
            let mut valid = self.castling[offset];
            if valid {
              for i in iter {
                if self.pieces[(start.0, i)] != 0 {
                  valid = false;
                  break;
                }
              }
            }
            valid
          })
      }

      _ => unreachable!(),
    }
  }

  /// Moves a piece from one square to another.
  /// This function assumes the move is legal.
  pub fn make_move(&mut self, start: (usize, usize), end: (usize, usize)) {
    // TODO: handle promotion, castling and king position updating
    self.halfmoves += 1;
    let piece = self.pieces[start];
    match piece.abs() {
      PAWN => {
        self.halfmoves = 0;
        if start.1 == end.1 {
          let (lowest, highest) = if piece > 0 {
            (start.0, end.0)
          } else {
            (end.0, start.0)
          };
          self.en_passant = if highest - lowest > 1 {
            Some([start.1, lowest + 1, highest - 1])
          } else {
            None
          }
        } else if let Some(coords) = self.en_passant {
          if end.1 == coords[0] && coords[1] <= end.0 && end.0 <= coords[2] {
            if piece > 0 {
              self.pieces[(coords[1] - 1, end.1)] = SQUARE;
            } else {
              self.pieces[(coords[2] + 1, end.1)] = SQUARE;
            }
          }
          self.en_passant = None;
        }
      }
      KING => {
        self.en_passant = None;
        if start.0 == self.castle_row {
          let offset = self.castle_offset();
          self.castling[offset] = false;
          self.castling[offset + 1] = false;
          match start.1 {
            _ if start.1 == end.1 + 2 => {
              // queenside castling
              let rook = (start.0, self.queen_column);
              self.pieces[(start.0, start.1 - 1)] = self.pieces[rook];
              self.pieces[rook] = SQUARE;
            }
            _ if start.1 == end.1 - 2 => {
              // kingside castling
              let rook = (start.0, self.king_column);
              self.pieces[(start.0, start.1 + 1)] = self.pieces[rook];
              self.pieces[rook] = SQUARE;
            }
            _ => (),
          }
        }
      }
      _ => {
        self.en_passant = None;
        if start.0 == self.castle_row {
          let offset = self.castle_offset();
          if start.1 == self.queen_column {
            self.castling[offset + 1] = false;
          } else if start.1 == self.king_column {
            self.castling[offset] = false;
          }
        }
      }
    }
    if self.pieces[end] != 0 {
      self.halfmoves = 0;
    }
    self.pieces[end] = piece;
    self.pieces[start] = SQUARE;
    self.to_move = !self.to_move;
    if self.to_move {
      self.moves += 1;
    }
  }

  fn ray_is_valid(&self, start: (isize, isize), end: (isize, isize), steps: isize) -> bool {
    let dx = (end.0 - start.0) / steps;
    let dy = (end.1 - start.1) / steps;
    for i in 1..steps {
      if self.pieces[((start.0 + i * dx) as usize, (start.1 + i * dy) as usize)] != 0 {
        return false;
      }
    }
    true
  }

  fn castle_offset(&self) -> usize {
    if self.to_move {
      0
    } else {
      2
    }
  }

  fn castle_row(&self) -> usize {
    if self.to_move() {
      self.castle_row
    } else {
      self.width - self.castle_row() - 1
    }
  }
}
