use array2d::Array2D;
use rand::Rng;
use rand_chacha::rand_core::SeedableRng;
use rand_chacha::ChaChaRng;

pub type Hash = u64;

pub struct ZobristKeys {
  pub colour: Array2D<Hash>,
  pub pieces: Array2D<[Hash; 18]>,
  pub en_passant: Array2D<Hash>,

  pub to_move: Hash,
  pub castling: [Hash; 4],
}

impl ZobristKeys {
  pub fn new(width: usize, height: usize) -> Self {
    // seed generated from random.org
    let mut rng = ChaChaRng::seed_from_u64(0xbe76_25d8_a3ac_f287);
    let mut keys: ZobristKeys = ZobristKeys {
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
}
