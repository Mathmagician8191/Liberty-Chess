use crate::evaluate::evaluate;
use crate::tt::{Entry, ScoreType};
use crate::{print_info, Output, SearchConfig, State, DRAW_SCORE};
use liberty_chess::moves::Move;
use liberty_chess::{Board, Gamestate};
use std::cmp::max;
use ulci::Score;

// Run a quiescence search of the given position that only considers recaptures
fn recaptures(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  mut alpha: Score,
  beta: Score,
  target: (usize, usize),
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, board.ply_count());
  if board.state() == Gamestate::InProgress {
    let mut best_score = evaluate(state, board);
    if best_score >= beta {
      return (Vec::new(), best_score);
    }
    if best_score > alpha {
      alpha = best_score;
    }
    let mut best_pv = Vec::new();
    let mut moves = board.generate_recaptures(target);
    moves.sort_by_key(|(_, piece)| state.parameters.pieces[usize::from(*piece - 1)].0);
    for (bestmove, _) in moves {
      if let Some(position) = board.test_move_legality(bestmove) {
        settings.nodes += 1;
        let (mut pv, mut score) = recaptures(state, settings, &position, -beta, -alpha, target);
        score = -score;
        if score >= beta {
          return (Vec::new(), score);
        }
        if score > best_score {
          best_score = score;
          let mut new_pv = vec![bestmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        break;
      }
    }
    (best_pv, best_score)
  } else {
    (Vec::new(), evaluate(state, board))
  }
}

/// Run a quiescence search of the given position
pub fn quiescence(
  state: &State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  mut alpha: Score,
  beta: Score,
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, board.ply_count());
  if board.state() == Gamestate::InProgress {
    let hash = board.hash();
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, 0);
    if let Some(score) = score {
      let mut pv = Vec::new();
      if let Some(bestmove) = ttmove {
        pv.push(bestmove);
      }
      return (pv, score);
    }
    if depth == 0 {
      return if let Some(last_move) = board.last_move {
        recaptures(state, settings, board, alpha, beta, last_move.end())
      } else {
        (Vec::new(), evaluate(state, board))
      };
    }
    let mut best_score = evaluate(state, board);
    if best_score >= beta {
      return (Vec::new(), best_score);
    }
    if best_score > alpha {
      alpha = best_score;
    }
    let mut best_pv = Vec::new();
    if !settings.search_is_over() {
      let mut moves = board.generate_qsearch();
      moves.sort_by_key(|(_, piece, capture)| {
        state.parameters.pieces[usize::from(*piece - 1)].0
          - 100 * state.parameters.pieces[usize::from(*capture - 1)].0
      });
      for (bestmove, _, _) in moves {
        if let Some(position) = board.test_move_legality(bestmove) {
          settings.nodes += 1;
          let (mut pv, mut score) =
            quiescence(state, settings, &position, depth - 1, -beta, -alpha);
          score = -score;
          if score >= beta {
            return (Vec::new(), score);
          }
          if score > best_score {
            best_score = score;
          }
          if score > alpha {
            alpha = score;
            let mut new_pv = vec![bestmove];
            new_pv.append(&mut pv);
            best_pv = new_pv;
          }
          if settings.search_is_over() {
            return (best_pv, best_score);
          }
        }
      }
    }
    (best_pv, best_score)
  } else {
    (Vec::new(), evaluate(state, board))
  }
}

fn alpha_beta(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  mut depth: u8,
  mut alpha: Score,
  beta: Score,
  pv_node: bool,
  // not allowed to nullmove if previous nullmove
  nullmove: bool,
) -> (Vec<Move>, Score) {
  settings.seldepth = max(settings.seldepth, board.ply_count());
  if let Score::Win(movecount) = alpha {
    let moves = board.moves();
    if moves >= movecount {
      // Mate distance pruning
      return (Vec::new(), alpha);
    }
  }
  let in_check = board.in_check();
  if in_check {
    depth += 1;
  }
  if board.state() != Gamestate::InProgress {
    (Vec::new(), evaluate(state, board))
  } else if depth == 0 {
    let (pv, score) = quiescence(state, settings, board, *settings.qdepth, alpha, beta);
    if !settings.search_is_over() {
      let tt_flag = if score >= beta {
        ScoreType::LowerBound
      } else if score > alpha {
        ScoreType::Exact
      } else {
        ScoreType::UpperBound
      };
      state.table.store(Entry {
        hash: board.hash(),
        depth: 0,
        movecount: board.moves(),
        scoretype: tt_flag,
        score,
        bestmove: pv.first().copied(),
      });
    }
    (pv, score)
  } else {
    let hash = board.hash();
    let (score, ttmove) = state.table.get(hash, board.moves(), alpha, beta, depth);

    if !pv_node {
      if let Some(score) = score {
        return (Vec::new(), score);
      }
    }

    if !pv_node && !in_check {
      let evaluation = evaluate(state, board);

      // Reverse futility pruning
      if let Score::Centipawn(beta_cp) = beta {
        let depth = i32::from(depth);
        let rfp_margin = 50 * depth * depth;
        let rfp_beta = Score::Centipawn(beta_cp + rfp_margin);
        if evaluation >= rfp_beta {
          let score = match evaluation {
            Score::Centipawn(score) => Score::Centipawn(score - rfp_margin),
            _ => beta,
          };
          return (Vec::new(), score);
        }
      }

      // Null move pruning
      if !nullmove && depth >= 3 && evaluation >= beta && board.has_pieces() {
        if let Some(nullmove) = board.nullmove() {
          let score = -null_move_search(state, settings, &nullmove, depth - 3, -beta);
          if score >= beta {
            // Verification search
            if depth >= 4 {
              let score = zero_window_search(state, settings, board, depth - 3, beta, true);
              if score >= beta {
                return (Vec::new(), score);
              }
            } else {
              let score = match score {
                Score::Centipawn(_) => score,
                _ => beta,
              };
              return (Vec::new(), score);
            }
          }
        }
      }
    }

    let mut best_pv = Vec::new();
    let mut best_score = Score::Loss(0);
    let mut move_count = 0;
    // Handle TTmove
    if let Some(ttmove) = ttmove {
      if let Some(position) = board.move_if_legal(ttmove) {
        settings.nodes += 1;
        move_count += 1;
        let (mut pv, mut score) = alpha_beta(
          state,
          settings,
          &position,
          depth - 1,
          -beta,
          -alpha,
          pv_node,
          nullmove,
        );
        score = -score;
        if score >= beta {
          let capture = board.get_piece(ttmove.end());
          if capture == 0 || ((capture > 0) == board.to_move()) {
            state.history.store(
              board.to_move(),
              board.get_piece(ttmove.start()).unsigned_abs(),
              ttmove.end(),
              depth,
            );
          }
          state.table.store(Entry {
            hash,
            depth,
            movecount: board.moves(),
            scoretype: ScoreType::LowerBound,
            score,
            bestmove: Some(ttmove),
          });
          return (Vec::new(), score);
        }
        if score > best_score {
          best_score = score;
        }
        if score > alpha {
          alpha = score;
          let mut new_pv = vec![ttmove];
          new_pv.append(&mut pv);
          best_pv = new_pv;
        }
        if settings.search_is_over() {
          return (best_pv, best_score);
        }
      }
    }
    let (mut captures, mut quiets) = board.generate_pseudolegal();
    captures.sort_by_key(|(_, piece, capture)| {
      state.parameters.pieces[usize::from(*piece - 1)].0
        - 100 * state.parameters.pieces[usize::from(*capture - 1)].0
    });
    for (bestmove, _, _) in captures {
      if Some(bestmove) != ttmove {
        if let Some(position) = board.test_move_legality(bestmove) {
          settings.nodes += 1;
          move_count += 1;
          let (mut pv, score) = if pv_node && move_count > 1 {
            // Zero window search to see if raises alpha
            let score =
              -zero_window_search(state, settings, &position, depth - 1, -alpha, nullmove);
            if score > alpha {
              let (pv, score) = alpha_beta(
                state,
                settings,
                &position,
                depth - 1,
                -beta,
                -alpha,
                true,
                nullmove,
              );
              (pv, -score)
            } else {
              (Vec::new(), score)
            }
          } else {
            let (pv, score) = alpha_beta(
              state,
              settings,
              &position,
              depth - 1,
              -beta,
              -alpha,
              pv_node,
              nullmove,
            );
            (pv, -score)
          };
          if score >= beta {
            state.table.store(Entry {
              hash,
              depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score,
              bestmove: Some(bestmove),
            });
            return (Vec::new(), score);
          }
          if score > best_score {
            best_score = score;
          }
          if score > alpha {
            alpha = score;
            let mut new_pv = vec![bestmove];
            new_pv.append(&mut pv);
            best_pv = new_pv;
          }
          if settings.search_is_over() {
            return (best_pv, best_score);
          }
        }
      }
    }
    let mut fail_lows: Vec<Move> = Vec::new();
    let seldepth = (board.ply_count() - state.root_ply_count) as usize;
    while state.killers.len() < seldepth {
      state.killers.push(None);
    }
    if let Some(killer) = state.killers[seldepth - 1] {
      // filter out capturing killers
      let capture = board.get_piece(killer.end());
      if Some(killer) != ttmove && (capture == 0 || ((capture > 0) == board.to_move())) {
        if let Some(position) = board.move_if_legal(killer) {
          settings.nodes += 1;
          move_count += 1;
          // Late move reductions
          let reduction = u8::from(depth >= 3 && move_count > 10 && !position.in_check());
          let (mut pv, score) = if (pv_node && move_count > 1) || reduction > 0 {
            // Zero window search to see if raises alpha
            let score = -zero_window_search(
              state,
              settings,
              &position,
              depth - 1 - reduction,
              -alpha,
              nullmove,
            );
            if score > alpha {
              let (pv, score) = alpha_beta(
                state,
                settings,
                &position,
                depth - 1,
                -beta,
                -alpha,
                pv_node,
                nullmove,
              );
              (pv, -score)
            } else {
              (Vec::new(), score)
            }
          } else {
            let (pv, score) = alpha_beta(
              state,
              settings,
              &position,
              depth - 1,
              -beta,
              -alpha,
              pv_node,
              nullmove,
            );
            (pv, -score)
          };
          if score >= beta {
            state.history.store(
              board.to_move(),
              board.get_piece(killer.start()).unsigned_abs(),
              killer.end(),
              depth,
            );
            state.table.store(Entry {
              hash,
              depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score,
              bestmove: Some(killer),
            });
            return (Vec::new(), score);
          }
          if score > best_score {
            best_score = score;
          }
          if score > alpha {
            alpha = score;
            let mut new_pv = vec![killer];
            new_pv.append(&mut pv);
            best_pv = new_pv;
          } else {
            fail_lows.push(killer);
          }
          if settings.search_is_over() {
            return (best_pv, best_score);
          }
        }
      }
    }
    quiets.sort_by_key(|r#move| {
      -state.history.get(
        board.to_move(),
        board.get_piece(r#move.start()).unsigned_abs(),
        r#move.end(),
      )
    });
    for bestmove in quiets {
      if Some(bestmove) != ttmove && Some(bestmove) != state.killers[seldepth - 1] {
        // Late move pruning
        if !pv_node
          && depth <= 2
          && move_count >= (5 << depth)
          && matches!(alpha, Score::Centipawn(_))
        {
          break;
        }
        if let Some(position) = board.test_move_legality(bestmove) {
          settings.nodes += 1;
          move_count += 1;
          // Late move reductions
          let reduction = u8::from(depth >= 3 && move_count > 10 && !position.in_check());
          let (mut pv, score) = if (pv_node && move_count > 1) || reduction > 0 {
            // Zero window search to see if raises alpha
            let score = -zero_window_search(
              state,
              settings,
              &position,
              depth - 1 - reduction,
              -alpha,
              nullmove,
            );
            if score > alpha {
              let (pv, score) = alpha_beta(
                state,
                settings,
                &position,
                depth - 1,
                -beta,
                -alpha,
                pv_node,
                nullmove,
              );
              (pv, -score)
            } else {
              (Vec::new(), score)
            }
          } else {
            let (pv, score) = alpha_beta(
              state,
              settings,
              &position,
              depth - 1,
              -beta,
              -alpha,
              pv_node,
              nullmove,
            );
            (pv, -score)
          };
          if score >= beta {
            for fail_low in fail_lows {
              state.history.malus(
                board.to_move(),
                board.get_piece(fail_low.start()).unsigned_abs(),
                fail_low.end(),
                depth,
              );
            }
            state.history.store(
              board.to_move(),
              board.get_piece(bestmove.start()).unsigned_abs(),
              bestmove.end(),
              depth,
            );
            state.killers[seldepth - 1] = Some(bestmove);
            state.table.store(Entry {
              hash,
              depth,
              movecount: board.moves(),
              scoretype: ScoreType::LowerBound,
              score,
              bestmove: Some(bestmove),
            });
            return (Vec::new(), score);
          }
          if score > best_score {
            best_score = score;
          }
          if score > alpha {
            alpha = score;
            let mut new_pv = vec![bestmove];
            new_pv.append(&mut pv);
            best_pv = new_pv;
          } else {
            fail_lows.push(bestmove);
          }
          if settings.search_is_over() {
            return (best_pv, best_score);
          }
        }
      }
    }
    if move_count == 0 {
      (
        Vec::new(),
        if in_check {
          // Checkmate
          Score::Loss(board.moves())
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
        movecount: board.moves(),
        scoretype,
        score: best_score,
        bestmove,
      });
      (best_pv, best_score)
    }
  }
}

fn null_move_search(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  alpha: Score,
) -> Score {
  let beta = match alpha {
    Score::Centipawn(cp) => Score::Centipawn(cp + 1),
    Score::Win(moves) => Score::Win(moves - 1),
    Score::Loss(moves) => Score::Loss(moves + 1),
  };
  let (_, score) = alpha_beta(state, settings, board, depth, alpha, beta, false, true);
  score
}

fn zero_window_search(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  depth: u8,
  beta: Score,
  nullmove: bool,
) -> Score {
  let alpha = match beta {
    Score::Centipawn(cp) => Score::Centipawn(cp - 1),
    Score::Win(moves) => Score::Win(moves + 1),
    Score::Loss(moves) => Score::Loss(moves - 1),
  };
  let (_, score) = alpha_beta(state, settings, board, depth, alpha, beta, false, nullmove);
  score
}

pub(crate) fn alpha_beta_root(
  state: &mut State,
  settings: &mut SearchConfig,
  board: &Board,
  moves: &Vec<Move>,
  depth: u8,
  out: &mut Output,
) -> (Vec<Move>, Score) {
  let mut alpha = settings.initial_alpha;
  let beta = Score::Win(0);
  let mut best_pv = Vec::new();
  let mut backup_pv = Vec::new();
  let mut move_count = 0;
  let mut show_output = false;
  for candidate in moves {
    let mut position = board.clone();
    position.play_move(*candidate);
    settings.nodes += 1;
    move_count += 1;
    let (mut pv, score) = if move_count > 1 {
      // Zero window search to see if raises alpha
      let score = -zero_window_search(state, settings, &position, depth - 1, -alpha, false);
      if score > alpha {
        if settings.search_is_over() {
          return (best_pv, alpha);
        }
        backup_pv = best_pv;
        best_pv = vec![*candidate];
        if show_output {
          print_info(
            out,
            board,
            alpha,
            depth,
            settings,
            &best_pv,
            state.table.capacity(),
          );
        }
        let (pv, score) = alpha_beta(
          state,
          settings,
          &position,
          depth - 1,
          -beta,
          -alpha,
          true,
          false,
        );
        (pv, -score)
      } else {
        (Vec::new(), score)
      }
    } else {
      let (pv, score) = alpha_beta(
        state,
        settings,
        &position,
        depth - 1,
        -beta,
        -alpha,
        true,
        false,
      );
      if settings.millis >= 100 {
        show_output = true;
      }
      (pv, -score)
    };
    if settings.search_is_over() {
      return (best_pv, alpha);
    }
    if score > alpha {
      alpha = score;
      let mut new_pv = vec![*candidate];
      new_pv.append(&mut pv);
      best_pv = new_pv;
      backup_pv = best_pv.clone();
      if show_output {
        print_info(
          out,
          board,
          alpha,
          depth,
          settings,
          &best_pv,
          state.table.capacity(),
        );
      }
    } else {
      // In case of PVS research fail-low, revert best pv
      best_pv = backup_pv.clone();
    }
    if !settings.hard_tm && settings.start.elapsed().as_millis() * 5 >= settings.max_time * 4 {
      return (best_pv, alpha);
    }
  }
  (best_pv, alpha)
}
