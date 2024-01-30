use liberty_chess::OBSTACLE;

pub(crate) const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  43,    // pawn
  309,   // knight
  371,   // bishop
  544,   // rook
  1261,  // queen
  -1360, // king
  944,   // archbishop
  1144,  // chancellor
  82,    // camel
  133,   // zebra
  51,    // mann
  616,   // nightrider
  505,   // champion
  550,   // centaur
  1846,  // amazon
  566,   // elephant
  14,    // obstacle
  65,    // wall
];

const ENDGAME_PIECE_VALUES: [i32; 18] = [
  185,  // pawn
  378,  // knight
  393,  // bishop
  711,  // rook
  1276, // queen
  1222, // king
  1171, // archbishop
  1391, // chancellor
  339,  // camel
  248,  // zebra
  334,  // mann
  369,  // nightrider
  967,  // champion
  1175, // centaur
  1560, // amazon
  604,  // elephant
  100,  // obstacle
  119,  // wall
];

const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [18, -3],    // pawn
  [37, 22],    // knight
  [51, 4],     // bishop
  [38, 30],    // rook
  [14, -1],    // queen
  [-109, -35], // king
  [13, -4],    // archbishop
  [14, -6],    // chancellor
  [-22, -6],   // camel
  [-33, 11],   // zebra
  [45, 0],     // mann
  [16, 11],    // nightrider
  [9, 0],      // champion
  [5, 5],      // centaur
  [9, 13],     // amazon
  [49, 20],    // elephant
  [0, 0],      // obstacle
  [0, 0],      // wall
];

const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [21, -1],  // pawn
  [14, 8],   // knight
  [28, 15],  // bishop
  [60, 8],   // rook
  [55, 35],  // queen
  [65, 20],  // king
  [118, 40], // archbishop
  [10, 109], // chancellor
  [24, 8],   // camel
  [31, 11],  // zebra
  [6, 26],   // mann
  [-16, 2],  // nightrider
  [119, 9],  // champion
  [64, 11],  // centaur
  [30, 55],  // amazon
  [130, 65], // elephant
  [0, 0],    // obstacle
  [0, 0],    // wall
];

const MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  18,  // knight
  15,  // bishop
  20,  // rook
  5,   // queen
  50,  // king
  -1,  // archbishop
  6,   // chancellor
  60,  // camel
  -26, // zebra
  12,  // mann
  0,   // nightrider
  0,   // champion
  0,   // centaur
  0,   // amazon
  0,   // elephant
  0,   // obstacle
  46,  // wall
];

const ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  36,  // pawn
  18,  // knight
  12,  // bishop
  7,   // rook
  -6,  // queen
  -1,  // king
  66,  // archbishop
  78,  // chancellor
  -34, // camel
  52,  // zebra
  -68, // mann
  0,   // nightrider
  61,  // champion
  0,   // centaur
  73,  // amazon
  4,   // elephant
  1,   // obstacle
  11,  // wall
];

const MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  2,   // knight
  12,  // bishop
  -16, // rook
  23,  // queen
  90,  // king
  38,  // archbishop
  14,  // chancellor
  -60, // camel
  -54, // zebra
  58,  // mann
  45,  // nightrider
  54,  // champion
  26,  // centaur
  88,  // amazon
  55,  // elephant
  0,   // obstacle
  24,  // wall
];

const ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  51,  // knight
  119, // bishop
  51,  // rook
  40,  // queen
  41,  // king
  81,  // archbishop
  131, // chancellor
  75,  // camel
  29,  // zebra
  165, // mann
  20,  // nightrider
  84,  // champion
  182, // centaur
  8,   // amazon
  0,   // elephant
  0,   // obstacle
  0,   // wall
];

// advanced pawns get a bonus of 1/(factor * squares_to_promotion + bonus) times the promotion value
const PAWN_SCALING_FACTOR: i32 = 8;
const PAWN_SCALING_BONUS: i32 = 0;

/// Maximum distance from the edge to apply penalty
pub(crate) const EDGE_DISTANCE: usize = 2;

pub(crate) const MIN_HALFMOVES: u8 = 20;
pub(crate) const HALFMOVE_SCALING: u8 = 80;

pub(crate) const ENDGAME_THRESHOLD: i32 = 32;

pub(crate) const ENDGAME_FACTOR: [i32; 18] = [
  0, // pawn
  1, // knight
  1, // bishop
  2, // rook
  4, // queen
  2, // king
  4, // archbishop
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
  pawn_scale_factor: PAWN_SCALING_FACTOR,
  pawn_scaling_bonus: PAWN_SCALING_BONUS,
};

/// Parameters for evaluation
#[derive(Copy, Clone, Debug)]
pub struct Parameters {
  pub(crate) mg_pieces: [i32; 18],
  pub(crate) eg_pieces: [i32; 18],
  pub(crate) mg_edge: [[i32; EDGE_DISTANCE]; 18],
  pub(crate) eg_edge: [[i32; EDGE_DISTANCE]; 18],
  pub(crate) mg_friendly_pawn_penalty: [i32; 18],
  pub(crate) eg_friendly_pawn_penalty: [i32; 18],
  pub(crate) mg_enemy_pawn_penalty: [i32; 18],
  pub(crate) eg_enemy_pawn_penalty: [i32; 18],
  pub(crate) pawn_scale_factor: i32,
  pub(crate) pawn_scaling_bonus: i32,
}

impl Parameters {
  /// How many parameters there are
  pub const COUNT: usize = 182;

  /// Is parameter index valid
  pub fn valid_index(index: usize) -> bool {
    let group = index / 18;
    let index = index % 18;
    match group {
      0 | 1 | 6 | 7 => true,
      2..=5 => index < OBSTACLE as usize - 1,
      8 | 9 => index != 0,
      10 => index < 2,
      _ => panic!("Invalid parameter index"),
    }
  }

  /// How many iterations to run
  pub fn iteration_count(index: usize) -> i32 {
    let group = index / 18;
    match group {
      0 | 1 => 4,
      2..=9 => 2,
      10 => 1,
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
      10 => match index {
        0 => self.pawn_scale_factor += bonus,
        1 => self.pawn_scaling_bonus += bonus,
        _ => panic!("Invalid parameter index"),
      },
      _ => panic!("Invalid parameter index"),
    }
  }
}
