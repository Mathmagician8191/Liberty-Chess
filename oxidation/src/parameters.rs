use liberty_chess::OBSTACLE;

pub(crate) const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  50,    // pawn
  315,   // knight
  393,   // bishop
  564,   // rook
  1239,  // queen
  -1464, // king
  935,   // archbishop
  1100,  // chancellor
  145,   // camel
  126,   // zebra
  169,   // mann
  657,   // nighrider
  510,   // champion
  594,   // centaur
  1841,  // amazon
  657,   // elephant
  21,    // obstacle
  117,   // wall
];

const ENDGAME_PIECE_VALUES: [i32; 18] = [
  182,  // pawn
  361,  // knight
  380,  // bishop
  682,  // rook
  1286, // queen
  1286, // king
  1058, // archbishop
  1400, // chancellor
  291,  // camel
  237,  // zebra
  264,  // mann
  375,  // nightrider
  1142, // champion
  1175, // centaur
  1579, // amazon
  651,  // elephant
  109,  // obstacle
  110,  // wall
];

const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [18, -10],   // pawn
  [42, 22],    // knight
  [59, 5],     // bishop
  [39, 29],    // rook
  [31, 5],     // queen
  [-108, -28], // king
  [35, 17],    // archbishop
  [13, 5],     // chancellor
  [-11, -4],   // camel
  [2, 19],     // zebra
  [15, 0],     // mann
  [16, -30],   // nightrider
  [11, 1],     // champion
  [34, 21],    // centaur
  [13, 15],    // amazon
  [63, 16],    // elephant
  [0, 0],
  [0, 0],
];

const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [36, 1],   // pawn
  [6, 0],    // knight
  [22, 16],  // bishop
  [44, -4],  // rook
  [37, 28],  // queen
  [74, 20],  // king
  [69, 21],  // archbishop
  [29, 1],   // chancellor
  [17, 2],   // camel
  [-2, 0],   // zebra
  [30, 10],  // mann
  [80, 35],  // nightrider
  [84, 0],   // champion
  [123, 40], // centaur
  [68, 62],  // amazon
  [86, 54],  // elephant
  [0, 0],
  [0, 0],
];

const MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  -6,  // pawn
  13,  // knight
  15,  // bishop
  0,   // rook
  -3,  // queen
  49,  // king
  8,   // archbishop
  4,   // chancellor
  14,  // camel
  0,   // zebra
  0,   // mann
  1,   // nightrider
  -26, // champion
  -6,  // centaur
  -26, // amazon
  -12, // elephant
  -1,  // obstacle
  1,   // wall
];

const ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  52,  // pawn
  22,  // knight
  0,   // bishop
  0,   // rook
  28,  // queen
  -10, // king
  22,  // archbishop
  76,  // chancellor
  0,   // camel
  35,  // zebra
  0,   // mann
  0,   // nightrider
  39,  // champion
  48,  // centaur
  77,  // amazon
  60,  // elephant
  11,  // obstacle
  13,  // wall
];

const MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  22,  // knight
  16,  // bishop
  -37, // rook
  14,  // queen
  110, // king
  16,  // archbishop
  6,   // chancellor
  0,   // camel
  0,   // zebra
  1,   // mann
  36,  // nighrider
  37,  // champion
  30,  // centaur
  10,  // amazon
  46,  // elephant
  0,   // obstacle
  31,  // wall
];

const ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // pawn
  40,  // knight
  129, // bishop
  72,  // rook
  36,  // queen
  55,  // king
  172, // archbishop
  7,   // chancellor
  37,  // camel
  27,  // zebra
  194, // mann
  67,  // nightrider
  130, // champion
  98,  // centaur
  162, // amazon
  0,   // elephant
  10,  // obstacle
  0,   // wall
];

// advanced pawns get a bonus of 1/(factor * squares_to_promotion + bonus) times the promotion value
const PAWN_SCALING_FACTOR: i32 = 8;
const PAWN_SCALING_BONUS: i32 = -1;

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
