use crate::evaluate::{evaluate, evaluate_terminal};
use crate::tt::{Entry, ScoreType};
use crate::{print_info, Output, SearchConfig, StackEntry, State, DRAW_SCORE};
use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use std::cmp::max;
use std::ops::{Add, Mul, Sub};
use ulci::Score;

/// The default parameters for the search
pub const SEARCH_PARAMETERS: SearchParameters = SearchParameters {
  lmr_base: 0.42826194,
  lmr_factor: 0.36211678,
  lmr_pv_reduction: 0.6459082,
  lmr_improving_reduction: 0.5,
};

/// Parameters affecting the behaviour of the search
///
/// Currently a subset of the values as these are the only ones that have been tuned
#[derive(Copy, Clone, Debug)]
pub struct SearchParameters {
  /// Base LMR reduction amount
  pub lmr_base: f32,
  /// LMR reduction scaling factor
  pub lmr_factor: f32,
  /// How much to reduce LMR by in pv nodes
  pub lmr_pv_reduction: f32,
  /// How much to increase LMR by when not improving
  pub lmr_improving_reduction: f32,
}

impl Add for SearchParameters {
  type Output = Self;

  fn add(self, rhs: Self) -> Self {
    Self {
      lmr_base: self.lmr_base + rhs.lmr_base,
      lmr_factor: self.lmr_factor + rhs.lmr_factor,
      lmr_pv_reduction: self.lmr_pv_reduction + rhs.lmr_pv_reduction,
      lmr_improving_reduction: self.lmr_improving_reduction + rhs.lmr_improving_reduction,
    }
  }
}

impl Sub for SearchParameters {
  type Output = Self;

  fn sub(self, rhs: Self) -> Self {
    Self {
      lmr_base: self.lmr_base - rhs.lmr_base,
      lmr_factor: self.lmr_factor - rhs.lmr_factor,
      lmr_pv_reduction: self.lmr_pv_reduction - rhs.lmr_pv_reduction,
      lmr_improving_reduction: self.lmr_improving_reduction - rhs.lmr_improving_reduction,
    }
  }
}

impl Mul<f32> for SearchParameters {
  type Output = Self;

  fn mul(self, rhs: f32) -> Self {
    Self {
      lmr_base: self.lmr_base * rhs,
      lmr_factor: self.lmr_factor * rhs,
      lmr_pv_reduction: self.lmr_pv_reduction * rhs,
      lmr_improving_reduction: self.lmr_improving_reduction * rhs,
    }
  }
}

// Run a quiescence search of the given position that only considers recaptures
fn recaptures(
  state: &mut State,
  settings: &mut SearchConfig,
  ply: usize,
  mut alpha: Score,
  beta: Score,
  target: (usize, usize),
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, ply);
  let board = &state.stack[ply].board;
  if board.state() == Gamestate::InProgress {
    let mut best_score = Score::Centipawn(evaluate(state, board));
    if best_score >= beta {
      return (Vec::new(), best_score);
    }
    if best_score > alpha {
      alpha = best_score;
    }
    let mut best_pv = Vec::new();
    let mut moves = board.generate_recaptures(target);
    moves.sort_by_key(|(_, piece)| state.parameters.pieces[usize::from(*piece - 1)].0);
    while state.stack.len() <= ply + 1 {
      state
        .stack
        .push(StackEntry::new(state.stack[ply].board.clone()));
    }
    for (mv, _) in moves {
      // Safety - the indices are different therefore the references don't alias
      let position = unsafe {
        let board = &*(&state.stack[ply].board as *const Board);
        let position = &mut state.stack[ply + 1].board;
        position.clone_from(board);
        position
      };
      if position.make_pseudolegal_move(mv) {
        settings.nodes += 1;
        let (mut pv, mut score) = recaptures(state, settings, ply + 1, -beta, -alpha, target);
        score = -score;
        if score >= beta {
          return (Vec::new(), score);
        }
        if score > best_score {
          best_score = score;
          let mut new_pv = vec![mv];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        break;
      }
    }
    (best_pv, best_score)
  } else {
    (Vec::new(), evaluate_terminal(board))
  }
}

/// Run a quiescence search of the given position
pub fn quiescence(
  state: &mut State,
  settings: &mut SearchConfig,
  ply: usize,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> Option<(Vec<Move>, Score)> {
  settings.seldepth = max(settings.seldepth, ply);
  let board = &state.stack[ply].board;
  if board.state() == Gamestate::InProgress {
    let hash = board.hash();
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, 0);
    if let Some(score) = score {
      let mut pv = Vec::new();
      if let Some(bestmove) = ttmove {
        pv.push(bestmove);
      }
      return Some((pv, score));
    }
    if depth == 0 {
      return Some(if let Some(last_move) = board.last_move {
        recaptures(state, settings, ply, alpha, beta, last_move.end())
      } else {
        (Vec::new(), Score::Centipawn(evaluate(state, board)))
      });
    }
    let mut best_score = Score::Centipawn(evaluate(state, board));
    if best_score >= beta {
      return Some((Vec::new(), best_score));
    }
    if best_score > alpha {
      alpha = best_score;
    }
    let mut best_pv = Vec::new();
    if settings.search_is_over() {
      return None;
    }
    let mut moves = board.generate_qsearch();
    moves.sort_by_key(|(_, piece, capture)| {
      state.parameters.pieces[usize::from(*piece - 1)].0
        - 100 * state.parameters.pieces[usize::from(*capture - 1)].0
    });
    while state.stack.len() <= ply + 1 {
      state
        .stack
        .push(StackEntry::new(state.stack[ply].board.clone()));
    }
    for (mv, _, _) in moves {
      // Safety - the indices are different therefore the references don't alias
      let position = unsafe {
        let board = &*(&state.stack[ply].board as *const Board);
        let position = &mut state.stack[ply + 1].board;
        position.clone_from(board);
        position
      };
      if position.make_pseudolegal_move(mv) {
        settings.nodes += 1;
        let (mut pv, mut score) = quiescence(state, settings, ply + 1, depth - 1, -beta, -alpha)?;
        score = -score;
        if score >= beta {
          return Some((Vec::new(), score));
        }
        if score > best_score {
          best_score = score;
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![mv];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
      }
    }
    Some((best_pv, best_score))
  } else {
    Some((Vec::new(), evaluate_terminal(board)))
  }
}

fn alpha_beta(
  state: &mut State,
  settings: &mut SearchConfig,
  ply: usize,
  mut depth: u8,
  mut alpha: Score,
  beta: Score,
  pv_node: bool,
  // not allowed to nullmove if previous nullmove
  nullmove: bool,
) -> Option<(Vec<Move>, Score)> {
  settings.seldepth = max(settings.seldepth, ply);
  let board = &state.stack[ply].board;
  if let Score::Win(movecount) = alpha {
    let moves = board.moves();
    if moves >= movecount {
      // Mate distance pruning
      return Some((Vec::new(), alpha));
    }
  }
  let in_check = board.in_check();
  if in_check {
    depth += 1;
  }
  if board.state() != Gamestate::InProgress {
    Some((Vec::new(), evaluate_terminal(board)))
  } else if depth == 0 {
    let (pv, score) = quiescence(state, settings, ply, 1, alpha, beta)?;
    let tt_flag = if score >= beta {
      ScoreType::LowerBound
    } else if score > alpha {
      ScoreType::Exact
    } else {
      ScoreType::UpperBound
    };
    let board = &state.stack[ply].board;
    state.table.store(Entry {
      hash: board.hash(),
      depth: 0,
      movecount: board.moves(),
      scoretype: tt_flag,
      score,
      bestmove: None,
    });
    Some((pv, score))
  } else {
    let hash = board.hash();
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, depth);

    if !pv_node {
      if let Some(score) = score {
        return Some((Vec::new(), score));
      }
    }

    let mut futility_score = None;
    let movecount = board.moves();

    let eval = evaluate(state, board);

    while state.stack.len() <= ply + 1 {
      state
        .stack
        .push(StackEntry::new(state.stack[ply].board.clone()));
    }

    state.stack[ply].eval = if in_check { None } else { Some(eval) };
    let improving = if in_check {
      false
    } else if ply < 2 {
      true
    } else if let Some(old_eval) = state.stack[ply - 2].eval {
      eval > old_eval
    } else {
      true
    };

    if !pv_node && !in_check {
      // Reverse futility pruning
      if depth <= 8 {
        if let Score::Centipawn(beta_cp) = beta {
          let mut depth = i32::from(depth);
          if improving {
            depth -= 1;
          }
          let rfp_margin = 120 * depth;
          let rfp_beta = beta_cp + rfp_margin;
          if eval >= rfp_beta {
            let score = Score::Centipawn(eval - rfp_margin);
            return Some((Vec::new(), score));
          }
        }
      }

      let board = &state.stack[ply].board;

      // Null move pruning
      if !nullmove && depth >= 2 && Score::Centipawn(eval) >= beta && board.has_pieces() {
        if let Some(nullmove) = board.nullmove() {
          let null_depth = depth.saturating_sub(3 + depth / 5);
          state.stack[ply + 1].board = nullmove;
          let score = -null_move_search(state, settings, ply + 1, null_depth, -beta)?;
          if score >= beta {
            let score = match score {
              Score::Centipawn(_) => score,
              _ => beta,
            };
            // Verification search
            if null_depth > 0 {
              let verif_score = zero_window_search(state, settings, ply, null_depth, beta, true)?;
              if verif_score >= beta {
                return Some((Vec::new(), score));
              }
            } else {
              return Some((Vec::new(), score));
            }
          }
        }
      }

      if depth <= 4 {
        if let Score::Centipawn(alpha_cp) = alpha {
          let futility_margin = 125 * i32::from(depth);
          let futility_threshold = alpha_cp - futility_margin;
          if eval < futility_threshold {
            futility_score = Some(Score::Centipawn(eval + futility_margin));
          }
        }
      }
    }

    if settings.search_is_over() {
      return None;
    }

    let mut best_pv = Vec::new();
    let mut best_score = Score::Loss(0);
    let mut move_count = 0;
    let mut fail_lows: Vec<Move> = Vec::new();
    state.stack[ply].movepicker.init(ttmove);
    while let Some((mv, is_capture)) = state.stack[ply].pick_move(&state.history, &state.parameters)
    {
      // Move loop pruning for quiets - we need to avoid mate first
      if !is_capture && !matches!(best_score, Score::Loss(_)) {
        if let Some(futility_score) = futility_score {
          best_score = max(best_score, futility_score);
          break;
        }

        // Late move pruning
        if depth <= 2 && move_count >= (5 << depth) {
          break;
        }
      }
      // Safety - the indices are different therefore the references don't alias
      let position = unsafe {
        let board = &*(&state.stack[ply].board as *const Board);
        let position = &mut state.stack[ply + 1].board;
        position.clone_from(board);
        position
      };
      if position.make_pseudolegal_move(mv) {
        settings.nodes += 1;
        move_count += 1;
        // Late move reductions
        let reduction = if !is_capture && depth >= 3 && move_count > 5 && !position.in_check() {
          let mut reduction = state.search_parameters.lmr_base
            + f32::from(depth).ln() * (move_count as f32).ln() * state.search_parameters.lmr_factor;
          if pv_node {
            reduction -= state.search_parameters.lmr_pv_reduction;
          }
          if !improving {
            reduction += state.search_parameters.lmr_improving_reduction;
          }
          // avoid dropping into qsearch
          (reduction as i8).clamp(0, (depth / 2) as i8) as u8
        } else {
          0
        };
        let (mut pv, score) = if (pv_node && move_count > 1) || reduction > 0 {
          // Zero window search to see if raises alpha
          let score = -zero_window_search(
            state,
            settings,
            ply + 1,
            depth - 1 - reduction,
            -alpha,
            nullmove,
          )?;
          if score > alpha {
            let (pv, score) = alpha_beta(
              state,
              settings,
              ply + 1,
              depth - 1,
              -beta,
              -alpha,
              pv_node,
              nullmove,
            )?;
            (pv, -score)
          } else {
            (Vec::new(), score)
          }
        } else {
          let (pv, score) = alpha_beta(
            state,
            settings,
            ply + 1,
            depth - 1,
            -beta,
            -alpha,
            pv_node,
            nullmove,
          )?;
          (pv, -score)
        };
        if score >= beta {
          if !is_capture {
            state.stack[ply].movepicker.store_killer(mv);
            let board = &state.stack[ply].board;
            for fail_low in fail_lows {
              state.history.malus(
                board.to_move(),
                board.get_piece(fail_low.start()).unsigned_abs(),
                fail_low.end(),
                depth,
              );
            }
            state.history.bonus(
              board.to_move(),
              board.get_piece(mv.start()).unsigned_abs(),
              mv.end(),
              depth,
            );
            if let Some(last_move) = board.last_move {
              let piece = board.get_piece(last_move.end()).unsigned_abs();
              state
                .history
                .store_countermove(board.to_move(), piece, last_move.end(), mv);
            }
          }
          state.table.store(Entry {
            hash,
            depth,
            movecount,
            scoretype: ScoreType::LowerBound,
            score,
            bestmove: Some(mv),
          });
          return Some((Vec::new(), score));
        }
        if score > best_score {
          best_score = score;
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![mv];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        } else if !is_capture {
          fail_lows.push(mv);
        }
      }
    }
    Some(if move_count == 0 {
      (
        Vec::new(),
        if in_check {
          // Checkmate
          Score::Loss(movecount)
        } else {
          // Stalemate
          DRAW_SCORE
        },
      )
    } else {
      let (scoretype, bestmove) = if best_pv.is_empty() {
        (ScoreType::UpperBound, ttmove)
      } else {
        (ScoreType::Exact, best_pv.first().copied())
      };
      state.table.store(Entry {
        hash,
        depth,
        movecount,
        scoretype,
        score: best_score,
        bestmove,
      });
      (best_pv, best_score)
    })
  }
}

fn null_move_search(
  state: &mut State,
  settings: &mut SearchConfig,
  ply: usize,
  depth: u8,
  alpha: Score,
) -> Option<Score> {
  let beta = match alpha {
    Score::Centipawn(cp) => Score::Centipawn(cp + 1),
    Score::Win(moves) => Score::Win(moves - 1),
    Score::Loss(moves) => Score::Loss(moves + 1),
  };
  let (_, score) = alpha_beta(state, settings, ply, depth, alpha, beta, false, true)?;
  Some(score)
}

fn zero_window_search(
  state: &mut State,
  settings: &mut SearchConfig,
  ply: usize,
  depth: u8,
  beta: Score,
  nullmove: bool,
) -> Option<Score> {
  let alpha = match beta {
    Score::Centipawn(cp) => Score::Centipawn(cp - 1),
    Score::Win(moves) => Score::Win(moves + 1),
    Score::Loss(moves) => Score::Loss(moves - 1),
  };
  let (_, score) = alpha_beta(state, settings, ply, depth, alpha, beta, false, nullmove)?;
  Some(score)
}

pub(crate) fn alpha_beta_root(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  captures: &[Move],
  quiets: &mut Vec<Move>,
  ttstore: bool,
  best_moves: &[Move],
  excluded_moves: &[Move],
  depth: u8,
  pv_line: u16,
  show_pv_line: bool,
  out: &mut Output,
) -> (Vec<Move>, Score) {
  let mut alpha = settings.initial_alpha;
  let beta = Score::Win(0);
  let mut best_pv = Vec::new();
  let mut backup_pv = Vec::new();
  let mut move_count = 0;
  let mut show_output = false;
  while state.stack.len() <= 1 {
    state.stack.push(StackEntry::new(board.clone()));
  }
  state.stack[0].eval = if board.in_check() {
    None
  } else {
    Some(evaluate(state, board))
  };
  for best_move in best_moves {
    if !excluded_moves.contains(best_move) {
      if let Some(position) = board.move_if_legal(*best_move) {
        let node_count = settings.nodes;
        settings.nodes += 1;
        move_count += 1;
        state.stack[1].board = position;
        let mut failed_high = false;
        let (mut pv, score) = if move_count > 1 {
          // Zero window search to see if raises alpha
          let score = zero_window_search(state, settings, 1, depth - 1, -alpha, false);
          if let Some(mut score) = score {
            score = -score;
            if score > alpha {
              failed_high = true;
              backup_pv = best_pv;
              best_pv = vec![*best_move];
              if show_output {
                print_info(
                  out,
                  board,
                  alpha,
                  depth,
                  settings,
                  &best_pv,
                  pv_line,
                  show_pv_line,
                  state.table.capacity(),
                );
              }
              if let Some((pv, score)) =
                alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
              {
                (pv, -score)
              } else {
                return (best_pv, alpha);
              }
            } else {
              (Vec::new(), score)
            }
          } else {
            return (best_pv, alpha);
          }
        } else if let Some((pv, score)) =
          alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
        {
          if settings.millis >= 100 {
            show_output = true;
          }
          (pv, -score)
        } else {
          return (best_pv, alpha);
        };
        if score > alpha {
          settings.best_move_nodes += settings.nodes - node_count;
          alpha = score;
          let mut new_pv = vec![*best_move];
          new_pv.append(&mut pv);
          best_pv = new_pv;
          backup_pv.clone_from(&best_pv);
          if show_output {
            print_info(
              out,
              board,
              alpha,
              depth,
              settings,
              &best_pv,
              pv_line,
              show_pv_line,
              state.table.capacity(),
            );
          }
        } else if failed_high {
          // In case of PVS research fail-low, revert best pv
          best_pv.clone_from(&backup_pv);
          if show_output {
            print_info(
              out,
              board,
              alpha,
              depth,
              settings,
              &best_pv,
              pv_line,
              show_pv_line,
              state.table.capacity(),
            );
          }
        }
      }
    }
  }
  for capture in captures {
    if !best_moves.contains(capture) && !excluded_moves.contains(capture) {
      let mut position = board.clone();
      position.play_move(*capture);
      let node_count = settings.nodes;
      settings.nodes += 1;
      move_count += 1;
      state.stack[1].board = position;
      let mut failed_high = false;
      let (mut pv, score) = if move_count > 1 {
        // Zero window search to see if raises alpha
        let score = zero_window_search(state, settings, 1, depth - 1, -alpha, false);
        if let Some(mut score) = score {
          score = -score;
          if score > alpha {
            failed_high = true;
            backup_pv = best_pv;
            best_pv = vec![*capture];
            if show_output {
              print_info(
                out,
                board,
                alpha,
                depth,
                settings,
                &best_pv,
                pv_line,
                show_pv_line,
                state.table.capacity(),
              );
            }
            if let Some((pv, score)) =
              alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
            {
              (pv, -score)
            } else {
              return (best_pv, alpha);
            }
          } else {
            (Vec::new(), score)
          }
        } else {
          return (best_pv, alpha);
        }
      } else if let Some((pv, score)) =
        alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
      {
        if settings.millis >= 100 {
          show_output = true;
        }
        (pv, -score)
      } else {
        return (best_pv, alpha);
      };
      if score > alpha {
        let nodes_taken = settings.nodes - node_count;
        if move_count == 1 {
          settings.best_move_nodes += nodes_taken;
        } else {
          settings.best_move_nodes = nodes_taken;
        }
        alpha = score;
        let mut new_pv = vec![*capture];
        new_pv.append(&mut pv);
        best_pv = new_pv;
        backup_pv.clone_from(&best_pv);
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            pv_line,
            show_pv_line,
            state.table.capacity(),
          );
        }
      } else if failed_high {
        // In case of PVS research fail-low, revert best pv
        best_pv.clone_from(&backup_pv);
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            pv_line,
            show_pv_line,
            state.table.capacity(),
          );
        }
      }
    }
  }
  quiets.sort_by_key(|mv| {
    -state.history.get(
      board.to_move(),
      board.get_piece(mv.start()).unsigned_abs(),
      mv.end(),
    )
  });
  for quiet in quiets {
    if !best_moves.contains(quiet) && !excluded_moves.contains(quiet) {
      let mut position = board.clone();
      position.play_move(*quiet);
      let node_count = settings.nodes;
      settings.nodes += 1;
      move_count += 1;
      // Late move reductions
      let reduction = if depth >= 3 && move_count > 5 && !position.in_check() {
        let reduction = state.search_parameters.lmr_base
          + f32::from(depth).ln() * (move_count as f32).ln() * state.search_parameters.lmr_factor
          - state.search_parameters.lmr_pv_reduction;
        // avoid dropping into qsearch
        (reduction as i8).clamp(0, (depth / 2) as i8) as u8
      } else {
        0
      };
      state.stack[1].board = position;
      let mut failed_high = false;
      let (mut pv, score) = if move_count > 1 {
        // Zero window search to see if raises alpha
        let score = zero_window_search(state, settings, 1, depth - 1 - reduction, -alpha, false);
        if let Some(mut score) = score {
          score = -score;
          if score > alpha {
            failed_high = true;
            backup_pv = best_pv;
            best_pv = vec![*quiet];
            if show_output {
              print_info(
                out,
                board,
                alpha,
                depth,
                settings,
                &best_pv,
                pv_line,
                show_pv_line,
                state.table.capacity(),
              );
            }
            if let Some((pv, score)) =
              alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
            {
              (pv, -score)
            } else {
              return (best_pv, alpha);
            }
          } else {
            (Vec::new(), score)
          }
        } else {
          return (best_pv, alpha);
        }
      } else if let Some((pv, score)) =
        alpha_beta(state, settings, 1, depth - 1, -beta, -alpha, true, false)
      {
        if settings.millis >= 100 {
          show_output = true;
        }
        (pv, -score)
      } else {
        return (best_pv, alpha);
      };
      if score > alpha {
        let nodes_taken = settings.nodes - node_count;
        if move_count == 1 {
          settings.best_move_nodes += nodes_taken;
        } else {
          settings.best_move_nodes = nodes_taken;
        }
        alpha = score;
        let mut new_pv = vec![*quiet];
        new_pv.append(&mut pv);
        best_pv = new_pv;
        backup_pv.clone_from(&best_pv);
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            pv_line,
            show_pv_line,
            state.table.capacity(),
          );
        }
      } else if failed_high {
        // In case of PVS research fail-low, revert best pv
        best_pv.clone_from(&backup_pv);
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            pv_line,
            show_pv_line,
            state.table.capacity(),
          );
        }
      }
    }
  }
  if move_count == 0 {
    (
      Vec::new(),
      if board.in_check() {
        // Checkmate
        Score::Loss(board.moves())
      } else {
        // Stalemate
        DRAW_SCORE
      },
    )
  } else {
    let (scoretype, bestmove) = if best_pv.is_empty() {
      (ScoreType::UpperBound, best_moves.first().copied())
    } else {
      (ScoreType::Exact, best_pv.first().copied())
    };
    if ttstore {
      state.table.store(Entry {
        hash: board.hash(),
        depth,
        movecount: board.moves(),
        scoretype,
        score: alpha,
        bestmove,
      });
    }
    (best_pv, alpha)
  }
}
