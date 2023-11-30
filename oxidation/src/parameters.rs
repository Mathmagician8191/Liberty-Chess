/// Values of all the pieces in the middlegame
pub const MIDDLEGAME_PIECE_VALUES: [i32; 18] = [
  84,   // pawn
  271,  // knight
  421,  // bishop
  572,  // rook
  1174, // queen
  154,  // king
  958,  // archbishop
  1069, // chancellor
  117,  // camel
  196,  // zebra
  223,  // mann
  461,  // nightrider
  621,  // champion
  581,  // centaur
  2400, // amazon
  900,  // elephant - tuned value (350) tweaked due to regressions
  30,   // obstacle
  134,  // wall
];

/// Penalties for being on the edge of the board in the middlegame
pub const MIDDLEGAME_EDGE_AVOIDANCE: [i32; 18] = [
  59,  // pawn
  52,  // knight
  60,  // bishop
  14,  // rook
  24,  // queen
  -12, // king
  26,  // archbishop
  -5,  // chancellor
  81,  // camel
  -13, // zebra
  30,  // mann - tuned value (-175) reverted due to regression
  -51, // nightrider
  25,  // champion
  32,  // centaur
  10,  // amazon
  1,   // elephant - tuned value (-55) tweaked due to regressions
  0,   // obstacle
  0,   // wall
];

/// Values of all the pieces in the endgame
pub const ENDGAME_PIECE_VALUES: [i32; 18] = [
  160,  // pawn
  259,  // knight
  263,  // bishop
  423,  // rook
  835,  // queen
  700,  // king
  730,  // archbishop
  994,  // chancellor
  224,  // camel
  143,  // zebra
  106,  // mann
  260,  // nightrider - tuned value (15) increased due to regressions
  417,  // champion
  671,  // centaur
  2600, // amazon
  800,  // elephant - tuned value (255) adjusted due to regressions
  5,    // obstacle
  9,    // wall
];

/// Penalties for being on the edge of the board in the endgame
pub const ENDGAME_EDGE_AVOIDANCE: [i32; 18] = [
  -6,   // pawn
  50,   // knight
  68,   // bishop
  14,   // rook
  -3,   // queen
  28,   // king
  89,   // archbishop
  130,  // chancellor
  -111, // camel
  -5,   // zebra
  30,   // mann - tuned value (-20) reverted due to regressions
  -115, // nightrider
  217,  // champion
  75,   // centaur
  0,    // amazon
  1,    // elephant
  0,    // obstacle
  0,    // wall
];

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
