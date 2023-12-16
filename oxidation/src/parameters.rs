/// Values of all the pieces in the middlegame
pub const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  87,   // pawn
  271,  // knight
  381,  // bishop
  582,  // rook
  1280, // queen
  254,  // king
  1045, // archbishop
  1139, // chancellor
  288,  // camel
  98,   // zebra
  223,  // mann - tuned value (163) reverted due to regressions
  448,  // nightrider
  526,  // champion
  631,  // centaur
  1763, // amazon
  280,  // elephant
  30,   // obstacle
  181,  // wall
];
// [
//   84,   // pawn
//   271,  // knight
//   421,  // bishop
//   572,  // rook
//   1174, // queen
//   154,  // king
//   958,  // archbishop
//   1069, // chancellor
//   117,  // camel
//   196,  // zebra
//   223,  // mann
//   461,  // nightrider
//   621,  // champion
//   581,  // centaur
//   2400, // amazon
//   900,  // elephant - tuned value (350) tweaked due to regressions
//   30,   // obstacle
//   134,  // wall
// ];

// Penalties for being on the edge of the board in the middlegame
const MIDDLEGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [39, 2],    // pawn
  [53, 21],   // knight
  [69, 5],    // bishop
  [46, 49],   // rook
  [18, 5],    // queen
  [-91, -38], // king
  [50, -16],  // archbishop
  [1, -1],    // chancellor
  [3, 0],     // camel
  [11, 89],   // zebra
  [30, 10],   // mann - tuned value [-136, -18] tweaked due to regression
  [17, -62],  // nightrider
  [93, 30],   // champion
  [50, 61],   // centaur - tuned value [-2, 61] tweaked due to regressions
  [-41, 451], // amazon
  [36, 34],   // elephant
  [0, 0],     // obstacle
  [0, 0],     // wall
];
// [
//   [
//     59,  // pawn
//     52,  // knight
//     60,  // bishop
//     14,  // rook
//     24,  // queen
//     -12, // king
//     26,  // archbishop
//     -5,  // chancellor
//     81,  // camel
//     -13, // zebra
//     30,  // mann - tuned value (-175) reverted due to regression
//     -51, // nightrider
//     25,  // champion
//     32,  // centaur
//     10,  // amazon
//     2,   // elephant - tuned value (-55) tweaked due to regressions
//     0,   // obstacle
//     0,   // wall
//   ],
// ];

// Values of all the pieces in the endgame
const ENDGAME_PIECE_VALUES: [i32; 18] = [
  185,  // pawn
  320,  // knight
  366,  // bishop
  551,  // rook
  963,  // queen
  885,  // king
  749,  // archbishop
  1140, // chancellor
  229,  // camel
  268,  // zebra
  180,  // mann
  321,  // nightrider - tuned value tweaked to be more than the knight
  696,  // champion
  671,  // centaur - tuned value (1032) reverted due to regressions
  2150, // amazon
  519,  // elephant
  1,    // obstacle
  1,    // wall
];
// [
//   160,  // pawn
//   259,  // knight
//   263,  // bishop
//   423,  // rook
//   835,  // queen
//   700,  // king
//   730,  // archbishop
//   994,  // chancellor
//   224,  // camel
//   143,  // zebra
//   106,  // mann
//   260,  // nightrider - tuned value (15) increased due to regressions
//   417,  // champion
//   671,  // centaur
//   2600, // amazon
//   800,  // elephant - tuned value (255) adjusted due to regressions
//   5,    // obstacle
//   9,    // wall
// ];

// Penalties for being on the edge of the board in the endgame
const ENDGAME_EDGE_AVOIDANCE: [[i32; EDGE_DISTANCE]; 18] = [
  [21, 0],    // pawn
  [-9, 10],   // knight
  [40, 6],    // bishop
  [76, -5],   // rook
  [76, 17],   // queen
  [107, 32],  // king
  [138, 99],  // archbishop
  [32, 15],   // chancellor
  [64, 61],   // camel
  [9, -8],    // zebra
  [30, 10],   // mann - tuned value [-579, 32] tweaked due to regression
  [79, -56],  // nightrider
  [363, 230], // champion
  [35, 0],    // centaur - tuned value [-115, -8] tweaked due to regressions
  [23, 212],  // amazon
  [83, 71],   // elephant
  [0, 0],     // obstacle
  [0, 0],     // wall
];
// [
//   [
//     -6,   // pawn
//     50,   // knight
//     68,   // bishop
//     14,   // rook
//     -3,   // queen
//     28,   // king
//     89,   // archbishop
//     130,  // chancellor
//     -111, // camel
//     -5,   // zebra
//     30,   // mann - tuned value (-20) reverted due to regressions
//     -115, // nightrider
//     217,  // champion
//     75,   // centaur
//     0,    // amazon
//     2,    // elephant
//     0,    // obstacle
//     0,    // wall
//   ],
// ];

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
}
