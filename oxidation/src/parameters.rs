use liberty_chess::parsing::to_name;
use liberty_chess::{CENTAUR, CHAMPION, ELEPHANT, KING, MANN, OBSTACLE, WALL};
use std::ops::{Add, AddAssign, Div, Mul};

const PIECE_VALUES: [(i32, i32); 18] = [
  (55, 138),    // Pawn
  (286, 346),   // Knight
  (323, 295),   // Bishop
  (451, 537),   // Rook
  (1060, 966),  // Queen
  (-269, 780),  // King
  (828, 1020),  // Archbishop
  (979, 1156),  // Chancellor
  (243, 225),   // Camel
  (179, 203),   // Zebra
  (166, 337),   // Mann
  (543, 350),   // Nightrider
  (477, 884),   // Champion
  (540, 1121),  // Centaur
  (1540, 1494), // Amazon
  (681, 600),   // Elephant
  (2, 38),      // Obstacle
  (30, 145),    // Wall
];

const MG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [-64, 0, -7, 21, -18, -16, -6, -12, -6],    // Pawn
  [34, 74, 39, 23, 48, 16, 18, 0, -4],        // Knight
  [46, 20, 51, 0, 0, -19, -13, -13, 2],       // Bishop
  [15, 16, -3, -19, -3, -9, -2, -15, -7],     // Rook
  [-5, 4, 5, -14, 18, -5, -5, 2, 3],          // Queen
  [-24, -46, -15, 10, 44, 18, 55, 6, 48],     // King
  [51, 5, 5, 12, 8, 2, -13, -11, -2],         // Archbishop
  [-23, -21, -25, 4, 36, -28, -11, -15, -8],  // Chancellor
  [-2, 94, 2, 14, 10, 24, 0, 62, 17],         // Camel
  [22, 43, -29, 22, 24, 24, 5, 19, -9],       // Zebra
  [69, 18, 76, 45, 35, 0, 0, 0, 25],          // Mann
  [13, -12, 35, -52, 35, -16, -46, -56, -14], // Nightrider
  [0, 29, 19, 0, 0, 0, 0, 0, 0],              // Champion
  [56, 44, 44, 35, 52, 30, 19, 11, 11],       // Centaur
  [19, 38, 33, -15, -36, -30, -21, 11, -15],  // Amazon
  [231, 208, 162, 119, 106, 85, 75, 117, 63], // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                // Wall
];

const EG_EDGE_AVOIDANCE: [[i32; EDGE_PARAMETER_COUNT]; 18] = [
  [127, 4, 20, 9, -8, 4, 2, 4, 2],             // Pawn
  [32, -16, 0, 18, -8, 2, 8, 6, 6],            // Knight
  [-36, -7, -20, 1, 7, 1, 7, 3, -11],          // Bishop
  [76, 24, 15, 21, 11, 6, -20, -1, -20],       // Rook
  [61, -6, -1, 13, -24, 2, -21, -28, -30],     // Queen
  [101, 85, 58, 40, 17, 22, 4, 19, -2],        // King
  [161, 156, 77, 51, -10, 7, 58, 33, -29],     // Archbishop
  [84, 45, 102, -52, -118, -21, -1, -76, -64], // Chancellor
  [20, -4, 10, -3, -16, 3, 4, -26, -7],        // Camel
  [10, 12, 34, -4, 12, -12, -17, -4, 13],      // Zebra
  [0, 101, 0, 13, 0, 21, 16, 0, 0],            // Mann
  [22, 32, -5, 10, 23, 18, 19, 71, 32],        // Nightrider
  [81, 100, 54, 110, 0, 0, 0, 0, 6],           // Champion
  [367, 142, 97, 48, 97, 103, 24, 46, 3],      // Centaur
  [52, -32, -52, -23, 8, 155, -47, 104, -3],   // Amazon
  [145, 131, 138, 120, 134, 85, 65, 15, 15],   // Elephant
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                 // Obstacle
  [0, 0, 0, 0, 0, 0, 0, 0, 0],                 // Wall
];

const MG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  0,  // Pawn
  11, // Knight
  0,  // Bishop
  0,  // Rook
  10, // Queen
  29, // King
  0,  // Archbishop
  9,  // Chancellor
  0,  // Camel
  0,  // Zebra
  0,  // Mann
  12, // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  0,  // Elephant
  0,  // Obstacle
  0,  // Wall
];

const EG_FRIENDLY_PAWN_PENALTY: [i32; 18] = [
  26, // Pawn
  14, // Knight
  14, // Bishop
  0,  // Rook
  0,  // Queen
  0,  // King
  6,  // Archbishop
  0,  // Chancellor
  0,  // Camel
  7,  // Zebra
  0,  // Mann
  0,  // Nightrider
  0,  // Champion
  0,  // Centaur
  0,  // Amazon
  8,  // Elephant
  24, // Obstacle
  11, // Wall
];

const MG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  19,  // Knight
  5,   // Bishop
  -14, // Rook
  1,   // Queen
  55,  // King
  24,  // Archbishop
  4,   // Chancellor
  18,  // Camel
  -30, // Zebra
  19,  // Mann
  20,  // Nightrider
  34,  // Champion
  31,  // Centaur
  6,   // Amazon
  55,  // Elephant
  0,   // Obstacle
  0,   // Wall
];

const EG_ENEMY_PAWN_PENALTY: [i32; 18] = [
  0,   // Pawn
  21,  // Knight
  78,  // Bishop
  81,  // Rook
  51,  // Queen
  50,  // King
  103, // Archbishop
  92,  // Chancellor
  3,   // Camel
  26,  // Zebra
  121, // Mann
  65,  // Nightrider
  72,  // Champion
  69,  // Centaur
  75,  // Amazon
  23,  // Elephant
  41,  // Obstacle
  0,   // Wall
];

const MG_MOBILITY_BONUS: [i32; 18] = [
  0, // Pawn
  0, // Knight
  4, // Bishop
  5, // Rook
  4, // Queen
  0, // King
  1, // Archbishop
  1, // Chancellor
  0, // Camel
  0, // Zebra
  0, // Mann
  4, // Nightrider
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
  8,  // Rook
  8,  // Queen
  0,  // King
  2,  // Archbishop
  10, // Chancellor
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

// advanced pawns get a bonus of numerator/(factor * squares_to_promotion + bonus) times the promotion value
pub(crate) const PAWN_SCALING_NUMERATOR: i32 = 40;
const PAWN_SCALING_FACTOR: i32 = 305;
const PAWN_SCALING_BONUS: i32 = -93;

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
