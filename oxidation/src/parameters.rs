use liberty_chess::parsing::to_name;
use liberty_chess::{CENTAUR, CHAMPION, ELEPHANT, KING, MANN, OBSTACLE, WALL};
use std::ops::{Add, AddAssign, Div, Mul};

const PIECE_VALUES: [(i32, i32); 18] = [
  (61, 141),    // Pawn
  (278, 367),   // Knight
  (323, 299),   // Bishop
  (443, 512),   // Rook
  (1047, 964),  // Queen
  (-505, 812),  // King
  (853, 1081),  // Archbishop
  (977, 1147),  // Chancellor
  (229, 252),   // Camel
  (195, 199),   // Zebra
  (208, 339),   // Mann
  (533, 369),   // Nightrider
  (549, 1076),  // Champion
  (544, 1197),  // Centaur
  (1595, 1635), // Amazon
  (753, 552),   // Elephant
  (18, 44),     // Obstacle
  (85, 105),    // Wall
];

const MG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [-38, 4, 5, 23, -31, -17, -6, -6, -7],    // Pawn
  [42, 53, 32, 27, 28, 17, 13, 0, -3],      // Knight
  [29, 33, 49, 6, 5, -4, 0, -7, 4],         // Bishop
  [28, 27, 1, 0, 0, -4, 3, -10, -6],        // Rook
  [23, -4, 0, -7, 23, 4, -3, 0, 1],         // Queen
  [-32, -49, -17, -1, 34, 19, 48, 10, 32],  // King
  [57, 9, 15, 25, 20, 3, -1, 2, -3],        // Archbishop
  [-2, 21, 0, 17, 84, -19, 3, -12, -1],     // Chancellor
  [-17, 73, 10, 7, 11, 10, -9, -5, -15],    // Camel
  [37, 0, -2, 29, 79, 2, 17, 57, 3],        // Zebra
  [7, 34, 10, 73, 14, 0, 0, 17, 16],        // Mann
  [46, 8, -10, -22, -76, -6, -74, -23, -5], // Nightrider
  [65, 38, 14, 9, 0, 1, 9, 18, 13],         // Champion
  [40, 26, 42, 6, 38, 29, 11, 5, 0],        // Centaur
  [88, 37, 7, 1, 33, 2, -26, 3, -22],       // Amazon
  [112, 128, 112, 75, 84, 43, 40, 38, 49],  // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],              // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],              // Wall
];

const EG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [141, 8, 20, 12, 1, 8, 3, 7, 6],             // Pawn
  [51, 9, 21, 23, 15, 11, 17, 5, 11],          // Knight
  [2, -11, -12, -7, 12, -6, 9, -6, -13],       // Bishop
  [76, 19, 18, 4, 26, 20, -3, 22, -4],         // Rook
  [58, 25, 13, 32, 6, 25, 6, 6, -19],          // Queen
  [132, 83, 56, 41, 20, 22, 3, 14, 0],         // King
  [298, 204, 127, 80, 134, 39, 61, 54, 5],     // Archbishop
  [73, 42, 81, -6, -8, 45, 25, 17, -10],       // Chancellor
  [34, 13, 26, 15, -1, 10, 8, 26, 21],         // Camel
  [-11, 6, 1, -17, -23, 20, -24, -31, -11],    // Zebra
  [180, 5, 73, 0, 16, 6, 47, 8, 0],            // Mann
  [5, 24, -5, -17, 1, 7, 49, 46, 5],           // Nightrider
  [14, 93, 156, 90, 111, 65, 69, 0, 0],        // Champion
  [381, 203, 189, 131, 166, 126, 89, 74, 41],  // Centaur
  [153, 139, 5, -39, -14, 82, -48, 78, -62],   // Amazon
  [207, 197, 154, 151, 167, 112, 120, 79, 31], // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                 // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                 // Wall
];

const MG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,  // Pawn
  10, // Knight
  2,  // Bishop
  0,  // Rook
  5,  // Queen
  25, // King
  7,  // Archbishop
  8,  // Chancellor
  0,  // Camel
  0,  // Zebra
  8,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  5,  // Elephant
  0,  // Obstacle
  18, // Wall
];

const EG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  31, // Pawn
  12, // Knight
  19, // Bishop
  0,  // Rook
  0,  // Queen
  0,  // King
  25, // Archbishop
  19, // Chancellor
  5,  // Camel
  0,  // Zebra
  33, // Mann
  0,  // Nightrider
  4,  // Champion
  0,  // Centaur
  0,  // Amazon
  20, // Elephant
  0,  // Obstacle
  0,  // Wall
];

const MG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  31,  // Knight
  13,  // Bishop
  -29, // Rook
  4,   // Queen
  61,  // King
  17,  // Archbishop
  11,  // Chancellor
  -9,  // Camel
  -32, // Zebra
  32,  // Mann
  2,   // Nightrider
  3,   // Champion
  21,  // Centaur
  39,  // Amazon
  61,  // Elephant
  -15, // Obstacle
  0,   // Wall
];

const EG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  19,  // Knight
  80,  // Bishop
  86,  // Rook
  51,  // Queen
  43,  // King
  82,  // Archbishop
  63,  // Chancellor
  26,  // Camel
  37,  // Zebra
  92,  // Mann
  31,  // Nightrider
  81,  // Champion
  52,  // Centaur
  141, // Amazon
  0,   // Elephant
  60,  // Obstacle
  0,   // Wall
];

const MG_MOBILITY_BONUS: [i32; 18] = [
  0, // Pawn
  0, // Knight
  4, // Bishop
  6, // Rook
  3, // Queen
  0, // King
  1, // Archbishop
  2, // Chancellor
  0, // Camel
  0, // Zebra
  0, // Mann
  6, // Nightrider
  0, // Champion
  0, // Centaur
  0, // Amazon
  0, // Elephant
  0, // Obstacle
  0, // Wall
];

const EG_MOBILITY_BONUS: [i32; 18] = [
  0,  // Pawn
  0,  // Knight
  6,  // Bishop
  10, // Rook
  10, // Queen
  0,  // King
  0,  // Archbishop
  13, // Chancellor
  0,  // Camel
  0,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  15, // Amazon
  0,  // Elephant
  0,  // Obstacle
  0,  // Wall
];

const MG_PAWN_ATTACKED_PENALTY: [i32; 18] = [
  -4,  // Pawn
  50,  // Knight
  53,  // Bishop
  61,  // Rook
  38,  // Queen
  0,   // King
  40,  // Archbishop
  49,  // Chancellor
  56,  // Camel
  22,  // Zebra
  68,  // Mann
  82,  // Nightrider
  34,  // Champion
  20,  // Centaur
  65,  // Amazon
  4,   // Elephant
  -14, // Obstacle
  22,  // Wall
];

const EG_PAWN_ATTACKED_PENALTY: [i32; 18] = [
  -30, // Pawn
  24,  // Knight
  50,  // Bishop
  -3,  // Rook
  34,  // Queen
  0,   // King
  -20, // Archbishop
  -95, // Chancellor
  48,  // Camel
  77,  // Zebra
  48,  // Mann
  -93, // Nightrider
  60,  // Champion
  0,   // Centaur
  84,  // Amazon
  106, // Elephant
  0,   // Obstacle
  54,  // Wall
];

const MG_PAWN_DEFENDED_BONUS: [i32; 18] = [
  7,   // Pawn
  4,   // Knight
  11,  // Bishop
  -20, // Rook
  -4,  // Queen
  -32, // King
  2,   // Archbishop
  -2,  // Chancellor
  12,  // Camel
  35,  // Zebra
  -31, // Mann
  20,  // Nightrider
  -3,  // Champion
  9,   // Centaur
  -5,  // Amazon
  4,   // Elephant
  26,  // Obstacle
  -13, // Wall
];

const EG_PAWN_DEFENDED_BONUS: [i32; 18] = [
  4,   // Pawn
  -5,  // Knight
  5,   // Bishop
  57,  // Rook
  34,  // Queen
  22,  // King
  -16, // Archbishop
  -6,  // Chancellor
  -5,  // Camel
  -27, // Zebra
  33,  // Mann
  70,  // Nightrider
  -23, // Champion
  27,  // Centaur
  165, // Amazon
  -4,  // Elephant
  9,   // Obstacle
  7,   // Wall
];

// advanced pawns get a bonus of numerator/(factor * squares_to_promotion + bonus) times the promotion value
pub(crate) const PAWN_SCALING_NUMERATOR: i32 = 20;
const PAWN_SCALING_FACTOR: i32 = 180;
const PAWN_SCALING_BONUS: i32 = -69;

pub(crate) const TEMPO_BONUS: i32 = 10;

/// Maximum distance from the edge to apply penalty
pub(crate) const EDGE_DISTANCE: usize = 3;
pub(crate) const EDGE_PARAMETER_COUNT: usize = EDGE_DISTANCE * (EDGE_DISTANCE + 3) / 2;
pub(crate) const INDEXING: [usize; (EDGE_DISTANCE + 1) * (EDGE_DISTANCE + 1)] =
  [0, 1, 2, 3, 1, 4, 5, 6, 2, 5, 7, 8, 3, 6, 8, 9];

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

#[cfg(not(feature = "feature_extraction"))]
pub(crate) const fn pack(mg: i32, eg: i32) -> i64 {
  ((eg as i64) << 32) + mg as i64
}

#[cfg(not(feature = "feature_extraction"))]
pub(crate) fn unpack_mg(value: i64) -> i32 {
  value as i32
}

#[cfg(not(feature = "feature_extraction"))]
pub(crate) fn unpack_eg(value: i64) -> i32 {
  ((value + 0x80000000) >> 32) as i32
}

#[cfg(not(feature = "feature_extraction"))]
pub(crate) struct PackedParameters {
  pub(crate) pieces: [i64; 18],
  pub(crate) edge_avoidance: [[i64; EDGE_PARAMETER_COUNT]; 18],
  pub(crate) friendly_pawn_penalty: [i64; 18],
  pub(crate) enemy_pawn_penalty: [i64; 18],
  pub(crate) mobility_bonus: [i64; 18],
  pub(crate) pawn_attacked_penalty: [i64; 18],
  pub(crate) pawn_defended_bonus: [i64; 18],
  #[cfg(not(feature = "pesto"))]
  pub(crate) pawn_scale_factor: i32,
  #[cfg(not(feature = "pesto"))]
  pub(crate) pawn_scaling_bonus: i32,
}

#[cfg(not(feature = "feature_extraction"))]
impl From<Parameters<i32>> for PackedParameters {
  fn from(value: Parameters<i32>) -> Self {
    let mut edge_avoidance = [[0; EDGE_PARAMETER_COUNT]; 18];
    let mut friendly_pawn_penalty = [0; 18];
    let mut enemy_pawn_penalty = [0; 18];
    let mut mobility_bonus = [0; 18];
    let mut pawn_attacked_penalty = [0; 18];
    let mut pawn_defended_bonus = [0; 18];
    for i in 0..18 {
      for j in 0..EDGE_PARAMETER_COUNT {
        edge_avoidance[i][j] = pack(value.mg_edge[i][j], value.eg_edge[i][j]);
      }
      friendly_pawn_penalty[i] = pack(
        value.mg_friendly_pawn_penalty[i],
        value.eg_friendly_pawn_penalty[i],
      );
      enemy_pawn_penalty[i] = pack(
        value.mg_enemy_pawn_penalty[i],
        value.eg_enemy_pawn_penalty[i],
      );
      mobility_bonus[i] = pack(value.mg_mobility_bonus[i], value.eg_mobility_bonus[i]);
      pawn_attacked_penalty[i] = pack(
        value.mg_pawn_attacked_penalty[i],
        value.eg_pawn_attacked_penalty[i],
      );
      pawn_defended_bonus[i] = pack(
        value.mg_pawn_defended_bonus[i],
        value.eg_pawn_defended_bonus[i],
      );
    }
    Self {
      pieces: value.pieces.map(|(mg, eg)| pack(mg, eg)),
      edge_avoidance,
      friendly_pawn_penalty,
      enemy_pawn_penalty,
      mobility_bonus,
      pawn_attacked_penalty,
      pawn_defended_bonus,
      #[cfg(not(feature = "pesto"))]
      pawn_scale_factor: value.pawn_scale_factor,
      #[cfg(not(feature = "pesto"))]
      pawn_scaling_bonus: value.pawn_scaling_bonus,
    }
  }
}

/// The default set of parameters
pub const DEFAULT_PARAMETERS: Parameters<i32> = Parameters {
  pieces: PIECE_VALUES,
  mg_edge: MG_EDGE_AVOIDANCE,
  eg_edge: EG_EDGE_AVOIDANCE,
  mg_friendly_pawn_penalty: MG_FRIENDLY_PAWN_PENALTY,
  eg_friendly_pawn_penalty: EG_FRIENDLY_PAWN_PENALTY,
  mg_enemy_pawn_penalty: MG_ENEMY_PAWN_PENALTY,
  eg_enemy_pawn_penalty: EG_ENEMY_PAWN_PENALTY,
  mg_mobility_bonus: MG_MOBILITY_BONUS,
  eg_mobility_bonus: EG_MOBILITY_BONUS,
  mg_pawn_attacked_penalty: MG_PAWN_ATTACKED_PENALTY,
  eg_pawn_attacked_penalty: EG_PAWN_ATTACKED_PENALTY,
  mg_pawn_defended_bonus: MG_PAWN_DEFENDED_BONUS,
  eg_pawn_defended_bonus: EG_PAWN_DEFENDED_BONUS,
  pawn_scale_factor: PAWN_SCALING_FACTOR,
  pawn_scaling_bonus: PAWN_SCALING_BONUS,
};

/// Parameters for evaluation
#[derive(Copy, Clone, Debug, Default)]
pub struct Parameters<T> {
  /// Piece values
  pub pieces: [(T, T); 18],
  /// Middlegame penalties for being on the board edge
  pub mg_edge: [[T; EDGE_PARAMETER_COUNT]; 18],
  /// Endgame penalties for being on the board edge
  pub eg_edge: [[T; EDGE_PARAMETER_COUNT]; 18],
  /// Middlegame penalty for a pawn being blocked by a friendly piece
  pub mg_friendly_pawn_penalty: [T; 18],
  /// Endgame penalty for a pawn being blocked by a friendly piece
  pub eg_friendly_pawn_penalty: [T; 18],
  /// Middlegame penalty for a pawn being blocked by an enemy piece
  pub mg_enemy_pawn_penalty: [T; 18],
  /// Endgame penalty for a pawn being blocked by an enemy piece
  pub eg_enemy_pawn_penalty: [T; 18],
  /// Middlegame mobility bonus
  pub mg_mobility_bonus: [T; 18],
  /// Endgame mobility bonus
  pub eg_mobility_bonus: [T; 18],
  /// Middlegame penalty for being attacked by a pawn
  pub mg_pawn_attacked_penalty: [T; 18],
  /// Endgame penalty for being attacked by a pawn
  pub eg_pawn_attacked_penalty: [T; 18],
  /// Middlegame bonus for being defended by a pawn
  pub mg_pawn_defended_bonus: [T; 18],
  /// Endgame bonus for being defended by a pawn
  pub eg_pawn_defended_bonus: [T; 18],
  /// Scaling factor for the advanced pawn bonus
  pub pawn_scale_factor: T,
  /// Scaling factor for the advanced pawn bonus
  pub pawn_scaling_bonus: T,
}

impl<T: Copy + AddAssign> AddAssign for Parameters<T> {
  fn add_assign(&mut self, rhs: Self) {
    for i in 0..18 {
      self.pieces[i].0 += rhs.pieces[i].0;
      self.pieces[i].1 += rhs.pieces[i].1;
      self.mg_friendly_pawn_penalty[i] += rhs.mg_friendly_pawn_penalty[i];
      self.eg_friendly_pawn_penalty[i] += rhs.eg_friendly_pawn_penalty[i];
      self.mg_enemy_pawn_penalty[i] += rhs.mg_enemy_pawn_penalty[i];
      self.eg_enemy_pawn_penalty[i] += rhs.eg_enemy_pawn_penalty[i];
      self.mg_mobility_bonus[i] += rhs.mg_mobility_bonus[i];
      self.eg_mobility_bonus[i] += rhs.eg_mobility_bonus[i];
      self.mg_pawn_attacked_penalty[i] += rhs.mg_pawn_attacked_penalty[i];
      self.eg_pawn_attacked_penalty[i] += rhs.eg_pawn_attacked_penalty[i];
      self.mg_pawn_defended_bonus[i] += rhs.mg_pawn_defended_bonus[i];
      self.eg_pawn_defended_bonus[i] += rhs.eg_pawn_defended_bonus[i];
    }
    for i in 0..18 {
      for j in 0..EDGE_PARAMETER_COUNT {
        self.mg_edge[i][j] += rhs.mg_edge[i][j];
        self.eg_edge[i][j] += rhs.eg_edge[i][j];
      }
    }
    self.pawn_scale_factor += rhs.pawn_scale_factor;
    self.pawn_scaling_bonus += rhs.pawn_scaling_bonus;
  }
}

impl<T: Copy + AddAssign> Add for Parameters<T> {
  type Output = Self;

  fn add(mut self, rhs: Self) -> Self {
    self += rhs;
    self
  }
}

impl Div<f64> for Parameters<f64> {
  type Output = Self;

  fn div(self, rhs: f64) -> Self {
    Self {
      pieces: self.pieces.map(|(x, y)| (x / rhs, y / rhs)),
      mg_edge: self.mg_edge.map(|x| x.map(|y| y / rhs)),
      eg_edge: self.eg_edge.map(|x| x.map(|y| y / rhs)),
      mg_friendly_pawn_penalty: self.mg_friendly_pawn_penalty.map(|x| x / rhs),
      eg_friendly_pawn_penalty: self.eg_friendly_pawn_penalty.map(|x| x / rhs),
      mg_enemy_pawn_penalty: self.mg_enemy_pawn_penalty.map(|x| x / rhs),
      eg_enemy_pawn_penalty: self.eg_enemy_pawn_penalty.map(|x| x / rhs),
      mg_mobility_bonus: self.mg_mobility_bonus.map(|x| x / rhs),
      eg_mobility_bonus: self.eg_mobility_bonus.map(|x| x / rhs),
      mg_pawn_attacked_penalty: self.mg_pawn_attacked_penalty.map(|x| x / rhs),
      eg_pawn_attacked_penalty: self.eg_pawn_attacked_penalty.map(|x| x / rhs),
      mg_pawn_defended_bonus: self.mg_pawn_defended_bonus.map(|x| x / rhs),
      eg_pawn_defended_bonus: self.eg_pawn_defended_bonus.map(|x| x / rhs),
      pawn_scale_factor: self.pawn_scale_factor / rhs,
      pawn_scaling_bonus: self.pawn_scaling_bonus / rhs,
    }
  }
}

impl Div<Self> for Parameters<f64> {
  type Output = Self;

  fn div(mut self, rhs: Self) -> Self {
    for i in 0..18 {
      self.pieces[i].0 /= rhs.pieces[i].0;
      self.pieces[i].1 /= rhs.pieces[i].1;
      self.mg_friendly_pawn_penalty[i] /= rhs.mg_friendly_pawn_penalty[i];
      self.eg_friendly_pawn_penalty[i] /= rhs.eg_friendly_pawn_penalty[i];
      self.mg_enemy_pawn_penalty[i] /= rhs.mg_enemy_pawn_penalty[i];
      self.eg_enemy_pawn_penalty[i] /= rhs.eg_enemy_pawn_penalty[i];
      self.mg_mobility_bonus[i] /= rhs.mg_mobility_bonus[i];
      self.eg_mobility_bonus[i] /= rhs.eg_mobility_bonus[i];
      self.mg_pawn_attacked_penalty[i] /= rhs.mg_pawn_attacked_penalty[i];
      self.eg_pawn_attacked_penalty[i] /= rhs.eg_pawn_attacked_penalty[i];
      self.mg_pawn_defended_bonus[i] /= rhs.mg_pawn_defended_bonus[i];
      self.eg_pawn_defended_bonus[i] /= rhs.eg_pawn_defended_bonus[i];
      for j in 0..EDGE_PARAMETER_COUNT {
        self.mg_edge[i][j] /= rhs.mg_edge[i][j];
        self.eg_edge[i][j] /= rhs.eg_edge[i][j];
      }
    }
    self.pawn_scale_factor /= rhs.pawn_scale_factor;
    self.pawn_scaling_bonus /= rhs.pawn_scaling_bonus;
    self
  }
}

impl Mul<f64> for Parameters<f64> {
  type Output = Self;

  fn mul(self, rhs: f64) -> Self {
    Self {
      pieces: self.pieces.map(|(x, y)| (x * rhs, y * rhs)),
      mg_edge: self.mg_edge.map(|x| x.map(|y| y * rhs)),
      eg_edge: self.eg_edge.map(|x| x.map(|y| y * rhs)),
      mg_friendly_pawn_penalty: self.mg_friendly_pawn_penalty.map(|x| x * rhs),
      eg_friendly_pawn_penalty: self.eg_friendly_pawn_penalty.map(|x| x * rhs),
      mg_enemy_pawn_penalty: self.mg_enemy_pawn_penalty.map(|x| x * rhs),
      eg_enemy_pawn_penalty: self.eg_enemy_pawn_penalty.map(|x| x * rhs),
      mg_mobility_bonus: self.mg_mobility_bonus.map(|x| x * rhs),
      eg_mobility_bonus: self.eg_mobility_bonus.map(|x| x * rhs),
      mg_pawn_attacked_penalty: self.mg_pawn_attacked_penalty.map(|x| x * rhs),
      eg_pawn_attacked_penalty: self.eg_pawn_attacked_penalty.map(|x| x * rhs),
      mg_pawn_defended_bonus: self.mg_pawn_defended_bonus.map(|x| x * rhs),
      eg_pawn_defended_bonus: self.eg_pawn_defended_bonus.map(|x| x * rhs),
      pawn_scale_factor: self.pawn_scale_factor * rhs,
      pawn_scaling_bonus: self.pawn_scaling_bonus * rhs,
    }
  }
}

impl Parameters<f64> {
  /// Get the absolute value of the parameters
  #[must_use]
  pub fn abs(&self) -> Self {
    Self {
      pieces: self.pieces.map(|(x, y)| (x.abs(), y.abs())),
      mg_edge: self.mg_edge.map(|x| x.map(f64::abs)),
      eg_edge: self.eg_edge.map(|x| x.map(f64::abs)),
      mg_friendly_pawn_penalty: self.mg_friendly_pawn_penalty.map(f64::abs),
      eg_friendly_pawn_penalty: self.eg_friendly_pawn_penalty.map(f64::abs),
      mg_enemy_pawn_penalty: self.mg_enemy_pawn_penalty.map(f64::abs),
      eg_enemy_pawn_penalty: self.eg_enemy_pawn_penalty.map(f64::abs),
      mg_mobility_bonus: self.mg_mobility_bonus.map(f64::abs),
      eg_mobility_bonus: self.eg_mobility_bonus.map(f64::abs),
      mg_pawn_attacked_penalty: self.mg_pawn_attacked_penalty.map(f64::abs),
      eg_pawn_attacked_penalty: self.eg_pawn_attacked_penalty.map(f64::abs),
      mg_pawn_defended_bonus: self.mg_pawn_defended_bonus.map(f64::abs),
      eg_pawn_defended_bonus: self.eg_pawn_defended_bonus.map(f64::abs),
      pawn_scale_factor: self.pawn_scale_factor.abs(),
      pawn_scaling_bonus: self.pawn_scaling_bonus.abs(),
    }
  }

  fn remove_nan(x: f64) -> f64 {
    if x.is_finite() {
      x
    } else {
      0.0
    }
  }

  /// Clear infinite/Nan values
  #[must_use]
  pub fn sanitize(&self) -> Self {
    Self {
      pieces: self
        .pieces
        .map(|(x, y)| (Self::remove_nan(x), Self::remove_nan(y))),
      mg_edge: self.mg_edge.map(|a| a.map(Self::remove_nan)),
      eg_edge: self.eg_edge.map(|a| a.map(Self::remove_nan)),
      mg_friendly_pawn_penalty: self.mg_friendly_pawn_penalty.map(Self::remove_nan),
      eg_friendly_pawn_penalty: self.eg_friendly_pawn_penalty.map(Self::remove_nan),
      mg_enemy_pawn_penalty: self.mg_enemy_pawn_penalty.map(Self::remove_nan),
      eg_enemy_pawn_penalty: self.eg_enemy_pawn_penalty.map(Self::remove_nan),
      mg_mobility_bonus: self.mg_mobility_bonus.map(Self::remove_nan),
      eg_mobility_bonus: self.eg_mobility_bonus.map(Self::remove_nan),
      mg_pawn_attacked_penalty: self.mg_pawn_attacked_penalty.map(Self::remove_nan),
      eg_pawn_attacked_penalty: self.eg_pawn_attacked_penalty.map(Self::remove_nan),
      mg_pawn_defended_bonus: self.mg_pawn_defended_bonus.map(Self::remove_nan),
      eg_pawn_defended_bonus: self.eg_pawn_defended_bonus.map(Self::remove_nan),
      pawn_scale_factor: Self::remove_nan(self.pawn_scale_factor),
      pawn_scaling_bonus: Self::remove_nan(self.pawn_scaling_bonus),
    }
  }

  /// Some parameter values are known to be preferred by the tuner but lose elo.
  /// Avoiding these values should allow the other values to better adjust to the constraints
  pub fn enforce_invariants(&mut self) {
    let (mg_pawn, eg_pawn) = self.pieces[0];
    for i in 0..18 {
      self.mg_mobility_bonus[i] = self.mg_mobility_bonus[i].max(0.0);
      self.eg_mobility_bonus[i] = self.eg_mobility_bonus[i].max(0.0);
      self.mg_friendly_pawn_penalty[i] = self.mg_friendly_pawn_penalty[i].clamp(0.0, mg_pawn);
      self.eg_friendly_pawn_penalty[i] = self.eg_friendly_pawn_penalty[i].clamp(0.0, eg_pawn);
      self.mg_enemy_pawn_penalty[i] = self.mg_enemy_pawn_penalty[i].min(mg_pawn);
      self.eg_enemy_pawn_penalty[i] = self.eg_enemy_pawn_penalty[i].clamp(0.0, eg_pawn);
      let (mg_piece, eg_piece) = self.pieces[i];
      match (i + 1) as i8 {
        MANN | CHAMPION | CENTAUR | ELEPHANT => {
          for j in 0..EDGE_PARAMETER_COUNT {
            self.mg_edge[i][j] = self.mg_edge[i][j].clamp(0.0, mg_piece);
            self.eg_edge[i][j] = self.eg_edge[i][j].clamp(0.0, eg_piece);
          }
        }
        KING | OBSTACLE | WALL => (),
        _ => {
          for j in 0..EDGE_PARAMETER_COUNT {
            self.mg_edge[i][j] = self.mg_edge[i][j].min(mg_piece);
            self.eg_edge[i][j] = self.eg_edge[i][j].min(eg_piece);
          }
        }
      }
    }
  }
}

impl From<Parameters<i32>> for Parameters<f64> {
  fn from(value: Parameters<i32>) -> Self {
    Self {
      pieces: value.pieces.map(|(a, b)| (f64::from(a), f64::from(b))),
      mg_edge: value.mg_edge.map(|a| a.map(f64::from)),
      eg_edge: value.eg_edge.map(|a| a.map(f64::from)),
      mg_friendly_pawn_penalty: value.mg_friendly_pawn_penalty.map(f64::from),
      eg_friendly_pawn_penalty: value.eg_friendly_pawn_penalty.map(f64::from),
      mg_enemy_pawn_penalty: value.mg_enemy_pawn_penalty.map(f64::from),
      eg_enemy_pawn_penalty: value.eg_enemy_pawn_penalty.map(f64::from),
      mg_mobility_bonus: value.mg_mobility_bonus.map(f64::from),
      eg_mobility_bonus: value.eg_mobility_bonus.map(f64::from),
      mg_pawn_attacked_penalty: value.mg_pawn_attacked_penalty.map(f64::from),
      eg_pawn_attacked_penalty: value.eg_pawn_attacked_penalty.map(f64::from),
      mg_pawn_defended_bonus: value.mg_pawn_defended_bonus.map(f64::from),
      eg_pawn_defended_bonus: value.eg_pawn_defended_bonus.map(f64::from),
      pawn_scale_factor: f64::from(value.pawn_scale_factor),
      pawn_scaling_bonus: f64::from(value.pawn_scaling_bonus),
    }
  }
}

// Converts the parameters as a float back to ints, suitable for being pasted into this file
impl ToString for Parameters<f64> {
  fn to_string(&self) -> String {
    let mut result = "const PIECE_VALUES: [(i32, i32); 18] = [".to_string();
    for i in 0..18 {
      let pieces = self.pieces[i];
      result += &format!(
        "\n  ({}, {}), // {}",
        pieces.0 as i32,
        pieces.1 as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {:?}, // {}",
        self.mg_edge[i].map(|x| x as i32),
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {:?}, // {}",
        self.eg_edge[i].map(|x| x as i32),
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_friendly_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_friendly_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_ENEMY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_enemy_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_ENEMY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_enemy_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_MOBILITY_BONUS: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_mobility_bonus[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_MOBILITY_BONUS: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_mobility_bonus[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_PAWN_ATTACKED_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_pawn_attacked_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_PAWN_ATTACKED_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_pawn_attacked_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MG_PAWN_DEFENDED_BONUS: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_pawn_defended_bonus[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst EG_PAWN_DEFENDED_BONUS: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_pawn_defended_bonus[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result + "\n];"
  }
}
