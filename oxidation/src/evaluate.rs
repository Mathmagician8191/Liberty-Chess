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

#[cfg(feature = "pesto")]
use crate::pesto::{EG_PSQT, MG_PSQT};

/// Extracted evaluation features
#[derive(Clone)]
pub struct Features {
  material: i32,
  pieces: [i8; 18],
  indexes: [[i8; EDGE_PARAMETER_COUNT]; 18],
  friendly_pawns: [i8; 18],
  enemy_pawns: [i8; 18],
  // squares to go and multiplier
  pawn_list: Vec<(u8, i8)>,
}

/// Returns the static evaluation from the provided raw data
#[must_use]
pub fn raw(
  pieces: &Array2D<Piece>,
  to_move: bool,
  #[cfg(not(feature = "pesto"))] promotion_values: (i32, i32),
  parameters: &Parameters<i32>,
) -> i32 {
  let mut middlegame = 0;
  let mut endgame = 0;
  let mut material = 0;
  let height = pieces.num_rows();
  let width = pieces.num_columns();
  for i in 0..height {
    for j in 0..width {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let multiplier = if piece > 0 { 1 } else { -1 };
        let piece_type = piece.unsigned_abs() as usize - 1;
        material += ENDGAME_FACTOR[piece_type];
        let (mut mg_value, mut eg_value) = parameters.pieces[piece_type];
        #[cfg(feature = "pesto")]
        {
          if height == 8 && width == 8 && piece_type < 6 {
            let index = if piece > 0 { 7 - i } else { i };
            mg_value += MG_PSQT[piece_type][index][j];
            eg_value += EG_PSQT[piece_type][index][j];
          } else {
            let horizontal_distance = min(i, height - 1 - i).min(EDGE_DISTANCE);
            let vertical_distance = min(j, width - 1 - j).min(EDGE_DISTANCE);
            let index = INDEXING[horizontal_distance * (EDGE_DISTANCE + 1) + vertical_distance];
            if index < EDGE_PARAMETER_COUNT {
              mg_value -= parameters.mg_edge[piece_type][index];
              eg_value -= parameters.eg_edge[piece_type][index];
            }
          }
        }
        #[cfg(not(feature = "pesto"))]
        {
          let horizontal_distance = min(i, height - 1 - i).min(EDGE_DISTANCE);
          let vertical_distance = min(j, width - 1 - j).min(EDGE_DISTANCE);
          let index = INDEXING[horizontal_distance * (EDGE_DISTANCE + 1) + vertical_distance];
          if index < EDGE_PARAMETER_COUNT {
            mg_value -= parameters.mg_edge[piece_type][index];
            eg_value -= parameters.eg_edge[piece_type][index];
          }
        }
        if piece.abs() == PAWN {
          // penalty for pawn being blocked
          let block_i = if piece > 0 { i + 1 } else { i - 1 };
          if let Some(piece) = pieces.get(block_i, j) {
            if *piece != 0 {
              let abs_piece = usize::from(piece.unsigned_abs());
              if (*piece > 0) ^ (multiplier > 0) {
                mg_value -= parameters.mg_enemy_pawn_penalty[abs_piece - 1];
                eg_value -= parameters.eg_enemy_pawn_penalty[abs_piece - 1];
              } else {
                mg_value -= parameters.mg_friendly_pawn_penalty[abs_piece - 1];
                eg_value -= parameters.eg_friendly_pawn_penalty[abs_piece - 1];
              }
            }
          }
          // bonus for advanced pawn
          #[cfg(not(feature = "pesto"))]
          {
            let squares_to_go = if piece > 0 { height - 1 - i } else { i } as i32;
            if squares_to_go != 0 {
              mg_value += promotion_values.0
                / (squares_to_go * parameters.pawn_scale_factor + parameters.pawn_scaling_bonus);
              eg_value += promotion_values.1
                / (squares_to_go * parameters.pawn_scale_factor + parameters.pawn_scaling_bonus);
            }
          }
        }
        middlegame += mg_value * multiplier;
        endgame += eg_value * multiplier;
      }
    }
  }
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
    let mg_edge = parameters.mg_edge[piece_type];
    let eg_edge = parameters.eg_edge[piece_type];
    let piece_count = features.indexes[piece_type];
    for index in 0..5 {
      let count = T::from(piece_count[index]);
      middlegame -= mg_edge[index] * count;
      endgame -= eg_edge[index] * count;
    }
  }
  for (squares_to_go, multiplier) in &features.pawn_list {
    let multiplier = T::from(*multiplier);
    let division_factor =
      T::from(*squares_to_go) * parameters.pawn_scale_factor + parameters.pawn_scaling_bonus;
    middlegame += promotion_values.0 / division_factor * multiplier;
    endgame += promotion_values.1 / division_factor * multiplier;
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
  features.indexes[OBSTACLE as usize - 1] = [0, 0, 0, 0, 0];
  features.indexes[WALL as usize - 1] = [0, 0, 0, 0, 0];
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
  let mut pawn_scale_factor = 0.0;
  let mut pawn_scaling_bonus = 0.0;
  let piece_value = promotion_values.0 * mg_factor + promotion_values.1 * eg_factor;
  for (squares, count) in &features.pawn_list {
    let squares = f64::from(*squares);
    let divisor = squares.mul_add(parameters.pawn_scale_factor, parameters.pawn_scaling_bonus);
    let scaling_factor = -piece_value * f64::from(*count) / divisor.powi(2);
    pawn_scale_factor += scaling_factor * squares;
    pawn_scaling_bonus += scaling_factor;
  }
  Parameters {
    pieces,
    mg_edge,
    eg_edge,
    mg_friendly_pawn_penalty,
    eg_friendly_pawn_penalty,
    mg_enemy_pawn_penalty,
    eg_enemy_pawn_penalty,
    pawn_scale_factor,
    pawn_scaling_bonus,
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
  let mut pawn_list = Vec::new();
  let height = pieces.num_rows();
  let width = pieces.num_columns();
  for i in 0..height {
    for j in 0..width {
      let piece = pieces[(i, j)];
      if piece != 0 {
        let multiplier = if piece > 0 { 1 } else { -1 };
        let piece_type = piece.unsigned_abs() as usize - 1;
        material += ENDGAME_FACTOR[piece_type];
        piece_counts[piece_type] += multiplier;
        let horizontal_distance = min(i, height - 1 - i).min(EDGE_DISTANCE);
        let vertical_distance = min(j, width - 1 - j).min(EDGE_DISTANCE);
        let index = INDEXING[horizontal_distance * (EDGE_DISTANCE + 1) + vertical_distance];
        if index < EDGE_PARAMETER_COUNT {
          indexes[piece_type][index] += multiplier;
        }
        if piece.abs() == PAWN {
          // penalty for pawn being blocked
          let block_i = if piece > 0 { i + 1 } else { i - 1 };
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
    pawn_list,
  }
}

/// Returns the static evaluation of the provided position
#[must_use]
pub fn evaluate(state: &State, board: &Board) -> Score {
  match board.state() {
    #[cfg(not(feature = "feature_extraction"))]
    Gamestate::InProgress => {
      let score = raw(
        board.board(),
        board.to_move(),
        #[cfg(not(feature = "pesto"))]
        state.promotion_values,
        &state.parameters,
      );
      Score::Centipawn(score)
    }
    #[cfg(feature = "feature_extraction")]
    Gamestate::InProgress => {
      let features = extract_features(board.board());
      let score = eval_features(
        &features,
        board.to_move(),
        state.promotion_values,
        &state.parameters,
      );
      Score::Centipawn(score)
    }
    Gamestate::Material | Gamestate::FiftyMove | Gamestate::Repetition | Gamestate::Stalemate => {
      DRAW_SCORE
    }
    Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Score::Loss(board.moves()),
  }
}
