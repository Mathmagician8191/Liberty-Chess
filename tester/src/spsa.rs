use liberty_chess::clock::format_time;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate};
use oxidation::glue::process_position;
use oxidation::parameters::DEFAULT_PARAMETERS;
use oxidation::search::{SearchParameters, SEARCH_PARAMETERS};
use oxidation::{State, HASH_SIZE, QDEPTH};
use rand::{thread_rng, Rng};
use std::sync::mpsc::{channel, Sender};
use std::time::Instant;
use tester::{get_threadpool, GameResult, POSITIONS, STC};
use ulci::server::UlciResult;
use ulci::SearchTime;

const ITERATION_COUNT: u16 = 2000;

const ALPHA: f32 = 0.602;
const GAMMA: f32 = 0.101;

const TC: SearchTime = STC;

trait Spsa<T>
where
  Self: Sized,
{
  fn random_delta() -> Self;

  fn apply_params(self, delta: Self, c_k: T) -> (Self, Self);

  fn update_params(&mut self, delta: Self, a_k: T, score: i32, count: u32);
}

impl Spsa<f32> for SearchParameters {
  fn random_delta() -> Self {
    let mut rng = thread_rng();
    Self {
      lmr_base: rng.gen_range(-0.06..0.06),
      lmr_factor: rng.gen_range(-0.04..0.04),
      lmr_pv_reduction: rng.gen_range(-0.1..0.1),
    }
  }

  fn apply_params(self, delta: Self, c_k: f32) -> (Self, Self) {
    let delta = delta * c_k;
    (self + delta, self - delta)
  }

  fn update_params(&mut self, delta: Self, a_k: f32, score: i32, count: u32) {
    let scale_factor = a_k * score as f32 / f32::sqrt(count as f32);
    *self = *self + delta * scale_factor;
  }
}

fn process_move(state: &mut State, board: &mut Board, search_time: &mut SearchTime) {
  let move_time = Instant::now();
  let (tx, rx) = channel();
  let (_tx_2, rx_2) = channel();
  process_position(
    &tx,
    &rx_2,
    board.send_to_thread(),
    *search_time,
    QDEPTH,
    state,
  );
  while let Ok(result) = rx.recv() {
    match result {
      UlciResult::Analysis(results) => {
        let mut test_board = board.clone();
        for pv_move in results.pv {
          if let Some(new_board) = test_board.move_if_legal(pv_move) {
            test_board = new_board;
          } else {
            println!(
              "illegal pv move {} in position {}",
              pv_move.to_string(),
              test_board.to_string()
            );
            break;
          }
        }
      }
      UlciResult::AnalysisStopped(bestmove) => {
        if let Some(new_board) = board.move_if_legal(bestmove) {
          *board = new_board;
        } else {
          println!(
            "illegal move {} in position {}",
            bestmove.to_string(),
            board.to_string()
          );
        }
        let elapsed = move_time.elapsed();
        let millis = elapsed.as_millis();
        match search_time {
          SearchTime::Increment(time, inc) => {
            let excess = millis.saturating_sub(*time);
            if excess > 0 {
              println!(
                "{} extra time in posiiton {}",
                format_time(excess),
                board.to_string()
              );
            }
            *time = time.saturating_sub(millis) + *inc;
          }
          SearchTime::Asymmetric(wtime, winc, btime, binc) => {
            let (time, inc) = if board.to_move() {
              (wtime, winc)
            } else {
              (btime, binc)
            };
            let excess = millis.saturating_sub(*time);
            if excess > 0 {
              println!(
                "{} extra time in posiiton {}",
                format_time(excess),
                board.to_string()
              );
            }
            *time = time.saturating_sub(millis) + *inc;
          }
          SearchTime::Other(limits) => {
            let excess = millis.saturating_sub(limits.time);
            if excess >= 25 {
              println!(
                "{} extra time in posiiton {}",
                format_time(excess),
                board.to_string()
              );
            }
          }
          SearchTime::Infinite | SearchTime::Mate(_) => (),
        }
        break;
      }
      UlciResult::Startup(_) | UlciResult::Info(..) => (),
    }
  }
}

fn play_game(
  board: CompressedBoard,
  params1: SearchParameters,
  params2: SearchParameters,
  side: bool,
  results: &Sender<GameResult>,
) {
  let mut board = board.load_from_thread();
  let mut pieces = [false; 18];
  for piece in board.board().elements_row_major_iter() {
    let piece = isize::from(*piece);
    if piece != 0 {
      pieces[piece.unsigned_abs() - 1] = true;
    }
  }
  for piece in board.promotion_options() {
    pieces[isize::from(*piece).unsigned_abs() - 1] = true;
  }
  let mut state_1 = State::new(HASH_SIZE, &board, params1, DEFAULT_PARAMETERS);
  let mut state_2 = State::new(HASH_SIZE, &board, params2, DEFAULT_PARAMETERS);
  let mut tc_1 = TC;
  let mut tc_2 = TC;
  while board.state() == Gamestate::InProgress {
    let (state, tc) = if board.to_move() ^ side {
      (&mut state_1, &mut tc_1)
    } else {
      (&mut state_2, &mut tc_2)
    };
    process_move(state, &mut board, tc);
  }
  let result = match board.state() {
    Gamestate::InProgress => unreachable!(),
    Gamestate::Checkmate(winner) | Gamestate::Elimination(winner) => {
      if side ^ winner {
        GameResult::ChallengeWin
      } else {
        GameResult::ChampWin
      }
    }
    Gamestate::Material | Gamestate::FiftyMove | Gamestate::Repetition | Gamestate::Stalemate => {
      GameResult::Draw
    }
  };
  results.send(result).ok();
}

fn run_match(params1: SearchParameters, params2: SearchParameters) -> (i32, u32) {
  let pool = get_threadpool();
  let (tx, rx) = channel();
  for (_, position, _) in POSITIONS {
    let position = position.get_position(thread_rng().gen_bool(0.5));
    let position_2 = position.clone();
    let tx = tx.clone();
    let tx_2 = tx.clone();
    pool.execute(move || play_game(position, params1, params2, true, &tx));
    pool.execute(move || play_game(position_2, params2, params1, false, &tx_2));
  }
  // to make sure it actually finishes
  drop(tx);
  let mut points = 0;
  let mut game_count = 0;
  for result in &rx {
    let score = match result {
      GameResult::ChampWin => 1,
      GameResult::Draw => 0,
      GameResult::ChallengeWin => -1,
    };
    game_count += 1;
    points += score;
  }
  (points, game_count)
}

fn main() {
  let mut params = SEARCH_PARAMETERS;
  for k in 0..ITERATION_COUNT {
    println!("Iteration {k}");
    let k = f32::from(k);
    let a_k = 1.0 / (k + 1.0).powf(ALPHA);
    let c_k = 1.0 / (k + 1.0).powf(GAMMA);
    let params_delta = SearchParameters::random_delta();
    let (params_plus, params_minus) = params.apply_params(params_delta, c_k);
    let (score, count) = run_match(params_plus, params_minus);
    params.update_params(params_delta, a_k / c_k, score, count);
    println!("{params:?}");
  }
}
