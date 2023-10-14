#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
#![allow(clippy::inline_always)]
//! The backend for Liberty Chess

use crate::keys::{Hash, Zobrist};
use array2d::Array2D;
use core::str::FromStr;
use moves::Move;
use std::rc::Rc;

/// A struct to represent a clock
pub mod clock;
/// Move representation
pub mod moves;

mod keys;

/// A type used for pieces.
/// Positive values indicate a white piece, negative values indicate a black piece and 0 indicates an empty square.
pub type Piece = i8;

/// An empty square
pub const SQUARE: Piece = 0;
/// A pawn with more configuration options
pub const PAWN: Piece = 1;
/// The standard chess knight
pub const KNIGHT: Piece = 2;
/// The standard chess bishop, plus a new move called "El Vaticano"
pub const BISHOP: Piece = 3;
/// The standard chess rook
pub const ROOK: Piece = 4;
/// The standard chess queen
pub const QUEEN: Piece = 5;
/// The standard chess king. Can castle with any piece at the right location.
pub const KING: Piece = 6;
/// Combo of bishop and knight
pub const ARCHBISHOP: Piece = 7;
/// Combo of rook and knight
pub const CHANCELLOR: Piece = 8;
/// Like the knight, but jumping a different number of squares
pub const CAMEL: Piece = 9;
/// Like the knight, but jumping a different number of squares
pub const ZEBRA: Piece = 10;
/// Like a king, but disposable
pub const MANN: Piece = 11;
/// Like a knight, but as a ray attack like a bishop or rook
pub const NIGHTRIDER: Piece = 12;
/// Moves like a mann but up to 2 spaces and can jump
pub const CHAMPION: Piece = 13;
/// Combo of mann and knight
pub const CENTAUR: Piece = 14;
/// Combo of queen and knight
pub const AMAZON: Piece = 15;
/// Like a mann, but immune to attack from most pieces
pub const ELEPHANT: Piece = 16;
/// Teleports to empty squares, but never captures
pub const OBSTACLE: Piece = 17;
/// Like an obstacle, but immune to attack from most pieces
pub const WALL: Piece = 18;

// attack and defence values of pieces
// 0 = empty square
// 1 = None
// 2 = Basic
// 3 = Powerful
const ATTACK: [Piece; 19] = [0, 3, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 1, 1];
const DEFENCE: [Piece; 19] = [0, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2];

/// An enum to represent the reasons for an L-FEN to be invalid.
#[derive(Debug)]
pub enum FenError {
  /// An unrecognised piece was encountered
  InvalidPiece(char),
  /// The board has a non-uniform width
  NonRectangular,
  /// The board has a width or height less than 2
  Size,
}

impl ToString for FenError {
  fn to_string(&self) -> String {
    match self {
      Self::InvalidPiece(c) => format!("Invalid piece found: {c}"),
      Self::NonRectangular => "Non-rectangular board found".to_owned(),
      Self::Size => "Board must be at least 2x2".to_owned(),
    }
  }
}

/// represents the status of the game
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Gamestate {
  /// The game is still ongoing.
  InProgress,
  /// The game is over by checkmate. True = White win, False = Black win
  Checkmate(bool),
  /// The game is drawn by stalemate.
  Stalemate,
  /// The game is drawn by the 50-move rule.
  Repetition,
  /// The game is drawn by 3-fold repitition.
  Move50,
  /// The game is over by elimination of 1 side. True = White win, False = Black win
  Elimination(bool),
  /// The game is drawn by insufficient material
  Material,
}

/// Represents a Liberty chess position
#[derive(Clone)]
pub struct Board {
  pieces: Array2D<Piece>,
  to_move: bool,
  castling: [bool; 4],
  en_passant: Option<[usize; 3]>,
  halfmoves: u8,
  moves: u16,
  pawn_moves: usize,
  pawn_row: usize,
  castle_row: usize,
  queen_column: usize,
  king_column: usize,
  promotion_target: Option<(usize, usize)>,
  promotion_options: Rc<Vec<Piece>>,
  white_kings: Vec<(usize, usize)>,
  black_kings: Vec<(usize, usize)>,
  state: Gamestate,
  duplicates: Vec<Hash>,
  previous: Vec<Hash>,
  hash: Hash,
  keys: Rc<Zobrist>,
  /// Whether friendly fire mode is enabled.
  /// Changing this value is only supported before moves are made.
  pub friendly_fire: bool,

  // Additional cached values
  // Piece counts ignore kings
  white_pieces: usize,
  black_pieces: usize,

  /// The last move the board has recorded
  pub last_move: Option<Move>,
}

impl PartialEq for Board {
  fn eq(&self, other: &Self) -> bool {
    self.hash == other.hash
  }
}

impl Eq for Board {}

impl ToString for Board {
  fn to_string(&self) -> String {
    // save board layout
    let mut rows = Vec::new();
    for row in self.pieces.rows_iter() {
      let mut squares = 0;
      let mut output = String::new();
      for piece in row {
        if piece == &SQUARE {
          squares += 1;
        } else {
          if squares > 0 {
            output.push_str(&squares.to_string());
            squares = 0;
          }
          output.push(to_char(*piece));
        }
      }
      if squares > 0 {
        output.push_str(&squares.to_string());
      }
      rows.push(output);
    }

    rows.reverse();

    let mut result = rows.join("/");

    // save side to move
    result.push_str(if self.to_move { " w " } else { " b " });

    // save castling rights
    let mut needs_castling = true;
    for i in 0..4 {
      if self.castling[i] {
        needs_castling = false;
        result.push(match i {
          0 => 'K',
          1 => 'Q',
          2 => 'k',
          3 => 'q',
          _ => unreachable!(),
        });
      }
    }

    if needs_castling {
      result.push('-');
    }

    result.push(' ');

    // save en passant
    match self.en_passant {
      Some([column, row_min, row_max]) => result.push_str(&to_indices(column, row_min, row_max)),
      None => result.push('-'),
    }

    // save halfmove clock and full move count
    result.push(' ');
    result.push_str(&self.halfmoves.to_string());
    result.push(' ');
    result.push_str(&self.moves.to_string());

    // save optional fields in reverse order
    // because previous ones are required
    let mut optional = Vec::new();

    if self.friendly_fire {
      optional.push("ff".to_owned());
    }

    let custom_promotion =
      self.friendly_fire || self.promotion_options.as_ref() != &[QUEEN, ROOK, BISHOP, KNIGHT];

    // save promotion options
    if custom_promotion {
      let mut promotion = String::new();
      for piece in self.promotion_options.as_ref() {
        promotion.push(to_char(-1 * piece));
      }
      optional.push(promotion);
    }

    // assemble misc options (also reversed)
    let mut misc = Vec::new();
    let mut misc_fields = if self.king_column == self.width() - 1 {
      false
    } else {
      misc.push(self.king_column.to_string());
      true
    };

    if misc_fields || self.queen_column != 0 {
      misc.push(self.queen_column.to_string());
      misc_fields = true;
    }

    if misc_fields || self.castle_row != 0 {
      misc.push((self.castle_row + 1).to_string());
      misc_fields = true;
    }

    if misc_fields || self.pawn_row != 2 {
      misc.push(self.pawn_row.to_string());
      misc_fields = true;
    }

    if misc_fields || self.pawn_moves != 2 {
      misc.push(self.pawn_moves.to_string());
    }

    if misc.is_empty() && custom_promotion {
      misc.push("-".to_owned());
    }

    if !misc.is_empty() {
      misc.reverse();
      optional.push(misc.join(","));
    }

    if !optional.is_empty() {
      optional.reverse();
      result.push(' ');
      result.push_str(&optional.join(" "));
    }

    result
  }
}

impl FromStr for Board {
  type Err = FenError;

  fn from_str(fen: &str) -> Result<Self, Self::Err> {
    Self::new(fen)
  }
}

/// Converts a character to a `Piece`
///
/// # Errors
///
/// Will return `Err` if `c` is not a recognised piece
pub const fn to_piece(c: char) -> Result<Piece, FenError> {
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

#[must_use]
const fn to_char(piece: Piece) -> char {
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

/// Convert a `Piece` into a string representing the type of piece it is.
#[must_use]
pub fn to_name(piece: Piece) -> &'static str {
  match piece.abs() {
    PAWN => "Pawn",
    KNIGHT => "Knight",
    BISHOP => "Bishop",
    ROOK => "Rook",
    QUEEN => "Queen",
    KING => "King",
    ARCHBISHOP => "Archbishop",
    CHANCELLOR => "Chancellor",
    CAMEL => "Camel",
    ZEBRA => "Zebra",
    MANN => "Mann",
    NIGHTRIDER => "Nightrider",
    CHAMPION => "Champion",
    CENTAUR => "Centaur",
    AMAZON => "Amazon",
    ELEPHANT => "Elephant",
    OBSTACLE => "Obstacle",
    WALL => "Wall",
    _ => unreachable!(),
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

const fn get_letter(letter: usize) -> char {
  (letter as u8 + b'a') as char
}

/// Convert a file to its representation as letters
#[must_use]
pub fn to_letters(mut column: usize) -> Vec<char> {
  let mut result = Vec::new();
  while column > 26 - usize::from(result.is_empty()) {
    if !result.is_empty() {
      column -= 1;
    }
    let c = get_letter(column % 26);
    result.push(c);
    column /= 26;
  }
  if !result.is_empty() {
    column -= 1;
  }
  result.push(get_letter(column));
  result.reverse();
  result
}

fn to_indices(column: usize, row_min: usize, row_max: usize) -> String {
  let result = to_letters(column);

  if row_min == row_max {
    format!("{}{}", result.iter().collect::<String>(), row_min + 1)
  } else {
    format!(
      "{}{}-{}",
      result.iter().collect::<String>(),
      row_min + 1,
      row_max + 1
    )
  }
}

// returns a board with default values for parameters
fn process_board(board: &str) -> Result<Board, FenError> {
  let rows: Vec<&str> = board.split('/').collect();

  let height = rows.len();
  let mut width: Option<usize> = None;
  let mut pieces = Vec::new();
  let mut white_kings = Vec::new();
  let mut black_kings = Vec::new();
  let mut white_pieces = 0;
  let mut black_pieces = 0;

  for row in (0..height).rev() {
    let string = rows[row];
    let mut squares = 0;
    let mut vec = Vec::new();

    for c in string.chars() {
      if c.is_ascii_digit() {
        squares *= 10;
        squares += c as usize - '0' as usize;
      } else {
        if squares > 0 {
          vec.append(&mut vec![0; squares]);
          squares = 0;
        }
        let piece = to_piece(c)?;
        if piece.abs() == KING {
          if piece > 0 {
            white_kings.push((height - row - 1, vec.len()));
          } else {
            black_kings.push((height - row - 1, vec.len()));
          }
        } else if piece > 0 {
          white_pieces += 1;
        } else {
          black_pieces += 1;
        }
        vec.push(piece);
      }
    }
    if squares > 0 {
      vec.append(&mut vec![0; squares]);
    }
    if let Some(i) = width {
      if vec.len() != i {
        Err(FenError::NonRectangular)?;
      }
    } else {
      width = Some(vec.len());
    }
    pieces.push(vec);
  }

  let width = width.unwrap_or(0);
  if width < 2 || height < 2 {
    Err(FenError::Size)?;
  }

  Ok(Board {
    pieces: Array2D::from_rows(&pieces),
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
    promotion_target: None,
    promotion_options: Rc::new(vec![QUEEN, ROOK, BISHOP, KNIGHT]),
    white_kings,
    black_kings,
    state: Gamestate::InProgress,
    duplicates: Vec::new(),
    previous: Vec::new(),
    hash: 0,
    keys: Rc::new(Zobrist::new(width, height)),
    friendly_fire: false,
    white_pieces,
    black_pieces,

    last_move: None,
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

    let mut board = process_board(fields[0])?;

    board.to_move = fields.len() == 1 || fields[1] == "w";

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

    if let Some(value) = fields.get(4).and_then(|x| x.parse::<u8>().ok()) {
      board.halfmoves = value;
    }

    if let Some(value) = fields.get(5).and_then(|x| x.parse::<u16>().ok()) {
      board.moves = value;
    }

    if fields.len() > 6 {
      let data: Vec<&str> = fields[6].split(',').collect();
      if let Some(pawn_moves) = data.first().and_then(|x| x.parse::<usize>().ok()) {
        board.pawn_moves = pawn_moves;
      }
      if let Some(pawn_row) = data.get(1).and_then(|x| x.parse::<usize>().ok()) {
        board.pawn_row = pawn_row;
      }
      if let Some(castle_row) = data.get(2).and_then(|x| x.parse::<usize>().ok()) {
        if castle_row > 0 {
          board.castle_row = castle_row - 1;
        }
      }
      if let Some(queen_column) = data.get(3).and_then(|x| x.parse::<usize>().ok()) {
        if queen_column > 0 && queen_column <= board.width() {
          board.queen_column = queen_column - 1;
        }
      }
      if let Some(king_column) = data.get(4).and_then(|x| x.parse::<usize>().ok()) {
        if king_column > 0 && king_column <= board.width() {
          board.king_column = king_column - 1;
        }
      }
    }

    if fields.len() > 7 && !fields[7].is_empty() {
      let mut promotion = Vec::with_capacity(fields[7].len());
      for c in fields[7].chars() {
        promotion.push(to_piece(c)?.abs());
      }
      board.promotion_options = Rc::new(promotion);
    }

    if fields.len() > 8 && fields[8] == "ff" {
      board.friendly_fire = true;
    }

    board.hash = board.get_hash();
    board.update();

    Ok(board)
  }

  /// Returns the piece at the given coordinates.
  #[must_use]
  pub fn get_piece(&self, coords: (usize, usize)) -> Piece {
    self.pieces[coords]
  }

  /// Returns the piece at the given coordinates if the coordinates are valid
  #[must_use]
  pub fn fetch_piece(&self, coords: (usize, usize)) -> Option<&Piece> {
    self.pieces.get(coords.0, coords.1)
  }

  /// The number of ranks the board has
  #[must_use]
  #[inline(always)]
  pub fn height(&self) -> usize {
    self.pieces.num_rows()
  }

  /// The number of columns the board has
  #[must_use]
  #[inline(always)]
  pub fn width(&self) -> usize {
    self.pieces.num_columns()
  }

  /// The side currently to move. `true` indicates white, `false` indicates black.
  #[must_use]
  pub fn to_move(&self) -> bool {
    self.to_move
  }

  /// Get the valid promotion possibilities
  #[must_use]
  pub fn promotion_options(&self) -> &Vec<Piece> {
    &self.promotion_options
  }

  /// Whether the board is waiting for a promotion
  #[must_use]
  pub fn promotion_available(&self) -> bool {
    self.promotion_target.is_some()
  }

  /// The coordinates of the kings under attack.
  /// Only considers the side to move.
  #[must_use]
  pub fn attacked_kings(&self) -> Vec<&(usize, usize)> {
    let mut attacked = Vec::new();
    for king in self.kings(self.to_move()) {
      if self.is_attacked((king.0, king.1), !self.to_move) {
        attacked.push(king);
      }
    }
    attacked
  }

  /// Get the current state of the game
  #[must_use]
  pub fn state(&self) -> Gamestate {
    self.state
  }

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

  /// Checks if a move is psuedo-legal.
  /// Pseudo-legal moves may expose the king to attack but are otherwise legal.
  #[must_use]
  pub fn check_pseudolegal(&self, start: (usize, usize), end: (usize, usize)) -> bool {
    let piece = self.pieces[start];
    if start == end
      || self.to_move == (piece < 0)
      || self.promotion_target.is_some()
      || self.state != Gamestate::InProgress
    {
      return false;
    }
    let destination = self.pieces[end];
    // El Vaticano
    if piece.abs() == BISHOP && piece == destination {
      let rows = start.0.abs_diff(end.0);
      let cols = start.1.abs_diff(end.1);
      if (rows == 2 && cols == 0) || (rows == 0 && cols == 2) {
        let target = self.pieces[((start.0 + end.0) / 2, (start.1 + end.1) / 2)];
        return target != 0
          && DEFENCE[destination.unsigned_abs() as usize] < ATTACK[BISHOP as usize]
          && ((target > 0) != (piece > 0) || self.friendly_fire);
      }
    }
    if ((piece > 0) == (destination > 0) && destination != 0 && !self.friendly_fire)
      || DEFENCE[destination.unsigned_abs() as usize] >= ATTACK[piece.unsigned_abs() as usize]
    {
      return false;
    }
    let istart = (start.0 as isize, start.1 as isize);
    let iend = (end.0 as isize, end.1 as isize);
    let rows = start.0.abs_diff(end.0);
    let cols = start.1.abs_diff(end.1);
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
      ROOK => (rows == 0 || cols == 0) && self.ray_is_valid(istart, iend, usize::max(rows, cols)),
      QUEEN => {
        if rows == 0 || cols == 0 {
          self.ray_is_valid(istart, iend, usize::max(rows, cols))
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
          || ((rows == 0 || cols == 0) && self.ray_is_valid(istart, iend, usize::max(rows, cols)))
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
            self.ray_is_valid(istart, iend, usize::max(rows, cols))
          } else {
            rows == cols && self.ray_is_valid(istart, iend, rows)
          }
        }
      }

      // Special cases
      PAWN => {
        (end.0 > start.0) == (piece > 0) && {
          match cols {
            0 => {
              destination == 0
                && (rows == 1
                  || (rows <= self.pawn_moves && {
                    let (valid, iter) = if piece > 0 {
                      (start.0 < self.pawn_row, start.0 + 1..end.0)
                    } else {
                      (self.height() - start.0 <= self.pawn_row, end.0 + 1..start.0)
                    };
                    valid && {
                      let mut valid = true;
                      for i in iter {
                        if self.pieces[(i, start.1)] != SQUARE {
                          valid = false;
                          break;
                        }
                      }
                      valid
                    }
                  }))
            }
            1 => {
              rows == 1
                && (destination != 0 || {
                  self.en_passant.map_or(false, |coords| {
                    end.1 == coords[0] && coords[1] <= end.0 && end.0 <= coords[2]
                  })
                })
            }
            _ => false,
          }
        }
      }
      KING => {
        (rows <= 1 && cols <= 1)
          || (start.0 == self.castle_row(self.to_move)
            && rows == 0
            && cols == 2
            && !self.attacked_kings().contains(&&start)
            && {
              let offset = Self::castle_offset(self.to_move);
              let (iter, offset) = if start.1 > end.1 {
                // Queenside Castling
                (self.queen_column + 1..start.1, offset + 1)
              } else {
                //Kingside Castling
                (start.1 + 1..self.king_column, offset)
              };
              let mut valid = self.castling[offset]
                && !self.is_attacked((start.0, ((start.1 + end.1) / 2)), !self.to_move);
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
  fn make_move(&mut self, start: (usize, usize), end: (usize, usize)) {
    self.last_move = Some(Move::new(start, end));
    let keys = self.keys.as_ref();
    self.halfmoves += 1;
    self.to_move = !self.to_move;
    self.hash ^= keys.to_move;
    if self.to_move {
      self.moves += 1;
    }
    let piece = self.pieces[start];
    if piece.abs() == BISHOP {
      // Test for El Vaticano
      if start.0 == end.0 {
        self.halfmoves = 0;
        let lowest = usize::min(start.1, end.1);
        let highest = usize::max(start.1, end.1);
        for i in lowest + 1..highest {
          let position = (start.0, i);
          keys.update_hash(&mut self.hash, self.pieces[position], position);
          if self.pieces[position] > 0 {
            self.white_pieces -= 1;
          } else {
            self.black_pieces -= 1;
          }
          self.pieces[position] = SQUARE;
        }
        return;
      } else if start.1 == end.1 {
        self.halfmoves = 0;
        let lowest = usize::min(start.0, end.0);
        let highest = usize::max(start.0, end.0);
        for i in lowest + 1..highest {
          let position = (i, start.1);
          keys.update_hash(&mut self.hash, self.pieces[position], position);
          if self.pieces[position] > 0 {
            self.white_pieces -= 1;
          } else {
            self.black_pieces -= 1;
          }
          self.pieces[position] = SQUARE;
        }
        return;
      }
    }
    keys.update_hash(&mut self.hash, piece, start);
    keys.update_hash(&mut self.hash, piece, end);
    match piece.abs() {
      PAWN => {
        self.halfmoves = 0;
        self.previous = Vec::new();
        self.duplicates = Vec::new();
        if start.1 == end.1 {
          let lowest = usize::min(start.0, end.0);
          let highest = usize::max(start.0, end.0);
          if let Some(en_passant) = self.en_passant {
            keys.update_en_passant(&mut self.hash, en_passant);
          }
          self.en_passant = if highest - lowest > 1 {
            keys.update_en_passant(&mut self.hash, [start.1, lowest + 1, highest - 1]);
            Some([start.1, lowest + 1, highest - 1])
          } else {
            None
          }
        } else if let Some([column, row_min, row_max]) = self.en_passant {
          if end.1 == column && row_min <= end.0 && end.0 <= row_max {
            let (coords, piece) = if piece > 0 {
              let coords = (row_min - 1, end.1);
              (coords, -self.pieces[coords])
            } else {
              let coords = (row_max + 1, end.1);
              self.hash ^= keys.colour[coords];
              (coords, self.pieces[coords])
            };
            self.hash ^= keys.pieces[coords][(piece - 1) as usize];
            if self.pieces[coords] > 0 {
              self.white_pieces -= 1;
            } else {
              self.black_pieces -= 1;
            }
            self.pieces[coords] = SQUARE;
          }
          keys.update_en_passant(&mut self.hash, [column, row_min, row_max]);
          self.en_passant = None;
        }
        if end.0 == (if self.to_move { 0 } else { self.height() - 1 }) {
          self.promotion_target = Some(end);
        }
      }
      KING => {
        if let Some(en_passant) = self.en_passant {
          keys.update_en_passant(&mut self.hash, en_passant);
          self.en_passant = None;
        }
        if start.0 == self.castle_row(!self.to_move) {
          let offset = Self::castle_offset(!self.to_move);
          if self.castling[offset] {
            self.castling[offset] = false;
            self.hash ^= keys.castling[offset];
          }
          if self.castling[offset + 1] {
            self.castling[offset + 1] = false;
            self.hash ^= keys.castling[offset + 1];
          }
          match start.1 {
            _ if start.1 == end.1 + 2 => {
              // queenside castling
              let rook = (start.0, self.queen_column);
              let end = (start.0, start.1 - 1);
              let rook_type = self.pieces[rook];
              keys.update_hash(&mut self.hash, rook_type, rook);
              keys.update_hash(&mut self.hash, rook_type, end);
              self.pieces[end] = rook_type;
              self.pieces[rook] = SQUARE;
            }
            _ if start.1 + 2 == end.1 => {
              // kingside castling
              let rook = (start.0, self.king_column);
              let end = (start.0, start.1 + 1);
              let rook_type = self.pieces[rook];
              keys.update_hash(&mut self.hash, rook_type, rook);
              keys.update_hash(&mut self.hash, rook_type, end);
              self.pieces[end] = rook_type;
              self.pieces[rook] = SQUARE;
            }
            _ => (),
          }
        }
        if piece > 0 {
          for i in 0..self.white_kings.len() {
            self.white_kings[i] = if start == self.white_kings[i] {
              end
            } else {
              self.white_kings[i]
            }
          }
        } else {
          for i in 0..self.black_kings.len() {
            self.black_kings[i] = if start == self.black_kings[i] {
              end
            } else {
              self.black_kings[i]
            }
          }
        }
      }
      _ => {
        if let Some(en_passant) = self.en_passant {
          keys.update_en_passant(&mut self.hash, en_passant);
          self.en_passant = None;
        }
      }
    }
    if start.0 == self.castle_row(!self.to_move) {
      let offset = Self::castle_offset(!self.to_move);
      if start.1 == self.queen_column {
        if self.castling[offset + 1] {
          self.castling[offset + 1] = false;
          self.hash ^= keys.castling[offset + 1];
        }
      } else if start.1 == self.king_column && self.castling[offset] {
        self.castling[offset] = false;
        self.hash ^= keys.castling[offset];
      }
    }
    let capture = self.pieces[end];
    if capture != SQUARE {
      keys.update_hash(&mut self.hash, capture, end);
      if capture > 0 {
        self.white_pieces -= 1;
      } else {
        self.black_pieces -= 1;
      }
      self.halfmoves = 0;
      self.previous = Vec::new();
      self.duplicates = Vec::new();
      if end.0 == self.castle_row(self.to_move) {
        let offset = Self::castle_offset(self.to_move);
        if end.1 == self.queen_column {
          if self.castling[offset + 1] {
            self.castling[offset + 1] = false;
            self.hash ^= keys.castling[offset + 1];
          }
        } else if end.1 == self.king_column && self.castling[offset] {
          self.castling[offset] = false;
          self.hash ^= keys.castling[offset];
        }
      }
    }
    self.pieces[end] = piece;
    self.pieces[start] = SQUARE;
    // Debugging options, enable validation checks that are slower
    #[cfg(feature = "validate")]
    {
      assert_eq!(self.hash, self.get_hash());
      let mut white_pieces = 0;
      let mut black_pieces = 0;
      for piece in self.pieces.elements_row_major_iter() {
        if piece != &0 && piece.abs() != KING {
          if piece > &0 {
            white_pieces += 1;
          } else {
            black_pieces += 1;
          }
        }
      }
      assert_eq!(self.white_pieces, white_pieces);
      assert_eq!(self.black_pieces, black_pieces);
    }
  }

  /// Returns a `Board` if the move is legal, and `None` otherwise.
  /// Assumes the move is psuedo-legal.
  /// Update the board afterwards if there is a result.
  #[must_use]
  pub fn get_legal(&self, start: (usize, usize), end: (usize, usize)) -> Option<Self> {
    let mut board = self.clone();
    board.make_move(start, end);
    for king in board.kings(!board.to_move) {
      if board.is_attacked((king.0, king.1), board.to_move) {
        None?;
      }
    }

    Some(board)
  }

  /// Apply a promotion, if valid in the position.
  /// This function assumes the piece is a valid promotion option.
  pub fn promote(&mut self, piece: Piece) {
    if let Some(target) = self.promotion_target {
      let keys = self.keys.as_ref();
      self.hash ^= keys.pieces[target][(PAWN - 1) as usize];
      self.hash ^= keys.pieces[target][(piece - 1) as usize];
      self.pieces[target] *= piece;
      self.promotion_target = None;
      if piece == KING {
        if self.to_move {
          &mut self.black_kings
        } else {
          &mut self.white_kings
        }
        .push(target);
      }
      self.update();
      if let Some(mut last_move) = self.last_move {
        last_move.add_promotion(piece);
      }
    }
  }

  /// Play a move from a move object
  pub fn play_move(&mut self, played_move: Move) {
    self.make_move(played_move.start(), played_move.end());
    if let Some(piece) = played_move.promotion() {
      self.promote(piece)
    }
  }

  /// Get whether a square is attacked by the specified side.
  #[must_use]
  // automatic flatten is 5% slower
  #[allow(clippy::manual_flatten)]
  // inlining gives approx 2% speed improvement
  #[inline(always)]
  fn is_attacked(&self, (row, column): (usize, usize), side: bool) -> bool {
    let multiplier = if side { 1 } else { -1 };
    for piece in self.straight((row, column), 1) {
      if let Some(piece) = piece {
        match piece * multiplier {
          ROOK | QUEEN | KING | CHANCELLOR | MANN | CHAMPION | CENTAUR | AMAZON | ELEPHANT => {
            return true
          }
          _ => (),
        }
      }
    }
    for piece in self.diagonal((row, column), 1) {
      if let Some(piece) = piece {
        match piece * multiplier {
          BISHOP | QUEEN | KING | ARCHBISHOP | MANN | CHAMPION | CENTAUR | AMAZON | ELEPHANT => {
            return true
          }
          _ => (),
        }
      }
    }
    for piece in self.jumps((row, column), 2, 1) {
      if let Some(piece) = piece {
        match piece * multiplier {
          KNIGHT | ARCHBISHOP | CHANCELLOR | NIGHTRIDER | CENTAUR | AMAZON => return true,
          _ => (),
        }
      }
    }
    for piece in self.jumps((row, column), 3, 1) {
      if piece == Some(&(CAMEL * multiplier)) {
        return true;
      }
    }
    for piece in self.jumps((row, column), 3, 2) {
      if piece == Some(&(ZEBRA * multiplier)) {
        return true;
      }
    }
    for piece in self.straight((row, column), 2) {
      if piece == Some(&(CHAMPION * multiplier)) {
        return true;
      }
    }
    for piece in self.diagonal((row, column), 2) {
      if piece == Some(&(CHAMPION * multiplier)) {
        return true;
      }
    }

    // check for pawn threat
    if self.get(row as isize - multiplier as isize, column as isize - 1)
      == Some(&(PAWN * multiplier))
      || self.get(row as isize - multiplier as isize, column as isize + 1)
        == Some(&(PAWN * multiplier))
    {
      return true;
    }

    for piece in self.straight_rays((row as isize, column as isize), 1) {
      if let Some(piece) = piece {
        match piece * multiplier {
          ROOK | QUEEN | CHANCELLOR | AMAZON => return true,
          _ => (),
        }
      }
    }

    for piece in self.diagonal_rays((row as isize, column as isize), 1) {
      if let Some(piece) = piece {
        match piece * multiplier {
          BISHOP | QUEEN | ARCHBISHOP | AMAZON => return true,
          _ => (),
        }
      }
    }

    for piece in self.all_rays((row as isize, column as isize), 2, 1) {
      if piece == Some(&(NIGHTRIDER * multiplier)) {
        return true;
      }
    }

    false
  }

  fn straight(&self, (row, column): (usize, usize), dx: usize) -> [Option<&Piece>; 4] {
    [
      self.pieces.get(row.wrapping_add(dx), column),
      self.pieces.get(row.wrapping_sub(dx), column),
      self.pieces.get(row, column.wrapping_add(dx)),
      self.pieces.get(row, column.wrapping_sub(dx)),
    ]
  }

  fn diagonal(&self, (row, column): (usize, usize), dx: usize) -> [Option<&Piece>; 4] {
    [
      self
        .pieces
        .get(row.wrapping_add(dx), column.wrapping_add(dx)),
      self
        .pieces
        .get(row.wrapping_add(dx), column.wrapping_sub(dx)),
      self
        .pieces
        .get(row.wrapping_sub(dx), column.wrapping_add(dx)),
      self
        .pieces
        .get(row.wrapping_sub(dx), column.wrapping_sub(dx)),
    ]
  }

  fn jumps(&self, (row, column): (usize, usize), dx: usize, dy: usize) -> [Option<&Piece>; 8] {
    [
      self
        .pieces
        .get(row.wrapping_add(dx), column.wrapping_add(dy)),
      self
        .pieces
        .get(row.wrapping_add(dx), column.wrapping_sub(dy)),
      self
        .pieces
        .get(row.wrapping_sub(dx), column.wrapping_add(dy)),
      self
        .pieces
        .get(row.wrapping_sub(dx), column.wrapping_sub(dy)),
      self
        .pieces
        .get(row.wrapping_add(dy), column.wrapping_add(dx)),
      self
        .pieces
        .get(row.wrapping_add(dy), column.wrapping_sub(dx)),
      self
        .pieces
        .get(row.wrapping_sub(dy), column.wrapping_add(dx)),
      self
        .pieces
        .get(row.wrapping_sub(dy), column.wrapping_sub(dx)),
    ]
  }

  #[allow(clippy::cast_sign_loss)]
  const fn jump_coords((row, column): (isize, isize), dx: isize, dy: isize) -> [(usize, usize); 8] {
    [
      ((row + dx) as usize, (column + dy) as usize),
      ((row + dx) as usize, (column - dy) as usize),
      ((row - dx) as usize, (column + dy) as usize),
      ((row - dx) as usize, (column - dy) as usize),
      ((row + dy) as usize, (column + dx) as usize),
      ((row + dy) as usize, (column - dx) as usize),
      ((row - dy) as usize, (column + dx) as usize),
      ((row - dy) as usize, (column - dx) as usize),
    ]
  }

  fn diagonal_rays(&self, (row, column): (isize, isize), dx: isize) -> [Option<&Piece>; 4] {
    [
      self.ray((row, column), dx, dx),
      self.ray((row, column), dx, -dx),
      self.ray((row, column), -dx, dx),
      self.ray((row, column), -dx, -dx),
    ]
  }

  fn straight_rays(&self, (row, column): (isize, isize), dx: isize) -> [Option<&Piece>; 4] {
    [
      self.ray((row, column), dx, 0),
      self.ray((row, column), -dx, 0),
      self.ray((row, column), 0, dx),
      self.ray((row, column), 0, -dx),
    ]
  }

  fn all_rays(&self, (row, column): (isize, isize), dx: isize, dy: isize) -> [Option<&Piece>; 8] {
    [
      self.ray((row, column), dx, dy),
      self.ray((row, column), dx, -dy),
      self.ray((row, column), -dx, dy),
      self.ray((row, column), -dx, -dy),
      self.ray((row, column), dy, dx),
      self.ray((row, column), dy, -dx),
      self.ray((row, column), -dy, dx),
      self.ray((row, column), -dy, -dx),
    ]
  }

  fn ray(&self, (mut row, mut column): (isize, isize), dx: isize, dy: isize) -> Option<&Piece> {
    loop {
      row += dx;
      column += dy;
      match self.get(row, column) {
        Some(&SQUARE) => (),
        piece => return piece,
      }
    }
  }

  fn get(&self, row: isize, column: isize) -> Option<&Piece> {
    self.pieces.get(row as usize, column as usize)
  }

  fn ray_is_valid(&self, start: (isize, isize), end: (isize, isize), steps: usize) -> bool {
    let dx = (end.0 - start.0) / steps as isize;
    let dy = (end.1 - start.1) / steps as isize;
    for i in 1..steps as isize {
      if self.pieces[((start.0 + i * dx) as usize, (start.1 + i * dy) as usize)] != SQUARE {
        return false;
      }
    }
    true
  }

  const fn castle_offset(side: bool) -> usize {
    if side {
      0
    } else {
      2
    }
  }

  fn castle_row(&self, side: bool) -> usize {
    if side {
      self.castle_row
    } else {
      self.height() - self.castle_row - 1
    }
  }

  fn kings(&self, side: bool) -> &Vec<(usize, usize)> {
    if side {
      &self.white_kings
    } else {
      &self.black_kings
    }
  }

  /// Update kings in check and game state.
  pub fn update(&mut self) {
    match (self.white_pieces == 0, self.black_pieces == 0) {
      (true, true) => return self.state = Gamestate::Material,
      (true, false) => {
        if self.white_kings.is_empty() {
          self.state = Gamestate::Elimination(true);
          return;
        }
      }
      (false, true) => {
        if self.black_kings.is_empty() {
          self.state = Gamestate::Elimination(false);
          return;
        }
      }
      (false, false) => (),
    }
    if !self.any_moves() {
      self.state = if self.attacked_kings().is_empty() {
        Gamestate::Stalemate
      } else {
        Gamestate::Checkmate(!self.to_move)
      }
    } else if self.halfmoves > 100 {
      self.state = Gamestate::Move50;
    } else if self.duplicates.contains(&self.hash) {
      self.state = Gamestate::Repetition;
    } else if self.previous.contains(&self.hash) {
      self.duplicates.push(self.hash);
    } else {
      self.previous.push(self.hash);
    }
  }

  #[must_use]
  fn get_hash(&self) -> Hash {
    let mut result = 0;
    let keys = self.keys.as_ref();

    if self.to_move {
      result ^= keys.to_move;
    }

    for i in 0..4 {
      if self.castling[i] {
        result ^= keys.castling[i];
      }
    }

    if let Some(en_passant) = self.en_passant {
      keys.update_en_passant(&mut result, en_passant);
    }

    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        keys.update_hash(&mut result, piece, (i, j));
      }
    }

    result
  }

  #[must_use]
  fn any_moves(&self) -> bool {
    for i in 0..self.height() {
      for j in 0..self.width() {
        let piece = self.pieces[(i, j)];
        if piece != 0 && self.to_move == (piece > 0) {
          // TODO: movegen based on piece type
          match piece.abs() {
            PAWN => {
              let left_column = usize::saturating_sub(j, 1);
              let right_column = usize::min(j + 1, self.width() - 1);
              for k in 0..self.height() {
                for l in left_column..=right_column {
                  if self.test_legal((i, j), (k, l)) {
                    return true;
                  }
                }
              }
            }
            ROOK => {
              for k in 0..self.height() {
                if self.test_legal((i, j), (k, j)) {
                  return true;
                }
              }
              for l in 0..self.width() {
                if self.test_legal((i, j), (i, l)) {
                  return true;
                }
              }
            }
            KNIGHT => {
              for (k, l) in Self::jump_coords((i as isize, j as isize), 2, 1) {
                if k < self.height() && l < self.width() && self.test_legal((i, j), (k, l)) {
                  return true;
                }
              }
            }
            _ => {
              for k in 0..self.height() {
                for l in 0..self.width() {
                  if self.test_legal((i, j), (k, l)) {
                    return true;
                  }
                }
              }
            }
          }
        }
      }
    }
    false
  }

  fn test_legal(&self, start: (usize, usize), end: (usize, usize)) -> bool {
    self.check_pseudolegal(start, end) && self.get_legal(start, end).is_some()
  }
}
