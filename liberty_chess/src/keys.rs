#![allow(clippy::inline_always)]

use crate::{Board, Piece, SQUARE};
use array2d::Array2D;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

/// A hash of a position
pub type Hash = u64;

pub struct Zobrist {
  pub colour: Array2D<Hash>,
  pub pieces: Array2D<[Hash; 18]>,
  en_passant: Array2D<Hash>,
  pub to_move: Hash,
  pub castling: [Hash; 4],
}

impl Zobrist {
  pub fn new(width: usize, height: usize) -> Self {
    // seed generated from random.org
    let mut rng = ChaChaRng::seed_from_u64(0xbe76_25d8_a3ac_f287);
    let mut keys = Self {
      colour: Array2D::filled_with(0, height, width),
      pieces: Array2D::filled_with([0; 18], height, width),
      en_passant: Array2D::filled_with(0, height, width),
      to_move: rng.gen(),
      castling: [0; 4],
    };

    rng.fill(&mut keys.castling);

    for i in 0..height {
      for j in 0..width {
        keys.colour[(i, j)] = rng.gen();
        rng.fill(&mut keys.pieces[(i, j)]);
        keys.en_passant[(i, j)] = rng.gen();
      }
    }

    keys
  }

  // inlining gives approx 2% performance improvement
  // same performance as when manually inlined, but with reliability and readability gain
  #[inline(always)]
  pub fn update_hash(&self, hash: &mut Hash, piece: Piece, index: (usize, usize)) {
    if piece > 0 {
      *hash ^= self.colour[index];
    }
    if piece != SQUARE {
      *hash ^= self.pieces[index][(piece.unsigned_abs() - 1) as usize];
    }
  }

  pub fn update_en_passant(&self, hash: &mut Hash, [column, row_min, row_max]: [usize; 3]) {
    *hash ^= self.en_passant[(row_min, column)];
    if row_min != row_max {
      *hash ^= self.en_passant[(row_max, column)];
    }
  }
}

/// Things not included in Zobrist Hash
#[derive(Eq, PartialEq)]
pub struct ExtraFlags {
  promotion_options: Vec<Piece>,
  pawn_moves: usize,
  pawn_row: usize,
  castle_row: usize,
  queen_column: usize,
  king_column: usize,
  friendly_fire: bool,
}

impl ExtraFlags {
  /// Extract the flags from a board
  #[must_use]
  pub fn new(board: &Board) -> Self {
    Self {
      promotion_options: board.shared_data.1.to_vec(),
      pawn_moves: board.pawn_moves,
      pawn_row: board.pawn_row,
      castle_row: board.castle_row,
      queen_column: board.queen_column,
      king_column: board.king_column,
      friendly_fire: board.friendly_fire,
    }
  }
}
