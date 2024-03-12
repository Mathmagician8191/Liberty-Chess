use liberty_chess::clock::format_time;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate};
use oxidation::glue::process_position;
use oxidation::parameters::{Parameters, DEFAULT_PARAMETERS};
use oxidation::{State, HASH_SIZE, QDEPTH};
use rand::{thread_rng, Rng};
use std::sync::mpsc::{channel, Sender};
use std::time::Instant;
use tester::{get_threadpool, GameResult, POSITIONS, STC};
use ulci::server::UlciResult;
use ulci::SearchTime;

const ITERATION_COUNT: u32 = 900;

const ALPHA: f64 = 0.602;
const GAMMA: f64 = 0.101;

const TC: SearchTime = STC;

#[derive(Debug)]
struct SPSAParams {
  mg_edge: [[f64; 5]; 18],
  eg_edge: [[f64; 5]; 18],
}

impl SPSAParams {
  fn random_delta() -> Self {
    Self {
      mg_edge: [[(); 5]; 18].map(|x| x.map(|()| thread_rng().gen_range(-10.0..=10.0))),
      eg_edge: [[(); 5]; 18].map(|x| x.map(|()| thread_rng().gen_range(-10.0..=10.0))),
    }
  }

  fn apply_params(&self, delta: &Self, c_k: f64) -> (Self, Self) {
    let mut mg_edge_plus = self.mg_edge;
    let mut mg_edge_minus = self.mg_edge;
    let mut eg_edge_plus = self.eg_edge;
    let mut eg_edge_minus = self.eg_edge;
    for i in 0..18 {
      for j in 0..5 {
        let mg_edge = delta.mg_edge[i][j] * c_k;
        mg_edge_plus[i][j] += mg_edge;
        mg_edge_minus[i][j] -= mg_edge;
        let eg_edge = delta.eg_edge[i][j] * c_k;
        eg_edge_plus[i][j] += eg_edge;
        eg_edge_minus[i][j] -= eg_edge;
      }
    }
    (
      Self {
        mg_edge: mg_edge_plus,
        eg_edge: eg_edge_plus,
      },
      Self {
        mg_edge: mg_edge_minus,
        eg_edge: eg_edge_minus,
      },
    )
  }

  fn update_params(&mut self, delta: &Self, a_k: f64, scores: [i32; 18], counts: [u32; 18]) {
    for i in 0..18 {
      let scale_factor = f64::from(scores[i]) / f64::sqrt(f64::from(counts[i]));
      for j in 0..5 {
        self.mg_edge[i][j] += a_k * delta.mg_edge[i][j] * scale_factor;
        self.eg_edge[i][j] += a_k * delta.eg_edge[i][j] * scale_factor;
      }
    }
  }
}

impl From<SPSAParams> for Parameters<i32> {
  fn from(value: SPSAParams) -> Self {
    Self {
      mg_edge: value.mg_edge.map(|x| x.map(|x| x as i32)),
      eg_edge: value.eg_edge.map(|x| x.map(|x| x as i32)),
      ..DEFAULT_PARAMETERS
    }
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
  params1: &Parameters<i32>,
  params2: &Parameters<i32>,
  side: bool,
  results: &Sender<(GameResult, [bool; 18])>,
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
  let mut state_1 = State::new(HASH_SIZE, &board, *params1);
  let mut state_2 = State::new(HASH_SIZE, &board, *params2);
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
  results.send((result, pieces)).ok();
}

fn run_match(params1: SPSAParams, params2: SPSAParams) -> ([i32; 18], [u32; 18]) {
  let params1 = params1.into();
  let params2 = params2.into();
  let pool = get_threadpool();
  let (tx, rx) = channel();
  for (_, position, _) in POSITIONS {
    let position = position.get_position(thread_rng().gen_bool(0.5));
    let position_2 = position.clone();
    let tx = tx.clone();
    let tx_2 = tx.clone();
    pool.execute(move || play_game(position, &params1, &params2, true, &tx));
    pool.execute(move || play_game(position_2, &params2, &params1, false, &tx_2));
  }
  // to make sure it actually finishes
  drop(tx);
  let mut points = [0; 18];
  let mut game_count = [0; 18];
  for (result, pieces) in &rx {
    let score = match result {
      GameResult::ChampWin => 1,
      GameResult::Draw => 0,
      GameResult::ChallengeWin => -1,
    };
    for (i, value) in pieces.iter().enumerate() {
      if *value {
        game_count[i] += 1;
        points[i] += score;
      }
    }
  }
  (points, game_count)
}

fn main() {
  let mut params = SPSAParams {
    mg_edge: DEFAULT_PARAMETERS.mg_edge.map(|x| x.map(f64::from)),
    eg_edge: DEFAULT_PARAMETERS.eg_edge.map(|x| x.map(f64::from)),
  };
  for k in 0..ITERATION_COUNT {
    println!("Iteration {k}");
    let k = f64::from(k);
    let a_k = 1.0 / (k + 1.0).powf(ALPHA);
    let c_k = 1.0 / (k + 1.0).powf(GAMMA);
    let params_delta = SPSAParams::random_delta();
    let (params_plus, params_minus) = params.apply_params(&params_delta, c_k);
    let (scores, counts) = run_match(params_plus, params_minus);
    params.update_params(&params_delta, a_k / c_k, scores, counts);
    println!("{params:?}");
  }
}
