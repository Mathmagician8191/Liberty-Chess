use liberty_chess::moves::Move;
use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, HORDE, LIBERTY_CHESS, LOADED_BOARD,
  MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::{Board, Gamestate};
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use std::time::{Duration, Instant};
use ulci::server::{startup, AnalysisRequest, Request, UlciResult};
use ulci::SearchTime;

const CHAMPION: &str = "./target/release/alphabeta";

const CHALLENGER: &str = "./target/release/oxidation";

const POSITIONS: &[(&str, &str)] = &[
  ("startpos", STARTPOS),
  ("rectangle", CAPABLANCA_RECTANGLE),
  ("capablanca", CAPABLANCA),
  ("liberty", LIBERTY_CHESS),
  ("mini", MINI),
  ("mongol", MONGOL),
  ("african", AFRICAN),
  ("narnia", NARNIA),
  ("trump", TRUMP),
  ("loaded", LOADED_BOARD),
  ("double", DOUBLE_CHESS),
  ("horde", HORDE),
  ("endgame", "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1"),
];

const MATCH_SIZE: usize = 500;

const CHAMP_TIME: SearchTime = SearchTime::Depth(2);
const CHALLENGE_TIME: SearchTime = SearchTime::Depth(2);

fn spawn_engine(path: &'static str, requests: Receiver<Request>, results: &Sender<UlciResult>) {
  let engine = Command::new(path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Loading engine failed");
  let stdin = engine.stdin.expect("Loading engine stdin failed");
  let stdout = engine.stdout.expect("Loading engine stdout failed");
  startup(requests, results, BufReader::new(stdout), stdin, false);
}

fn load_engine(path: &'static str) -> (Sender<Request>, Receiver<UlciResult>) {
  let (send_results, results) = channel();
  let (tx, rx) = channel();
  spawn(move || spawn_engine(path, rx, &send_results));
  while let Ok(result) = results.recv() {
    if let UlciResult::Startup(_, _) = result {
      break;
    }
  }
  (tx, results)
}

// Copied from perft with modification
fn format_time(micros: u128) -> String {
  let millis = micros / 1000;
  if millis > 100 {
    format!("{millis} ms")
  } else if millis >= 10 {
    format!("{millis}.{} ms", (micros / 100) % 10)
  } else {
    format!("{micros} μs")
  }
}

fn process_move(
  name: &'static str,
  results: &Receiver<UlciResult>,
  board: &mut Board,
  moves: &mut Vec<Move>,
  current_board: &mut Board,
  time: &mut Duration,
  move_count: &mut u32,
) {
  let move_time = Instant::now();
  while let Ok(result) = results.recv() {
    match result {
      UlciResult::Analysis(results) => {
        let mut test_board = current_board.clone();
        for pv_move in results.pv {
          if let Some(new_board) = test_board.move_if_legal(pv_move) {
            test_board = new_board;
          } else {
            println!(
              "{name} made illegal pv move {} in position {}",
              pv_move.to_string(),
              test_board.to_string()
            );
          }
        }
      }
      UlciResult::AnalysisStopped(bestmove) => {
        if let Some(new_board) = current_board.move_if_legal(bestmove) {
          *current_board = new_board;
          if current_board.halfmoves() == 0 {
            *board = current_board.clone();
            moves.clear();
          } else {
            moves.push(bestmove);
          }
        } else {
          println!(
            "{name} made illegal move {} in position {}",
            bestmove.to_string(),
            current_board.to_string()
          );
        }
        *time += move_time.elapsed();
        *move_count += 1;
        break;
      }
      UlciResult::Startup(_, _) | UlciResult::Info(_, _) => (),
    }
  }
}

fn test_position(
  name: &str,
  board: &Board,
  champ_requests: &Sender<Request>,
  champ_results: &Receiver<UlciResult>,
  challenge_requests: &Sender<Request>,
  challenge_results: &Receiver<UlciResult>,
) {
  println!("Testing {name}");
  let (mut win, mut draw, mut loss) = (0, 0, 0);
  let mut champ_time = Duration::ZERO;
  let mut challenge_time = Duration::ZERO;
  let (mut champ_moves, mut challenge_moves) = (0, 0);
  let mut champion_side: bool = thread_rng().gen();
  let mut positions = HashSet::new();
  for _ in 0..MATCH_SIZE {
    champion_side = !champion_side;
    let mut board = board.clone();
    let mut moves = Vec::new();
    let mut current_board = board.clone();
    while current_board.state() == Gamestate::InProgress {
      if current_board.to_move() ^ champion_side {
        challenge_requests
          .send(Request::Analysis(AnalysisRequest {
            fen: board.to_string(),
            moves: moves.clone(),
            time: CHALLENGE_TIME,
            searchmoves: Vec::new(),
          }))
          .ok();
        process_move(
          "challenger",
          challenge_results,
          &mut board,
          &mut moves,
          &mut current_board,
          &mut challenge_time,
          &mut challenge_moves,
        );
      } else {
        champ_requests
          .send(Request::Analysis(AnalysisRequest {
            fen: board.to_string(),
            moves: moves.clone(),
            time: CHAMP_TIME,
            searchmoves: Vec::new(),
          }))
          .ok();
        process_move(
          "champion",
          champ_results,
          &mut board,
          &mut moves,
          &mut current_board,
          &mut champ_time,
          &mut champ_moves,
        );
      }
      if current_board.moves() <= 4 {
        positions.insert(current_board.hash());
      }
    }
    match current_board.state() {
      Gamestate::InProgress => unreachable!(),
      Gamestate::Checkmate(winner) | Gamestate::Elimination(winner) => {
        if champion_side ^ winner {
          loss += 1;
        } else {
          win += 1;
        }
      }
      Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
        draw += 1;
      }
    }
  }
  println!(
    "+{win} ={draw} -{loss}, {} moves",
    champ_moves + challenge_moves
  );
  let champ_time = format_time((champ_time / champ_moves).as_micros());
  let challenge_time = format_time((challenge_time / challenge_moves).as_micros());
  println!("Champion: {champ_time} per move, Challenger: {challenge_time} per move");
  println!("{} unique positions", positions.len());
}

fn main() {
  let (champ_requests, champ_results) = load_engine(CHAMPION);
  let (challenge_requests, challenge_results) = load_engine(CHALLENGER);
  for (name, position) in POSITIONS {
    let mut board = Board::new(position).expect("Loading board failed");
    test_position(
      name,
      &board,
      &champ_requests,
      &champ_results,
      &challenge_requests,
      &challenge_results,
    );
    board.friendly_fire = true;
    test_position(
      &format!("friendly {name}"),
      &board,
      &champ_requests,
      &champ_results,
      &challenge_requests,
      &challenge_results,
    );
  }
}
