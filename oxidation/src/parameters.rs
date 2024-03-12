use liberty_chess::parsing::to_name;
use std::ops::{Add, AddAssign, Div, Mul};

const PIECE_VALUES: [(i32, i32); 18] = [
  (59, 143),    // Pawn
  (288, 298),   // Knight
  (357, 314),   // Bishop
  (467, 582),   // Rook
  (979, 1108),  // Queen
  (-431, 1209), // King
  (751, 948),   // Archbishop
  (882, 1226),  // Chancellor
  (171, 250),   // Camel
  (165, 179),   // Zebra
  (186, 261),   // Mann
  (534, 311),   // Nightrider
  (427, 850),   // Champion
  (502, 953),   // Centaur
  (1337, 1706), // Amazon
  (621, 621),   // Elephant
  (1, 94),      // Obstacle
  (2, 123),     // Wall
];

const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [0, 15, 25, -19, -3],     // Pawn
  [50, 55, 42, 26, 7],      // Knight
  [58, 61, 45, 5, -6],      // Bishop
  [52, 39, 21, 11, 5],      // Rook
  [52, 39, 29, 31, 6],      // Queen
  [-123, -83, -40, 18, 27], // King
  [62, 37, 21, -9, 0],      // Archbishop
  [28, 9, -3, 24, -23],     // Chancellor
  [10, 32, -38, -2, -21],   // Camel
  [-14, -24, -14, 15, -20], // Zebra
  [0, 0, 5, 0, 0],          // Mann
  [0, 0, 0, 0, 0],          // Nightrider
  [0, 2, 2, 54, 0],         // Champion
  [45, 42, 33, 35, 24],     // Centaur
  [5, 4, 3, 2, 1],          // Amazon
  [159, 109, 51, 65, 0],    // Elephant
  [0, 0, 0, 0, 0],          // Obstacle
  [0, 0, 0, 0, 0],          // Wall
];

const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [121, 0, 9, -8, -2],     // Pawn
  [22, -3, -2, 3, 9],      // Knight
  [15, 19, 38, 26, 15],    // Bishop
  [97, 39, 9, 22, 7],      // Rook
  [71, 28, 30, 19, 10],    // Queen
  [100, 73, 47, 12, 1],    // King
  [282, 113, 41, -20, 30], // Archbishop
  [-23, -39, 0, -27, 7],   // Chancellor
  [-16, 21, 43, 24, 23],   // Camel
  [-11, 84, 0, 55, 6],     // Zebra
  [43, 23, 23, 45, 24],    // Mann
  [0, 0, 0, 0, 0],         // Nightrider
  [0, 84, 0, 50, 0],       // Champion
  [218, 156, 156, 45, 65], // Centaur
  [0, 0, 0, 0, 0],         // Amazon
  [100, 94, 88, 106, 79],  // Elephant
  [0, 0, 0, 0, 0],         // Obstacle
  [0, 0, 0, 0, 0],         // Wall
];

const MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,  // Pawn
  9,  // Knight
  11, // Bishop
  0,  // Rook
  1,  // Queen
  65, // King
  1,  // Archbishop
  10, // Chancellor
  0,  // Camel
  0,  // Zebra
  22, // Mann
  0,  // Nightrider
  0,  // Champion
  3,  // Centaur
  0,  // Amazon
  0,  // Elephant
  0,  // Obstacle
  0,  // Wall
];

const ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  27, // Pawn
  15, // Knight
  0,  // Bishop
  13, // Rook
  0,  // Queen
  0,  // King
  0,  // Archbishop
  41, // Chancellor
  28, // Camel
  1,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  37, // Centaur
  0,  // Amazon
  38, // Elephant
  28, // Obstacle
  10, // Wall
];

const MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  17,  // Knight
  9,   // Bishop
  -19, // Rook
  0,   // Queen
  72,  // King
  15,  // Archbishop
  10,  // Chancellor
  10,  // Camel
  1,   // Zebra
  1,   // Mann
  -1,  // Nightrider
  61,  // Champion
  32,  // Centaur
  23,  // Amazon
  56,  // Elephant
  0,   // Obstacle
  0,   // Wall
];

const ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  40,  // Knight
  112, // Bishop
  37,  // Rook
  49,  // Queen
  63,  // King
  137, // Archbishop
  19,  // Chancellor
  4,   // Camel
  25,  // Zebra
  141, // Mann
  90,  // Nightrider
  52,  // Champion
  62,  // Centaur
  73,  // Amazon
  55,  // Elephant
  63,  // Obstacle
  0,   // Wall
];

// advanced pawns get a bonus of numerator/(factor * squares_to_promotion + bonus) times the promotion value
pub(crate) const PAWN_SCALING_NUMERATOR: i32 = 9;
const PAWN_SCALING_FACTOR: i32 = 60;
const PAWN_SCALING_BONUS: i32 = -7;

pub(crate) const TEMPO_BONUS: i32 = 10;

/// Maximum distance from the edge to apply penalty
pub(crate) const EDGE_DISTANCE: usize = 2;
pub(crate) const EDGE_PARAMETER_COUNT: usize = EDGE_DISTANCE * (EDGE_DISTANCE + 3) / 2;
pub(crate) const INDEXING: [usize; (EDGE_DISTANCE + 1) * (EDGE_DISTANCE + 1)] =
  [0, 1, 2, 1, 3, 4, 2, 4, 5];

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
pub const DEFAULT_PARAMETERS: Parameters<i32> = Parameters {
  pieces: PIECE_VALUES,
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
  /// Scaling factor for the advanced pawn bonus
  pub pawn_scale_factor: T,
  /// Scaling factor for the advanced pawn bonus
  pub pawn_scaling_bonus: T,
}

impl<T: Default> Default for Parameters<T> {
  fn default() -> Self {
    Self {
      pieces: Default::default(),
      mg_edge: Default::default(),
      eg_edge: Default::default(),
      mg_friendly_pawn_penalty: Default::default(),
      eg_friendly_pawn_penalty: Default::default(),
      mg_enemy_pawn_penalty: Default::default(),
      eg_enemy_pawn_penalty: Default::default(),
      pawn_scale_factor: T::default(),
      pawn_scaling_bonus: T::default(),
    }
  }
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
    }
    for i in 0..18 {
      for j in 0..5 {
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
      for j in 0..5 {
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
      pawn_scale_factor: Self::remove_nan(self.pawn_scale_factor),
      pawn_scaling_bonus: Self::remove_nan(self.pawn_scaling_bonus),
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
    result += "\n];\n\nconst MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {:?}, // {}",
        self.mg_edge[i].map(|x| x as i32),
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {:?}, // {}",
        self.eg_edge[i].map(|x| x as i32),
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MIDDLEGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_friendly_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst ENDGAME_FRIENDLY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_friendly_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst MIDDLEGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.mg_enemy_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result += "\n];\n\nconst ENDGAME_ENEMY_PAWN_PENALTY: [i32; 18] = [";
    for i in 0..18 {
      result += &format!(
        "\n  {}, // {}",
        self.eg_enemy_pawn_penalty[i] as i32,
        to_name(i as i8 + 1)
      );
    }
    result + "\n];"
  }
}
