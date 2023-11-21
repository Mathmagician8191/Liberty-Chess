use std::rc::Rc;

use crate::keys::{Hash, Zobrist};
use crate::{Board, Gamestate, Piece};
use array2d::Array2D;

/// A `Board`, compressed to be sent to another thread
#[derive(Clone)]
pub struct CompressedBoard {
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
  promotion_options: Vec<Piece>,
  white_kings: Vec<(usize, usize)>,
  black_kings: Vec<(usize, usize)>,
  state: Gamestate,
  duplicates: Vec<Hash>,
  previous: Vec<Hash>,
  hash: Hash,
  /// Whether friendly fire mode is enabled.
  /// Changing this value is only supported before moves are made.
  pub friendly_fire: bool,

  // Additional cached values
  // Piece counts ignore kings
  white_pieces: usize,
  black_pieces: usize,
}

impl CompressedBoard {
  /// Load a board sent to another thread
  #[must_use]
  pub fn load_from_thread(self) -> Board {
    let width = self.pieces.num_columns();
    let height = self.pieces.num_rows();
    Board {
      pieces: self.pieces,
      to_move: self.to_move,
      castling: self.castling,
      en_passant: self.en_passant,
      halfmoves: self.halfmoves,
      moves: self.moves,
      pawn_moves: self.pawn_moves,
      pawn_row: self.pawn_row,
      castle_row: self.castle_row,
      queen_column: self.queen_column,
      king_column: self.king_column,
      promotion_target: self.promotion_target,
      promotion_options: Rc::new(self.promotion_options),
      white_kings: self.white_kings,
      black_kings: self.black_kings,
      state: self.state,
      duplicates: self.duplicates,
      previous: self.previous,
      hash: self.hash,
      keys: Rc::new(Zobrist::new(width, height)),
      friendly_fire: self.friendly_fire,
      white_pieces: self.white_pieces,
      black_pieces: self.black_pieces,
      last_move: None,
    }
  }
}

impl Board {
  /// Using an `Arc` has performance impacts but an `Rc` doesn't allow the `Board` to be shared between threads.
  /// This is a workaround to create the `Rc` data on the new thread
  #[must_use]
  pub fn send_to_thread(&self) -> CompressedBoard {
    CompressedBoard {
      pieces: self.pieces.clone(),
      to_move: self.to_move,
      castling: self.castling,
      en_passant: self.en_passant,
      halfmoves: self.halfmoves,
      moves: self.moves,
      pawn_moves: self.pawn_moves,
      pawn_row: self.pawn_row,
      castle_row: self.castle_row,
      queen_column: self.queen_column,
      king_column: self.king_column,
      promotion_target: self.promotion_target,
      promotion_options: self.promotion_options.to_vec(),
      white_kings: self.white_kings.clone(),
      black_kings: self.black_kings.clone(),
      state: self.state,
      duplicates: self.duplicates.clone(),
      previous: self.previous.clone(),
      hash: self.hash,
      friendly_fire: self.friendly_fire,
      white_pieces: self.white_pieces,
      black_pieces: self.black_pieces,
    }
  }
}
