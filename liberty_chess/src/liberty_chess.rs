#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
#![allow(clippy::inline_always)]
//! The backend for Liberty Chess

pub use crate::keys::Hash;

use crate::keys::Zobrist;
use crate::parsing::{from_chars, get_indices, process_board, FenError};
use array2d::Array2D;
use moves::Move;
use std::rc::Rc;

/// A struct to represent a clock
pub mod clock;
/// Move representation
pub mod moves;
/// Functions to handle converting information to and from strings
pub mod parsing;
/// A collection of preset positions
pub mod positions;
/// Functions to handle sending boards between threads
pub mod threading;

mod keys;
mod movegen;

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

/// All the pieces available
pub const ALL_PIECES: &str = "kmqcaehuriwbznxlop";

// attack and defence values of pieces
// 0 = empty square
// 1 = None
// 2 = Basic
// 3 = Powerful
const ATTACK: [Piece; 19] = [0, 3, 2, 2, 2, 2, 3, 2, 2, 2, 2, 2, 2, 2, 2, 2, 3, 1, 1];
const DEFENCE: [Piece; 19] = [0, 1, 1, 1, 1, 1, 3, 1, 1, 1, 1, 1, 1, 1, 1, 1, 2, 1, 2];

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
      let promotion = from_chars(fields[7]);
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
  pub const fn to_move(&self) -> bool {
    self.to_move
  }

  /// Get the valid promotion possibilities
  #[must_use]
  pub fn promotion_options(&self) -> &Vec<Piece> {
    &self.promotion_options
  }

  /// Whether the board is waiting for a promotion
  #[must_use]
  pub const fn promotion_available(&self) -> bool {
    self.promotion_target.is_some()
  }

  /// Get the number of halfmoves since last pawn move/capture
  #[must_use]
  pub const fn halfmoves(&self) -> u8 {
    self.halfmoves
  }

  /// Get the number of moves since the start of the game
  #[must_use]
  pub const fn moves(&self) -> u16 {
    self.moves
  }

  /// Get the hash of the current position
  #[must_use]
  pub const fn hash(&self) -> Hash {
    self.hash
  }

  /// Returns the number of non-king pieces on the board
  #[must_use]
  pub const fn pieces(&self) -> (usize, usize) {
    (self.white_pieces, self.black_pieces)
  }

  /// Get the pieces on the board
  #[must_use]
  pub const fn board(&self) -> &Array2D<Piece> {
    &self.pieces
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
  pub const fn state(&self) -> Gamestate {
    self.state
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
          && DEFENCE[target.unsigned_abs() as usize] < ATTACK[BISHOP as usize]
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
      if let Some(en_passant) = self.en_passant {
        keys.update_en_passant(&mut self.hash, en_passant);
        self.en_passant = None;
      }
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
      if let Some(ref mut last_move) = self.last_move {
        last_move.add_promotion(piece);
      }
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

  const fn kings(&self, side: bool) -> &Vec<(usize, usize)> {
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
          self.state = Gamestate::Elimination(false);
          return;
        }
      }
      (false, true) => {
        if self.black_kings.is_empty() {
          self.state = Gamestate::Elimination(true);
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
