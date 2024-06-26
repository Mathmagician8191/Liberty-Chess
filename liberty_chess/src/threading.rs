use crate::keys::Hash;
use crate::moves::Move;
use crate::{Board, Gamestate, Piece, SharedData, PAWN};
use array2d::Array2D;
use std::rc::Rc;

/// A `Board`, compressed to be sent to another thread
#[derive(Clone)]
pub struct CompressedBoard {
  pieces: Array2D<Piece>,
  to_move: bool,
  castling: u8,
  en_passant: Option<[usize; 3]>,
  halfmoves: u8,
  moves: u32,
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
  white_pieces: u32,
  black_pieces: u32,

  last_move: Option<Move>,
}

impl CompressedBoard {
  /// Load a board sent to another thread
  #[must_use]
  pub fn load_from_thread(self) -> Board {
    let width = self.pieces.num_columns();
    let height = self.pieces.num_rows();

    let mut piece_types = Vec::new();
    for piece in self.pieces.elements_row_major_iter() {
      let piece = piece.abs();
      if piece != 0 && !piece_types.contains(&piece) {
        piece_types.push(piece);
      }
    }
    if piece_types.contains(&PAWN) {
      for promotion in &self.promotion_options {
        if !piece_types.contains(promotion) {
          piece_types.push(*promotion);
        }
      }
    }

    let shared_data = SharedData::new(
      width,
      height,
      &self.white_kings,
      &self.black_kings,
      self.pawn_moves,
      self.pawn_row,
      self.castle_row,
      self.queen_column,
      self.king_column,
      self.promotion_options,
      piece_types,
    );

    Board {
      pieces: self.pieces,
      to_move: self.to_move,
      castling: self.castling,
      en_passant: self.en_passant,
      halfmoves: self.halfmoves,
      moves: self.moves,
      promotion_target: self.promotion_target,
      white_kings: self.white_kings,
      black_kings: self.black_kings,
      state: self.state,
      duplicates: self.duplicates,
      previous: self.previous,
      hash: self.hash,
      shared_data: Rc::new(shared_data),
      friendly_fire: self.friendly_fire,
      white_pieces: self.white_pieces,
      black_pieces: self.black_pieces,
      skip_checkmate: false,
      last_move: self.last_move,
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
      pawn_moves: self.shared_data.pawn_moves,
      pawn_row: self.shared_data.pawn_row,
      castle_row: self.shared_data.castle_row,
      queen_column: self.shared_data.queen_column,
      king_column: self.shared_data.king_column,
      promotion_target: self.promotion_target,
      promotion_options: self.shared_data.promotion_options.clone(),
      white_kings: self.white_kings.clone(),
      black_kings: self.black_kings.clone(),
      state: self.state,
      duplicates: self.duplicates.clone(),
      previous: self.previous.clone(),
      hash: self.hash,
      friendly_fire: self.friendly_fire,
      white_pieces: self.white_pieces,
      black_pieces: self.black_pieces,
      last_move: self.last_move,
    }
  }
}
