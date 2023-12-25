/// Values of all the pieces in the middlegame
pub const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  80,   // pawn
  275,  // knight
  365,  // bishop
  529,  // rook
  1193, // queen
  -6,   // king
  842,  // archbishop
  1100, // chancellor
  173,  // camel
  151,  // zebra
  86,   // mann
  428,  // nightrider
  526,  // champion - tuned value (424) reverted due to regressions
  631,  // centaur - tuned value (573) reverted due to regressions
  2343, // elephant
  510,  // elephant
  30,   // obstacle
  148,  // wall
];

// Penalties for being on the edge of the board in the middlegame
const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [63, 9],     // pawn
  [37, 19],    // knight
  [80, -8],    // bishop
  [55, 48],    // rook
  [57, 25],    // queen
  [-113, -55], // king
  [6, 4],      // archbishop
  [47, 23],    // chancellor
  [-51, -7],   // camel
  [28, 46],    // zebra
  [30, 10],    // mann - tuned value [-178, -13] reverted due to history of regressions
  [-52, -10],  // nightrider
  [1, 0],      // champion - tuned value [-24, -98] tweaked due to regressions
  [31, 5],     // centaur
  [222, 224],  // amazon
  [76, -6],    // elephant
  [0, 0],      // obstacle
  [0, 0],      // wall
];

// Values of all the pieces in the endgame
const ENDGAME_PIECE_VALUES: [i32; 18] = [
  197,  // pawn
  365,  // knight
  399,  // bishop
  683,  // rook
  925,  // queen
  910,  // king
  886,  // archbishop
  1219, // chancellor
  304,  // camel
  204,  // zebra
  307,  // mann
  366,  // nightrider - tuned value (277) tweaked to be higher than the knight
  696,  // champion - tuned value (1191) reverted due to regressions
  671,  // centaur - tuned value (1195) reverted due to regressions
  2720, // amazon
  699,  // elephant
  2,    // obstacle
  53,   // wall
];

// Penalties for being on the edge of the board in the endgame
const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [6, -2],    // pawn
  [9, 10],    // knight
  [29, 23],   // bishop
  [58, 6],    // rook
  [-26, -39], // queen
  [86, 38],   // king
  [138, 36],  // archbishop
  [29, 0],    // chancellor - tuned value [29, -83] tweaked due to regressions
  [22, -11],  // camel
  [-8, -12],  // zebra
  [82, 111],  // mann
  [-16, 104], // nightrider
  [126, 61],  // champion - tuned value [626, 161] tweaked due to regressions
  [91, 0],    // centaur - tuned value [91, -62] reverted due to regressions
  [51, 19],   // amazon
  [110, 91],  // elephant
  [0, 0],     // obstacle
  [0, 0],     // wall
];

/// Maximum distance from the edge to apply penalty
pub const EDGE_DISTANCE: usize = 2;

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
  middlegame_pieces: MIDDLEGAME_PIECE_VALUES,
  middlegame_edge: MIDDLEGAME_EDGE_AVOIDANCE,
  endgame_pieces: ENDGAME_PIECE_VALUES,
  endgame_edge: ENDGAME_EDGE_AVOIDANCE,
  min_halfmoves: 20,
  halfmove_scaling: 80,
};

/// Parameters for evaluation
#[derive(Copy, Clone, Debug)]
pub struct Parameters {
  /// The values of pieces in the middlegame
  pub middlegame_pieces: [i32; 18],
  /// Penalties for being on the edge of the board in the middlegame
  pub middlegame_edge: [[i32; EDGE_DISTANCE]; 18],
  /// The values of pieces in the endgame
  pub endgame_pieces: [i32; 18],
  /// Penalties for being on the edge of the board in the endgame
  pub endgame_edge: [[i32; EDGE_DISTANCE]; 18],
  /// Minimum halfmoves for scaling
  pub min_halfmoves: u8,
  /// Halfmove scaling factor
  pub halfmove_scaling: u8,
}

impl Parameters {
  /// How many parameters there are
  pub const COUNT: usize = 108;

  /// Set a parameter
  pub fn set_parameter(&mut self, parameter: usize, bonus: i32) {
    let index = parameter % 18;
    if parameter >= 90 {
      self.middlegame_pieces[index] += bonus;
    } else if parameter >= 72 {
      self.middlegame_edge[index][0] += bonus;
    } else if parameter >= 54 {
      self.middlegame_edge[index][1] += bonus;
    } else if parameter >= 36 {
      self.endgame_pieces[index] += bonus;
    } else if parameter >= 18 {
      self.endgame_edge[index][0] += bonus;
    } else {
      self.endgame_edge[index][1] += bonus;
    }
  }
}
