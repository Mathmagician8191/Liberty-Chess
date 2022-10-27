use crate::FenError::*;
use array2d::Array2D;

pub type Piece = i8;
pub type Hash = u64;

const SQUARE: Piece = 0;
const PAWN: Piece = 1;
const KNIGHT: Piece = 2;
const BISHOP: Piece = 3;
const ROOK: Piece = 4;
const QUEEN: Piece = 5;
const KING: Piece = 6;
const ARCHBISHOP: Piece = 7;
const CHANCELLOR: Piece = 8;
const CAMEL: Piece = 9;
const ZEBRA: Piece = 10;
const MANN: Piece = 11;
const NIGHTRIDER: Piece = 12;
const CHAMPION: Piece = 13;
const CENTAUR: Piece = 14;
const AMAZON: Piece = 15;
const ELEPHANT: Piece = 16;
const OBSTACLE: Piece = 17;
const WALL: Piece = 18;

// attack and defence values of pieces
// 0 = empty square
// 1 = None
// 2 = Basic
// 3 = Powerful
const ATTACK: [Piece; 19] = [0, 2, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 1, 1];
const DEFENCE: [Piece; 19] = [0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2];

#[derive(Debug)]
pub enum FenError {
  InvalidPieceError(char), // encounters a piece that doesn't exist
  NonRectangularError,     // board doesn't have a uniform width
  SizeError,               // board must be 2x2 at minimum
  MissingFieldsError,      // FEN is missing required fields
  InvalidSyntaxError,      // other syntax error
}

impl ToString for FenError {
  fn to_string(&self) -> String {
    match self {
      InvalidPieceError(c) => format!("Invalid piece found: {}", c),
      NonRectangularError => "Non-rectangular board found".to_string(),
      SizeError => "Board is too small (must be at least 2x2)".to_string(),
      MissingFieldsError => "Required field (side to move) missing".to_string(),
      InvalidSyntaxError => "FEN Syntax error".to_string(),
    }
  }
}

#[derive(Clone, Debug)]
pub struct Board {
  pub height: usize,
  pub width: usize,
  pub pieces: Array2D<Piece>,
  pub to_move: bool,
}

fn to_piece(c: char) -> Result<Piece, FenError> {
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
    _ => return Err(InvalidPieceError(c.to_ascii_lowercase())),
  };
  return Ok(multiplier * piece_type);
}

fn to_char(piece: Piece) -> Result<char, FenError> {
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
  return Ok(if piece > 0 { c.to_ascii_uppercase() } else { c });
}

// returns a board with default values for parameters
fn process_board(board: &str) -> Result<Board, FenError> {
  let rows: Vec<&str> = board.split("/").collect();

  let height = rows.len();
  let mut width: Option<usize> = None;
  let mut pieces: Vec<Vec<Piece>> = Vec::new();

  for row in (0..height).rev() {
    let string = rows[row];
    let mut squares = 0;
    let mut vec: Vec<Piece> = Vec::new();

    for c in string.chars() {
      if c.is_digit(10) {
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
        return Err(NonRectangularError);
      }
    } else {
      width = Some(vec.len());
    }
    pieces.push(vec);
  }

  if width < Some(2) || height < 2 {
    return Err(SizeError);
  }
  let width = width.unwrap();

  Ok(Board {
    pieces: Array2D::from_rows(&pieces),
    height,
    width,
    to_move: true,
  })
}

impl Board {
  pub fn new(fen: &str) -> Result<Self, FenError> {
    let fields: Vec<&str> = fen.split(" ").collect();
    if fields.len() < 2 {
      return Err(MissingFieldsError);
    }

    let mut board = process_board(fields[0])?;

    board.to_move = fields[1] == "w";

    // TODO: other fields

    return Ok(board);
  }

  pub fn check_pseudolegal(&self, start: (usize, usize), end: (usize, usize)) -> bool {
    let piece = self.pieces[start];
    if start == end || self.to_move == (piece < 0) {
      return false;
    }
    let destination = self.pieces[end];
    if destination != 0 && (piece > 0) == (destination > 0) {
      return false;
    }
    if DEFENCE[destination.abs() as usize] >= ATTACK[piece.abs() as usize] {
      return false;
    }
    let row_diff = (start.0 as i32 - end.0 as i32).abs();
    let column_diff = (start.1 as i32 - end.1 as i32).abs();
    match piece.abs() {
      //Teleporting pieces
      OBSTACLE => true,
      WALL => true,

      //Jumping pieces
      KNIGHT => (row_diff == 2 && column_diff == 1) || (row_diff == 1 && column_diff == 2),
      CAMEL => (row_diff == 3 && column_diff == 1) || (row_diff == 1 && column_diff == 3),
      ZEBRA => (row_diff == 3 && column_diff == 2) || (row_diff == 2 && column_diff == 3),
      MANN => row_diff <= 1 && column_diff <= 1,
      ELEPHANT => row_diff <= 1 && column_diff <= 1,
      CHAMPION => {
        row_diff <= 2
          && column_diff <= 2
          && (row_diff == 0 || column_diff == 0 || row_diff == column_diff)
      }
      CENTAUR => {
        (row_diff <= 1 && column_diff <= 1)
          || (row_diff == 2 && column_diff == 1)
          || (row_diff == 1 && column_diff == 2)
      }

      // Leaping pieces - TODO
      BISHOP => true,
      ROOK => true,
      QUEEN => true,
      ARCHBISHOP => true,
      CHANCELLOR => true,
      NIGHTRIDER => true,
      AMAZON => true,

      // Special cases - TODO
      PAWN => true,
      KING => row_diff <= 1 && column_diff <= 1,

      _ => unreachable!(),
    }
  }

  pub fn make_move(&mut self, start: (usize, usize), end: (usize, usize)) {
    // TODO: handle special cases
    self.pieces[end] = self.pieces[start];
    self.pieces[start] = SQUARE;
    self.to_move = !self.to_move;
  }
}
