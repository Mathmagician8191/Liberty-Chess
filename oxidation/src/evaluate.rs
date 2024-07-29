use crate::parameters::{
  Parameters, EDGE_DISTANCE, EDGE_PARAMETER_COUNT, ENDGAME_FACTOR, ENDGAME_THRESHOLD, INDEXING,
  TEMPO_BONUS,
};
use crate::{State, DRAW_SCORE};
use array2d::Array2D;
use liberty_chess::{Board, Gamestate, Piece, OBSTACLE, PAWN, WALL};
use std::cmp::min;
use std::ops::{Add, AddAssign, Div, Mul, Neg, Sub, SubAssign};
use ulci::Score;

#[cfg(not(feature = "feature_extraction"))]
use crate::parameters::{pack, unpack_eg, unpack_mg, PackedParameters};

/// Extracted evaluation features
#[derive(Clone)]
pub struct Features {
  material: i32,
  pieces: [i8; 18],
  indexes: [[i8; EDGE_PARAMETER_COUNT]; 18],
  friendly_pawns: [i8; 18],
  enemy_pawns: [i8; 18],
  mobility: [i16; 18],
  attacked_by_pawn: [i8; 18],
  defended_by_pawn: [i8; 18],
  // squares to go and multiplier
  pawn_list: Vec<(u8, i8)>,
}

#[must_use]
#[cfg(not(feature = "feature_extraction"))]
pub(crate) fn raw(
  pieces: &Array2D<Piece>,
  to_move: bool,
  promotion_values: (i32, i32),
  parameters: &PackedParameters,
) -> i32 {
  let mut value = 0;
  let mut material = 0;
  let height = pieces.num_rows();
  let width = pieces.num_columns();
  for i in 0..height {
    for j in 0..width {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let (multiplier, block_i, defend_i, enemy_pawn, friendly_pawn) = if piece > 0 {
          (1, i + 1, i.wrapping_sub(1), Some(&-PAWN), Some(&PAWN))
        } else {
          (-1, i.wrapping_sub(1), i + 1, Some(&PAWN), Some(&-PAWN))
        };
        let piece_type = piece.unsigned_abs() as usize - 1;
        material += ENDGAME_FACTOR[piece_type];
        let mut piece_value = parameters.pieces[piece_type];
        let mobility = Board::mobility(pieces, (i, j), piece);
        piece_value += mobility * parameters.mobility_bonus[piece_type];
        let horizontal_distance = min(i, height - 1 - i).min(EDGE_DISTANCE);
        let vertical_distance = min(j, width - 1 - j).min(EDGE_DISTANCE);
        let index = INDEXING[horizontal_distance * (EDGE_DISTANCE + 1) + vertical_distance];
        if index < EDGE_PARAMETER_COUNT {
          piece_value -= parameters.edge_avoidance[piece_type][index];
        }
        if pieces.get(block_i, j.wrapping_sub(1)) == enemy_pawn
          || pieces.get(block_i, j + 1) == enemy_pawn
        {
          piece_value -= parameters.pawn_attacked_penalty[piece_type];
        }
        if pieces.get(defend_i, j.wrapping_sub(1)) == friendly_pawn
          || pieces.get(defend_i, j + 1) == friendly_pawn
        {
          piece_value += parameters.pawn_defended_bonus[piece_type];
        }
        if piece.abs() == PAWN {
          // penalty for pawn being blocked
          if let Some(piece) = pieces.get(block_i, j) {
            if *piece != 0 {
              let abs_piece = usize::from(piece.unsigned_abs());
              if (*piece > 0) ^ (multiplier > 0) {
                piece_value -= parameters.enemy_pawn_penalty[abs_piece - 1];
              } else {
                piece_value -= parameters.friendly_pawn_penalty[abs_piece - 1];
              }
            }
          }
          // bonus for advanced pawn
          let squares_to_go = if piece > 0 { height - 1 - i } else { i } as i32;
          if squares_to_go != 0 {
            let mg_divisor =
              squares_to_go * parameters.mg_pawn_scale_factor + parameters.mg_pawn_scaling_bonus;
            let eg_divisor =
              squares_to_go * parameters.eg_pawn_scale_factor + parameters.eg_pawn_scaling_bonus;
            let mg_value = promotion_values.0 / mg_divisor;
            let eg_value = promotion_values.1 / eg_divisor;
            piece_value += pack(mg_value, eg_value);
          }
        }
        value += piece_value * multiplier;
      }
    }
  }
  let middlegame = unpack_mg(value);
  let endgame = unpack_eg(value);
  material = min(material, ENDGAME_THRESHOLD);
  let score = material * middlegame + (ENDGAME_THRESHOLD - material) * endgame;
  let mut score = score / ENDGAME_THRESHOLD;
  if !to_move {
    score *= -1;
  }
  score += TEMPO_BONUS;
  score
}

/// Returns the static evaluation from the provided features
#[must_use]
pub fn eval_features<
  T: Copy
    + Add<Output = T>
    + Sub<Output = T>
    + Neg<Output = T>
    + Mul<Output = T>
    + Div<Output = T>
    + AddAssign
    + SubAssign
    + Default
    + From<i8>
    + From<u8>
    + From<i16>
    + From<i32>,
>(
  features: &Features,
  to_move: bool,
  promotion_values: (T, T),
  parameters: &Parameters<T>,
) -> T {
  let mut middlegame = T::default();
  let mut endgame = T::default();
  for piece_type in 0..18 {
    let piece_count = T::from(features.pieces[piece_type]);
    let (mg_value, eg_value) = parameters.pieces[piece_type];
    middlegame += mg_value * piece_count;
    endgame += eg_value * piece_count;
    let piece_count = T::from(features.friendly_pawns[piece_type]);
    middlegame -= parameters.mg_friendly_pawn_penalty[piece_type] * piece_count;
    endgame -= parameters.eg_friendly_pawn_penalty[piece_type] * piece_count;
    let piece_count = T::from(features.enemy_pawns[piece_type]);
    middlegame -= parameters.mg_enemy_pawn_penalty[piece_type] * piece_count;
    endgame -= parameters.eg_enemy_pawn_penalty[piece_type] * piece_count;
    let mobility = T::from(features.mobility[piece_type]);
    middlegame += parameters.mg_mobility_bonus[piece_type] * mobility;
    endgame += parameters.eg_mobility_bonus[piece_type] * mobility;
    let attacked_by_pawn = T::from(features.attacked_by_pawn[piece_type]);
    middlegame -= parameters.mg_pawn_attacked_penalty[piece_type] * attacked_by_pawn;
    endgame -= parameters.eg_pawn_attacked_penalty[piece_type] * attacked_by_pawn;
    let defended_by_pawn = T::from(features.defended_by_pawn[piece_type]);
    middlegame += parameters.mg_pawn_defended_bonus[piece_type] * defended_by_pawn;
    endgame += parameters.eg_pawn_defended_bonus[piece_type] * defended_by_pawn;
    let mg_edge = parameters.mg_edge[piece_type];
    let eg_edge = parameters.eg_edge[piece_type];
    let piece_count = features.indexes[piece_type];
    for index in 0..EDGE_PARAMETER_COUNT {
      let count = T::from(piece_count[index]);
      middlegame -= mg_edge[index] * count;
      endgame -= eg_edge[index] * count;
    }
  }
  for (squares_to_go, multiplier) in &features.pawn_list {
    let multiplier = T::from(*multiplier);
    let mg_divisor =
      T::from(*squares_to_go) * parameters.mg_pawn_scale_factor + parameters.mg_pawn_scaling_bonus;
    let eg_divisor =
      T::from(*squares_to_go) * parameters.eg_pawn_scale_factor + parameters.eg_pawn_scaling_bonus;
    middlegame += promotion_values.0 / mg_divisor * multiplier;
    endgame += promotion_values.1 / eg_divisor * multiplier;
  }
  let threshold = T::from(ENDGAME_THRESHOLD);
  let material = T::from(features.material);
  let score = material * middlegame + (threshold - material) * endgame;
  let mut score = score / threshold;
  if !to_move {
    score = -score;
  }
  score += T::from(TEMPO_BONUS);
  score
}

/// Calculates the derivative of evaluation wrt parameter values
#[must_use]
pub fn gradient(
  mut features: Features,
  promotion_values: (f64, f64),
  parameters: &Parameters<f64>,
) -> Parameters<f64> {
  // clear features that are not tuned
  features.indexes[OBSTACLE as usize - 1] = [0; EDGE_PARAMETER_COUNT];
  features.indexes[WALL as usize - 1] = [0; EDGE_PARAMETER_COUNT];
  let mg_factor = f64::from(features.material) / f64::from(ENDGAME_THRESHOLD);
  let eg_factor = 1.0 - mg_factor;
  let pieces = features.pieces.map(|x| {
    let x = f64::from(x);
    (x * mg_factor, x * eg_factor)
  });
  let mg_edge = features
    .indexes
    .map(|x| x.map(|x| -f64::from(x) * mg_factor));
  let eg_edge = features
    .indexes
    .map(|x| x.map(|x| -f64::from(x) * eg_factor));
  let mg_friendly_pawn_penalty = features.friendly_pawns.map(|x| -f64::from(x) * mg_factor);
  let eg_friendly_pawn_penalty = features.friendly_pawns.map(|x| -f64::from(x) * eg_factor);
  let mg_enemy_pawn_penalty = features.enemy_pawns.map(|x| -f64::from(x) * mg_factor);
  let eg_enemy_pawn_penalty = features.enemy_pawns.map(|x| -f64::from(x) * eg_factor);
  let mg_mobility_bonus = features.mobility.map(|x| f64::from(x) * mg_factor);
  let eg_mobility_bonus = features.mobility.map(|x| f64::from(x) * eg_factor);
  let mg_pawn_attacked_penalty = features.attacked_by_pawn.map(|x| -f64::from(x) * mg_factor);
  let eg_pawn_attacked_penalty = features.attacked_by_pawn.map(|x| -f64::from(x) * eg_factor);
  let mg_pawn_defended_bonus = features.defended_by_pawn.map(|x| f64::from(x) * mg_factor);
  let eg_pawn_defended_bonus = features.defended_by_pawn.map(|x| f64::from(x) * eg_factor);
  let mut mg_pawn_scale_factor = 0.0;
  let mut mg_pawn_scaling_bonus = 0.0;
  let mut eg_pawn_scale_factor = 0.0;
  let mut eg_pawn_scaling_bonus = 0.0;
  for (squares, count) in &features.pawn_list {
    let squares = f64::from(*squares);
    let mg_divisor = squares.mul_add(
      parameters.mg_pawn_scale_factor,
      parameters.mg_pawn_scaling_bonus,
    );
    let eg_divisor = squares.mul_add(
      parameters.eg_pawn_scale_factor,
      parameters.eg_pawn_scaling_bonus,
    );
    let mg_scaling_factor =
      -promotion_values.0 * mg_factor * f64::from(*count) / mg_divisor.powi(2);
    let eg_scaling_factor =
      -promotion_values.0 * eg_factor * f64::from(*count) / eg_divisor.powi(2);
    mg_pawn_scale_factor += mg_scaling_factor * squares;
    mg_pawn_scaling_bonus += mg_scaling_factor;
    eg_pawn_scale_factor += eg_scaling_factor * squares;
    eg_pawn_scaling_bonus += eg_scaling_factor;
  }
  Parameters {
    pieces,
    mg_edge,
    eg_edge,
    mg_friendly_pawn_penalty,
    eg_friendly_pawn_penalty,
    mg_enemy_pawn_penalty,
    eg_enemy_pawn_penalty,
    mg_mobility_bonus,
    eg_mobility_bonus,
    mg_pawn_attacked_penalty,
    eg_pawn_attacked_penalty,
    mg_pawn_defended_bonus,
    eg_pawn_defended_bonus,
    mg_pawn_scale_factor,
    mg_pawn_scaling_bonus,
    eg_pawn_scale_factor,
    eg_pawn_scaling_bonus,
  }
}

/// Returns the static evaluation from the provided raw data
#[must_use]
pub fn extract_features(pieces: &Array2D<Piece>) -> Features {
  let mut material = 0;
  let mut piece_counts = [0; 18];
  let mut indexes = [[0; EDGE_PARAMETER_COUNT]; 18];
  let mut friendly_pawns = [0; 18];
  let mut enemy_pawns = [0; 18];
  let mut mobility = [0; 18];
  let mut attacked_by_pawn = [0; 18];
  let mut defended_by_pawn = [0; 18];
  let mut pawn_list = Vec::new();
  let height = pieces.num_rows();
  let width = pieces.num_columns();
  for i in 0..height {
    for j in 0..width {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let (multiplier, block_i, defence_i) = if piece > 0 {
          (1, i + 1, i - 1)
        } else {
          (-1, i - 1, i + 1)
        };
        let piece_type = piece.unsigned_abs() as usize - 1;
        mobility[piece_type] +=
          i16::from(multiplier) * Board::mobility(pieces, (i, j), piece) as i16;
        material += ENDGAME_FACTOR[piece_type];
        piece_counts[piece_type] += multiplier;
        let horizontal_distance = min(i, height - 1 - i).min(EDGE_DISTANCE);
        let vertical_distance = min(j, width - 1 - j).min(EDGE_DISTANCE);
        let index = INDEXING[horizontal_distance * (EDGE_DISTANCE + 1) + vertical_distance];
        if index < EDGE_PARAMETER_COUNT {
          indexes[piece_type][index] += multiplier;
        }
        let temp = -multiplier;
        let enemy_pawn = Some(&temp);
        if pieces.get(block_i, j.wrapping_sub(1)) == enemy_pawn
          || pieces.get(block_i, j + 1) == enemy_pawn
        {
          attacked_by_pawn[piece_type] += multiplier;
        }
        if pieces.get(defence_i, j.wrapping_sub(1)) == Some(&multiplier)
          || pieces.get(defence_i, j + 1) == Some(&multiplier)
        {
          defended_by_pawn[piece_type] += multiplier;
        }
        if piece.abs() == PAWN {
          // penalty for pawn being blocked
          if let Some(piece) = pieces.get(block_i, j) {
            if *piece != 0 {
              let abs_piece = usize::from(piece.unsigned_abs());
              if (*piece > 0) ^ (multiplier > 0) {
                enemy_pawns[abs_piece - 1] += multiplier;
              } else {
                friendly_pawns[abs_piece - 1] += multiplier;
              }
            }
          }
          // bonus for advanced pawn
          let squares_to_go = if piece > 0 { height - 1 - i } else { i } as u8;
          if squares_to_go != 0 {
            pawn_list.push((squares_to_go, multiplier));
          }
        }
      }
    }
  }
  material = min(material, ENDGAME_THRESHOLD);
  Features {
    material,
    pieces: piece_counts,
    indexes,
    friendly_pawns,
    enemy_pawns,
    mobility,
    attacked_by_pawn,
    defended_by_pawn,
    pawn_list,
  }
}

/// Returns the static evaluation of the provided position
#[must_use]
pub fn evaluate(state: &State, board: &Board) -> i32 {
  #[cfg(not(feature = "feature_extraction"))]
  let score = raw(
    board.board(),
    board.to_move(),
    state.promotion_values,
    &state.packed_parameters,
  );
  #[cfg(feature = "feature_extraction")]
  let features = extract_features(board.board());
  #[cfg(feature = "feature_extraction")]
  let score = eval_features(
    &features,
    board.to_move(),
    state.promotion_values,
    &state.parameters,
  );
  score
}

pub(crate) fn evaluate_terminal(board: &Board) -> Score {
  match board.state() {
    Gamestate::InProgress
    | Gamestate::Material
    | Gamestate::FiftyMove
    | Gamestate::Repetition
    | Gamestate::Stalemate => DRAW_SCORE,
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(board.moves()),
  }
}
