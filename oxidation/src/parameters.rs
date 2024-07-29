use liberty_chess::parsing::to_name;
use liberty_chess::{CENTAUR, CHAMPION, ELEPHANT, KING, MANN, OBSTACLE, WALL};
use std::ops::{Add, AddAssign, Div, Mul};

const PIECE_VALUES: [(i32, i32); 18] = [
  (71, 132),    // Pawn
  (286, 350),   // Knight
  (328, 289),   // Bishop
  (430, 502),   // Rook
  (1044, 966),  // Queen
  (-765, 815),  // King
  (819, 1033),  // Archbishop
  (933, 1115),  // Chancellor
  (249, 241),   // Camel
  (233, 175),   // Zebra
  (212, 344),   // Mann
  (505, 358),   // Nightrider
  (570, 1043),  // Champion
  (555, 1198),  // Centaur
  (1436, 1671), // Amazon
  (798, 563),   // Elephant
  (1, 82),      // Obstacle
  (118, 94),    // Wall
];

const MG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [-89, 2, 3, 22, -29, -14, -2, -2, 0],     // Pawn
  [29, 50, 35, 32, 28, 12, 14, 6, 3],       // Knight
  [58, 39, 42, 6, 7, -3, -2, -2, 2],        // Bishop
  [22, 25, 4, 0, -2, -5, -3, -10, -1],      // Rook
  [18, 13, 4, 0, 22, -2, -2, -1, 3],        // Queen
  [-11, -49, -17, -1, 48, 29, 50, 13, 37],  // King
  [58, 1, 14, 22, 11, 12, 3, -12, 1],       // Archbishop
  [10, 15, 1, 12, 72, -17, -3, -6, 6],      // Chancellor
  [-21, 75, -4, 6, 17, 11, -7, 19, 0],      // Camel
  [0, 32, 12, 23, 82, 10, 19, 64, 11],      // Zebra
  [74, 68, 21, 43, 0, 25, 0, 18, 24],       // Mann
  [-9, 1, -13, -22, 59, -23, -40, -12, -1], // Nightrider
  [49, 61, 36, 0, 4, 22, 21, 10, 19],       // Champion
  [40, 22, 33, 26, 36, 26, 14, 5, 2],       // Centaur
  [53, 32, 9, -3, 13, 8, -5, -17, -5],      // Amazon
  [207, 171, 138, 63, 120, 49, 12, 11, 22], // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],              // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],              // Wall
];

const EG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [132, 7, 19, 16, 4, 9, 5, 5, 2],              // Pawn
  [56, 10, 18, 24, 17, 18, 17, 7, 3],           // Knight
  [-20, -8, -5, -1, 5, 1, 15, -7, -2],          // Bishop
  [88, 28, 17, 9, 30, 18, -1, 12, -9],          // Rook
  [61, 10, 12, 16, 5, 22, 6, -3, -19],          // Queen
  [127, 91, 60, 44, 21, 25, 8, 20, 3],          // King
  [204, 153, 108, 57, 86, 0, 17, 19, -11],      // Archbishop
  [32, -6, 58, -4, -45, 13, 0, 6, -30],         // Chancellor
  [53, 5, 31, 26, -1, 9, 8, 15, 3],             // Camel
  [37, 19, -2, -13, -3, 11, -20, -38, -6],      // Zebra
  [32, 85, 104, 4, 35, 15, 23, 15, 0],          // Mann
  [31, 38, 29, 15, 5, 45, 48, 37, 15],          // Nightrider
  [124, 138, 124, 161, 182, 38, 2, 17, 0],      // Champion
  [396, 232, 182, 141, 115, 102, 56, 57, 16],   // Centaur
  [72, -10, 80, 37, 21, 23, -16, 69, -67],      // Amazon
  [192, 173, 140, 176, 145, 119, 159, 106, 62], // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                  // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                  // Wall
];

const MG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,  // Pawn
  11, // Knight
  8,  // Bishop
  0,  // Rook
  5,  // Queen
  31, // King
  7,  // Archbishop
  0,  // Chancellor
  0,  // Camel
  0,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  20, // Elephant
  0,  // Obstacle
  15, // Wall
];

const EG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  37, // Pawn
  8,  // Knight
  10, // Bishop
  0,  // Rook
  0,  // Queen
  0,  // King
  0,  // Archbishop
  4,  // Chancellor
  11, // Camel
  1,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  4,  // Elephant
  18, // Obstacle
  13, // Wall
];

const MG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  25,  // Knight
  8,   // Bishop
  -21, // Rook
  3,   // Queen
  71,  // King
  21,  // Archbishop
  8,   // Chancellor
  -18, // Camel
  -53, // Zebra
  23,  // Mann
  31,  // Nightrider
  10,  // Champion
  18,  // Centaur
  16,  // Amazon
  71,  // Elephant
  0,   // Obstacle
  0,   // Wall
];

const EG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  23,  // Knight
  77,  // Bishop
  87,  // Rook
  61,  // Queen
  39,  // King
  84,  // Archbishop
  89,  // Chancellor
  35,  // Camel
  55,  // Zebra
  106, // Mann
  2,   // Nightrider
  97,  // Champion
  61,  // Centaur
  132, // Amazon
  0,   // Elephant
  28,  // Obstacle
  0,   // Wall
];

const MG_MOBILITY_BONUS: [i32; 18] = [
  0, // Pawn
  0, // Knight
  3, // Bishop
  6, // Rook
  3, // Queen
  0, // King
  2, // Archbishop
  2, // Chancellor
  0, // Camel
  0, // Zebra
  0, // Mann
  8, // Nightrider
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
  9,  // Rook
  9,  // Queen
  0,  // King
  1,  // Archbishop
  16, // Chancellor
  0,  // Camel
  0,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  16, // Amazon
  0,  // Elephant
  0,  // Obstacle
  0,  // Wall
];

const MG_PAWN_ATTACKED_PENALTY: [i32; 18] = [
  -18, // Pawn
  52,  // Knight
  56,  // Bishop
  53,  // Rook
  31,  // Queen
  0,   // King
  30,  // Archbishop
  54,  // Chancellor
  53,  // Camel
  61,  // Zebra
  69,  // Mann
  52,  // Nightrider
  27,  // Champion
  22,  // Centaur
  45,  // Amazon
  28,  // Elephant
  0,   // Obstacle
  22,  // Wall
];

const EG_PAWN_ATTACKED_PENALTY: [i32; 18] = [
  -12,  // Pawn
  28,   // Knight
  45,   // Bishop
  49,   // Rook
  32,   // Queen
  0,    // King
  -11,  // Archbishop
  -100, // Chancellor
  69,   // Camel
  67,   // Zebra
  21,   // Mann
  -44,  // Nightrider
  45,   // Champion
  -20,  // Centaur
  -208, // Amazon
  86,   // Elephant
  6,    // Obstacle
  56,   // Wall
];

const MG_PAWN_DEFENDED_BONUS: [i32; 18] = [
  11,  // Pawn
  5,   // Knight
  8,   // Bishop
  -26, // Rook
  -9,  // Queen
  -40, // King
  0,   // Archbishop
  -1,  // Chancellor
  15,  // Camel
  24,  // Zebra
  -14, // Mann
  11,  // Nightrider
  -5,  // Champion
  5,   // Centaur
  -8,  // Amazon
  -12, // Elephant
  22,  // Obstacle
  -10, // Wall
];

const EG_PAWN_DEFENDED_BONUS: [i32; 18] = [
  2,   // Pawn
  -6,  // Knight
  4,   // Bishop
  55,  // Rook
  26,  // Queen
  24,  // King
  -19, // Archbishop
  -6,  // Chancellor
  -12, // Camel
  -22, // Zebra
  60,  // Mann
  112, // Nightrider
  7,   // Champion
  10,  // Centaur
  90,  // Amazon
  23,  // Elephant
  -5,  // Obstacle
  0,   // Wall
];

// advanced pawns get a bonus of numerator/(factor * squares_to_promotion + bonus) times the promotion value
pub(crate) const PAWN_SCALING_NUMERATOR: i32 = 20;
const MG_PAWN_SCALING_FACTOR: i32 = 276;
const MG_PAWN_SCALING_BONUS: i32 = -11;
const EG_PAWN_SCALING_FACTOR: i32 = 146;
const EG_PAWN_SCALING_BONUS: i32 = -56;

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
  pub(crate) mg_pawn_scale_factor: i32,
  pub(crate) mg_pawn_scaling_bonus: i32,
  pub(crate) eg_pawn_scale_factor: i32,
  pub(crate) eg_pawn_scaling_bonus: i32,
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
      mg_pawn_scale_factor: value.mg_pawn_scale_factor,
      mg_pawn_scaling_bonus: value.mg_pawn_scaling_bonus,
      eg_pawn_scale_factor: value.eg_pawn_scale_factor,
      eg_pawn_scaling_bonus: value.eg_pawn_scaling_bonus,
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
  mg_pawn_scale_factor: MG_PAWN_SCALING_FACTOR,
  mg_pawn_scaling_bonus: MG_PAWN_SCALING_BONUS,
  eg_pawn_scale_factor: EG_PAWN_SCALING_FACTOR,
  eg_pawn_scaling_bonus: EG_PAWN_SCALING_BONUS,
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
  pub mg_pawn_scale_factor: T,
  /// Scaling factor for the advanced pawn bonus
  pub mg_pawn_scaling_bonus: T,
  /// Scaling factor for the advanced pawn bonus
  pub eg_pawn_scale_factor: T,
  /// Scaling factor for the advanced pawn bonus
  pub eg_pawn_scaling_bonus: T,
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
    self.mg_pawn_scale_factor += rhs.mg_pawn_scale_factor;
    self.mg_pawn_scaling_bonus += rhs.mg_pawn_scaling_bonus;
    self.eg_pawn_scale_factor += rhs.eg_pawn_scale_factor;
    self.eg_pawn_scaling_bonus += rhs.eg_pawn_scaling_bonus;
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
      mg_pawn_scale_factor: self.mg_pawn_scale_factor / rhs,
      mg_pawn_scaling_bonus: self.mg_pawn_scaling_bonus / rhs,
      eg_pawn_scale_factor: self.eg_pawn_scale_factor / rhs,
      eg_pawn_scaling_bonus: self.eg_pawn_scaling_bonus / rhs,
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
    self.mg_pawn_scale_factor /= rhs.mg_pawn_scale_factor;
    self.mg_pawn_scaling_bonus /= rhs.mg_pawn_scaling_bonus;
    self.eg_pawn_scale_factor /= rhs.eg_pawn_scale_factor;
    self.eg_pawn_scaling_bonus /= rhs.eg_pawn_scaling_bonus;
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
      mg_pawn_scale_factor: self.mg_pawn_scale_factor * rhs,
      mg_pawn_scaling_bonus: self.mg_pawn_scaling_bonus * rhs,
      eg_pawn_scale_factor: self.eg_pawn_scale_factor * rhs,
      eg_pawn_scaling_bonus: self.eg_pawn_scaling_bonus * rhs,
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
      mg_pawn_scale_factor: self.mg_pawn_scale_factor.abs(),
      mg_pawn_scaling_bonus: self.mg_pawn_scaling_bonus.abs(),
      eg_pawn_scale_factor: self.eg_pawn_scale_factor.abs(),
      eg_pawn_scaling_bonus: self.eg_pawn_scaling_bonus.abs(),
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
      mg_pawn_scale_factor: Self::remove_nan(self.mg_pawn_scale_factor),
      mg_pawn_scaling_bonus: Self::remove_nan(self.mg_pawn_scaling_bonus),
      eg_pawn_scale_factor: Self::remove_nan(self.eg_pawn_scale_factor),
      eg_pawn_scaling_bonus: Self::remove_nan(self.eg_pawn_scaling_bonus),
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
      mg_pawn_scale_factor: f64::from(value.mg_pawn_scale_factor),
      mg_pawn_scaling_bonus: f64::from(value.mg_pawn_scaling_bonus),
      eg_pawn_scale_factor: f64::from(value.eg_pawn_scale_factor),
      eg_pawn_scaling_bonus: f64::from(value.eg_pawn_scaling_bonus),
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
