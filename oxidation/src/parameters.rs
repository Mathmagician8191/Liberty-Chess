use liberty_chess::parsing::to_name;
use std::ops::{Add, AddAssign, Div, Mul};

const PIECE_VALUES: [(i32, i32); 18] = [
  (67, 144),    // Pawn
  (323, 297),   // Knight
  (360, 263),   // Bishop
  (489, 481),   // Rook
  (1024, 998),  // Queen
  (-195, 887),  // King
  (832, 965),   // Archbishop
  (979, 1117),  // Chancellor
  (253, 195),   // Camel
  (179, 167),   // Zebra
  (169, 299),   // Mann
  (560, 313),   // Nightrider
  (503, 973),   // Champion
  (575, 1026),  // Centaur
  (1432, 1644), // Amazon
  (653, 633),   // Elephant
  (1, 25),      // Obstacle
  (44, 110),    // Wall
];

const MG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [-7, 16, 29, -10, 0],        // Pawn
  [43, 47, 38, 30, 13],        // Knight
  [53, 45, 25, -7, -5],        // Bishop
  [36, 34, 4, 7, 5],           // Rook
  [19, 13, 4, 18, -3],         // Queen
  [-165, -151, -91, -25, -25], // King
  [65, 17, 14, 16, 0],         // Archbishop
  [9, 13, 5, 62, -8],          // Chancellor
  [42, 89, 52, 62, 41],        // Camel
  [-28, 42, 9, 66, 28],        // Zebra
  [45, 15, 5, 0, 0],           // Mann
  [34, 34, 6, 71, -24],        // Nightrider
  [17, 58, 9, 54, 31],         // Champion
  [55, 30, 35, 39, 24],        // Centaur
  [15, 26, 1, 1, 1],           // Amazon
  [122, 100, 84, 66, 64],      // Elephant
  [0, 0, 0, 0, 0],             // Obstacle
  [0, 0, 0, 0, 0],             // Wall
];

const EG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [103, 0, 8, -12, -1],     // Pawn
  [29, 0, 1, 0, 3],         // Knight
  [0, 0, 0, 10, 4],         // Bishop
  [104, 25, 23, 20, 0],     // Rook
  [69, 22, 10, 12, 15],     // Queen
  [125, 105, 59, 23, 18],   // King
  [279, 148, 91, 184, 44],  // Archbishop
  [18, 86, 48, 44, 76],     // Chancellor
  [0, -16, -16, -13, -28],  // Camel
  [76, -10, -26, -15, -26], // Zebra
  [53, 0, 22, 19, 7],       // Mann
  [96, 19, 18, 29, 24],     // Nightrider
  [153, 0, 25, 17, 9],      // Champion
  [349, 186, 81, 56, 29],   // Centaur
  [151, 0, 0, 0, 0],        // Amazon
  [141, 112, 103, 71, 26],  // Elephant
  [0, 0, 0, 0, 0],          // Obstacle
  [0, 0, 0, 0, 0],          // Wall
];

const MG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,  // Pawn
  12, // Knight
  7,  // Bishop
  1,  // Rook
  1,  // Queen
  42, // King
  9,  // Archbishop
  0,  // Chancellor
  0,  // Camel
  0,  // Zebra
  1,  // Mann
  1,  // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  0,  // Elephant
  9,  // Obstacle
  0,  // Wall
];

const EG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  36,  // Pawn
  13,  // Knight
  2,   // Bishop
  0,   // Rook
  0,   // Queen
  -3,  // King
  19,  // Archbishop
  0,   // Chancellor
  5,   // Camel
  1,   // Zebra
  0,   // Mann
  0,   // Nightrider
  0,   // Champion
  11,  // Centaur
  111, // Amazon
  33,  // Elephant
  2,   // Obstacle
  2,   // Wall
];

const MG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  15,  // Knight
  2,   // Bishop
  -38, // Rook
  1,   // Queen
  122, // King
  25,  // Archbishop
  -8,  // Chancellor
  0,   // Camel
  0,   // Zebra
  0,   // Mann
  9,   // Nightrider
  50,  // Champion
  28,  // Centaur
  0,   // Amazon
  65,  // Elephant
  9,   // Obstacle
  9,   // Wall
];

const EG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  38,  // Knight
  100, // Bishop
  65,  // Rook
  16,  // Queen
  46,  // King
  82,  // Archbishop
  36,  // Chancellor
  29,  // Camel
  24,  // Zebra
  121, // Mann
  118, // Nightrider
  7,   // Champion
  70,  // Centaur
  26,  // Amazon
  30,  // Elephant
  30,  // Obstacle
  -18, // Wall
];

const MG_MOBILITY_BONUS: [i32; 18] = [
  0, // Pawn
  0, // Knight
  4, // Bishop
  6, // Rook
  2, // Queen
  0, // King
  2, // Archbishop
  2, // Chancellor
  0, // Camel
  0, // Zebra
  0, // Mann
  2, // Nightrider
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
  5,  // Bishop
  7,  // Rook
  7,  // Queen
  0,  // King
  0,  // Archbishop
  5,  // Chancellor
  0,  // Camel
  0,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  13, // Amazon
  0,  // Elephant
  0,  // Obstacle
  0,  // Wall
];

// advanced pawns get a bonus of numerator/(factor * squares_to_promotion + bonus) times the promotion value
pub(crate) const PAWN_SCALING_NUMERATOR: i32 = 12;
const PAWN_SCALING_FACTOR: i32 = 89;
const PAWN_SCALING_BONUS: i32 = -21;

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
  mg_edge: MG_EDGE_AVOIDANCE,
  eg_edge: EG_EDGE_AVOIDANCE,
  mg_friendly_pawn_penalty: MG_FRIENDLY_PAWN_PENALTY,
  eg_friendly_pawn_penalty: EG_FRIENDLY_PAWN_PENALTY,
  mg_enemy_pawn_penalty: MG_ENEMY_PAWN_PENALTY,
  eg_enemy_pawn_penalty: EG_ENEMY_PAWN_PENALTY,
  mg_mobility_bonus: MG_MOBILITY_BONUS,
  eg_mobility_bonus: EG_MOBILITY_BONUS,
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
      mg_mobility_bonus: self.mg_mobility_bonus.map(|x| x / rhs),
      eg_mobility_bonus: self.eg_mobility_bonus.map(|x| x / rhs),
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
      mg_mobility_bonus: self.mg_mobility_bonus.map(|x| x * rhs),
      eg_mobility_bonus: self.eg_mobility_bonus.map(|x| x * rhs),
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
      mg_mobility_bonus: value.mg_mobility_bonus.map(f64::from),
      eg_mobility_bonus: value.eg_mobility_bonus.map(f64::from),
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
    result + "\n];"
  }
}
