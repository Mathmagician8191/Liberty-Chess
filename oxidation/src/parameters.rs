use liberty_chess::OBSTACLE;

pub(crate) const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  79,   // pawn
  271,  // knight
  337,  // bishop
  513,  // rook
  1114, // queen
  -316, // king
  839,  // archbishop
  1012, // chancellor
  132,  // camel
  98,   // zebra
  86,   // mann
  401,  // nightrider
  528,  // champion
  598,  // centaur
  2368, // amazon
  662,  // elephant
  30,   // obstacle
  44,   // wall
];

const ENDGAME_PIECE_VALUES: [i32; 18] = [
  205,  // pawn
  379,  // knight
  401,  // bishop
  683,  // rook
  1129, // queen
  702,  // king
  913,  // archbishop
  1389, // chancellor
  291,  // camel
  219,  // zebra
  308,  // mann
  295,  // nightrider
  701,  // champion
  1031, // centaur
  2818, // amazon
  802,  // elephant
  2,    // obstacle
  149,  // wall
];

const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [14, -2],    // pawn
  [46, 20],    // knight
  [62, 9],     // bishop
  [51, 42],    // rook
  [25, 7],     // queen
  [-93, -34],  // king
  [72, 31],    // archbishop
  [21, 31],    // chancellor
  [-33, 69],   // camel
  [14, 79],    // zebra
  [30, 10],    // mann
  [-101, -68], // nightrider
  [1, 0],      // champion
  [10, 0],     // centaur
  [211, 0],    // amazon
  [107, 56],   // elephant
  [0, 0],      // obstacle
  [0, 0],      // wall
];

const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [29, 5],    // pawn
  [9, 9],     // knight
  [54, 7],    // bishop
  [37, 7],    // rook
  [46, 9],    // queen
  [76, 30],   // king
  [113, -20], // archbishop
  [206, 12],  // chancellor
  [46, -27],  // camel
  [21, -7],   // zebra
  [30, 10],   // mann
  [-26, -5],  // nightrider
  [100, 11],  // champion
  [140, 24],  // centaur
  [239, 0],   // amazon
  [71, 37],   // elephant
  [0, 0],     // obstacle
  [0, 0],     // wall
];

const MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  6,   // pawn
  20,  // knight
  7,   // bishop
  12,  // rook
  48,  // queen
  -21, // king
  13,  // archbishop
  10,  // chancellor
  0,   // camel
  26,  // zebra
  0,   // mann
  0,   // nightrider
  11,  // champion
  0,   // centaur
  1,   // amazon
  0,   // elephant
  0,   // obstacle
  29,  // wall
];

const ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  29, // pawn
  17, // knight
  53, // bishop
  0,  // rook
  0,  // queen
  21, // king
  29, // archbishop
  10, // chancellor
  0,  // camel
  18, // zebra
  80, // mann
  0,  // nightrider
  11, // champion
  51, // centaur
  1,  // amazon
  37, // elephant
  0,  // obstacle
  10, // wall
];

const MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,  // pawn
  28, // knight
  38, // bishop
  0,  // rook
  17, // queen
  37, // king
  20, // archbishop
  0,  // chancellor
  0,  // camel
  0,  // zebra
  41, // mann
  71, // nightrider
  39, // centaur
  64, // champion
  1,  // amazon
  78, // elephant
  0,  // obstacle
  28, // wall
];

const ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,  // pawn
  20, // knight
  78, // bishop
  21, // rook
  45, // queen
  38, // king
  48, // archbishop
  65, // chancellor
  56, // camel
  0,  // zebra
  72, // mann
  84, // nightrider
  68, // centaur
  70, // champion
  1,  // amazon
  97, // elephant
  0,  // obstacle
  0,  // wall
];

/// Maximum distance from the edge to apply penalty
pub(crate) const EDGE_DISTANCE: usize = 2;

pub(crate) const MIN_HALFMOVES: u8 = 20;
pub(crate) const HALFMOVE_SCALING: u8 = 80;

pub(crate) const ENDGAME_THRESHOLD: i32 = 32;

pub(crate) const ENDGAME_FACTOR: [i32; 18] = [
  0, // pawn
  1, //knight
  1, //bishop
  2, // rook
  4, // queen
  2, // king
  4, //archbishop
  4, // chancellor
  1, // camel
  1, // zebra
  1, // mann
  1, // nightrider
  3, // champion
  3, // centaur
  8, // amazon
  2, // elephant
  0, // obstacle
  0, // wall
];

/// The default set of parameters
pub const DEFAULT_PARAMETERS: Parameters = Parameters {
  mg_pieces: MIDDLEGAME_PIECE_VALUES,
  eg_pieces: ENDGAME_PIECE_VALUES,
  mg_edge: MIDDLEGAME_EDGE_AVOIDANCE,
  eg_edge: ENDGAME_EDGE_AVOIDANCE,
  mg_friendly_pawn_penalty: MIDDLEGAME_FRIENDLY_PAWN_PENALTY,
  eg_friendly_pawn_penalty: ENDGAME_FRIENDLY_PAWN_PENALTY,
  mg_enemy_pawn_penalty: MIDDLEGAME_ENEMY_PAWN_PENALTY,
  eg_enemy_pawn_penalty: ENDGAME_ENEMY_PAWN_PENALTY,
};

/// Parameters for evaluation
#[derive(Copy, Clone, Debug)]
pub struct Parameters {
  pub(crate) mg_pieces: [i32; 18],
  pub(crate) eg_pieces: [i32; 18],
  pub(crate) mg_edge: [[i32; EDGE_DISTANCE]; 18],
  pub(crate) eg_edge: [[i32; EDGE_DISTANCE]; 18],
  pub(crate) mg_friendly_pawn_penalty: [i32; 18],
  /// Penalties for pawns blocked by friendly pieces in the endgame
  pub eg_friendly_pawn_penalty: [i32; 18],
  /// Penalties for pawns blocked by enemy pieces in the middlegame
  pub mg_enemy_pawn_penalty: [i32; 18],
  /// Penalties for pawns blocked by enemy pieces in the endgame
  pub eg_enemy_pawn_penalty: [i32; 18],
}

impl Parameters {
  /// How many parameters there are
  pub const COUNT: usize = 180;

  /// Is parameter index valid
  pub fn valid_index(index: usize) -> bool {
    let group = index / 18;
    let index = index % 18;
    match group {
      0 | 1 | 6..=9 => index != OBSTACLE as usize - 1,
      2..=5 => index < OBSTACLE as usize - 1,
      _ => panic!("Invalid parameter index"),
    }
  }

  /// How many iterations to run
  pub fn iteration_count(index: usize) -> i32 {
    let group = index / 18;
    match group {
      0 | 1 => 5,
      2..=9 => 3,
      _ => panic!("Invalid parameter index"),
    }
  }

  /// Set a parameter
  pub fn set_parameter(&mut self, parameter: usize, bonus: i32) {
    let index = parameter % 18;
    let group = parameter / 18;
    match group {
      0 => self.mg_pieces[index] += bonus,
      1 => self.eg_pieces[index] += bonus,
      2 => self.mg_edge[index][0] += bonus,
      3 => self.mg_edge[index][1] += bonus,
      4 => self.eg_edge[index][0] += bonus,
      5 => self.eg_edge[index][1] += bonus,
      6 => self.mg_friendly_pawn_penalty[index] += bonus,
      7 => self.eg_friendly_pawn_penalty[index] += bonus,
      8 => self.mg_enemy_pawn_penalty[index] += bonus,
      9 => self.eg_enemy_pawn_penalty[index] += bonus,
      _ => panic!("Invalid parameter index"),
    }
  }
}
