use liberty_chess::OBSTACLE;

pub(crate) const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  64,    // pawn
  331,   // knight
  407,   // bishop
  570,   // rook
  1239,  // queen
  -1635, // king
  966,   // archbishop
  1113,  // chancellor
  182,   // camel
  177,   // zebra
  148,   // mann
  657,   // nightrider
  556,   // champion
  585,   // centaur
  1841,  // amazon
  651,   // elephant
  0,     // obstacle
  53,    // wall
];

const ENDGAME_PIECE_VALUES: [i32; 18] = [
  179,  // pawn
  368,  // knight
  391,  // bishop
  702,  // rook
  1286, // queen
  1297, // king
  1079, // archbishop
  1400, // chancellor
  297,  // camel
  221,  // zebra
  357,  // mann
  375,  // nightrider
  957,  // champion
  1175, // centaur
  1594, // amazon
  651,  // elephant
  79,   // obstacle
  131,  // wall
];

const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [30, 13, 18, -6, -5],         // pawn
  [43, 61, 34, 28, 17],         // knight
  [156, 69, 66, 17, 5],         // bishop
  [77, 70, 27, 39, 23],         // rook
  [20, 36, 28, 12, 2],          // queen
  [-210, -153, -111, -34, -32], // king
  [118, 34, 34, 7, 5],          // archbishop
  [14, 25, 8, -4, -11],         // chancellor
  [-30, 66, 26, 26, -2],        // camel
  [9, 14, -18, 13, -24],        // zebra
  [0, 0, 0, 0, 0],              // mann
  [-24, -62, -26, -57, -63],    // nightrider
  [21, 0, 0, 0, 7],             // champion
  [40, 33, 34, 23, 18],         // centaur
  [92, 114, 13, 39, 0],         // amazon
  [118, 139, 54, 61, 40],       // elephant
  [0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0],
];

const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [182, 18, 25, -8, 5],    // pawn
  [36, 0, 8, 13, -1],      // knight
  [1, 50, 40, 18, 21],     // bishop
  [89, 29, 22, 16, 3],     // rook
  [119, 47, 50, 59, 28],   // queen
  [166, 106, 67, 26, 22],  // king
  [255, 169, 107, 21, 0],  // archbishop
  [49, 59, 36, 84, 7],     // chancellor
  [35, -3, -4, 16, -18],   // camel
  [29, -28, -4, 49, 9],    // zebra
  [68, 32, 31, 0, 0],      // mann
  [22, 44, 25, 76, 38],    // nightrider
  [173, 95, 85, 0, 55],    // champion
  [370, 173, 90, 70, 0],   // centaur
  [141, 52, 78, -56, -78], // amazon
  [159, 144, 140, 73, 60], // elephant
  [0, 0, 0, 0, 0],
  [0, 0, 0, 0, 0],
];

const MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  -14, // pawn
  14,  // knight
  11,  // bishop
  12,  // rook
  5,   // queen
  50,  // king
  19,  // archbishop
  16,  // chancellor
  0,   // camel
  0,   // zebra
  0,   // mann
  0,   // nightrider
  2,   // champion
  5,   // centaur
  1,   // amazon
  0,   // elephant
  1,   // obstacle
  7,   // wall
];

const ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  54, // pawn
  17, // knight
  12, // bishop
  1,  // rook
  0,  // queen
  -1, // king
  62, // archbishop
  98, // chancellor
  0,  // camel
  19, // zebra
  0,  // mann
  0,  // nightrider
  0,  // champion
  34, // centaur
  0,  // amazon
  27, // elephant
  0,  // obstacle
  13, // wall
];

const MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  19,  // knight
  14,  // bishop
  -31, // rook
  17,  // queen
  115, // king
  31,  // archbishop
  -9,  // chancellor
  39,  // camel
  3,   // zebra
  40,  // mann
  80,  // nightrider
  44,  // champion
  47,  // centaur
  0,   // amazon
  60,  // elephant
  9,   // obstacle
  19,  // wall
];

const ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  41,  // knight
  123, // bishop
  64,  // rook
  26,  // queen
  56,  // king
  146, // archbishop
  64,  // chancellor
  0,   // camel
  26,  // zebra
  173, // mann
  24,  // nightrider
  0,   // champion
  98,  // centaur
  131, // amazon
  40,  // elephant
  27,  // obstacle
  0,   // wall
];

// advanced pawns get a bonus of 1/(factor * squares_to_promotion + bonus) times the promotion value
const PAWN_SCALING_FACTOR: i32 = 8;
const PAWN_SCALING_BONUS: i32 = -1;

pub(crate) const TEMPO_BONUS: i32 = 10;

/// Maximum distance from the edge to apply penalty
pub(crate) const EDGE_DISTANCE: usize = 2;
pub(crate) const EDGE_PARAMETER_COUNT: usize = EDGE_DISTANCE * (EDGE_DISTANCE + 3) / 2;
pub(crate) const INDEXING: [usize; (EDGE_DISTANCE + 1) * (EDGE_DISTANCE + 1)] =
  [0, 1, 2, 1, 3, 4, 2, 4, 5];

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
  pub(crate) mg_edge: [[i32; EDGE_PARAMETER_COUNT]; 18],
  pub(crate) eg_edge: [[i32; EDGE_PARAMETER_COUNT]; 18],
  pub(crate) mg_friendly_pawn_penalty: [i32; 18],
  pub(crate) eg_friendly_pawn_penalty: [i32; 18],
  pub(crate) mg_enemy_pawn_penalty: [i32; 18],
  pub(crate) eg_enemy_pawn_penalty: [i32; 18],
  pub(crate) pawn_scale_factor: i32,
  pub(crate) pawn_scaling_bonus: i32,
}

impl Parameters {
  /// How many parameters there are
  pub const COUNT: usize = 290;

  /// Is parameter index valid
  pub fn valid_index(index: usize) -> bool {
    let group = index / 18;
    let index = index % 18;
    match group {
      0 | 1 | 12 | 13 => true,
      2..=11 => index < OBSTACLE as usize - 1,
      14 | 15 => index != 0,
      16 => index < 2,
      _ => panic!("Invalid parameter index"),
    }
  }

  /// How many iterations to run
  pub fn iteration_count(index: usize) -> i32 {
    let group = index / 18;
    match group {
      0 | 1 => 4,
      2..=15 => 2,
      16 => 1,
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
      4 => self.mg_edge[index][2] += bonus,
      5 => self.mg_edge[index][3] += bonus,
      6 => self.mg_edge[index][4] += bonus,
      7 => self.eg_edge[index][0] += bonus,
      8 => self.eg_edge[index][1] += bonus,
      9 => self.eg_edge[index][2] += bonus,
      10 => self.eg_edge[index][3] += bonus,
      11 => self.eg_edge[index][4] += bonus,
      12 => self.mg_friendly_pawn_penalty[index] += bonus,
      13 => self.eg_friendly_pawn_penalty[index] += bonus,
      14 => self.mg_enemy_pawn_penalty[index] += bonus,
      15 => self.eg_enemy_pawn_penalty[index] += bonus,
      16 => match index {
        0 => self.pawn_scale_factor += bonus,
        1 => self.pawn_scaling_bonus += bonus,
        _ => panic!("Invalid parameter index"),
      },
      _ => panic!("Invalid parameter index"),
    }
  }
}
