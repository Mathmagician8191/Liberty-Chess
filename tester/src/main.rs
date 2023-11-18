#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! A testing program for comparing 2 different engines against each other in a range of positions.

use liberty_chess::clock::format_time;
use liberty_chess::moves::Move;
use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, HORDE, LIBERTY_CHESS, LOADED_BOARD,
  MINI, MONGOL, NARNIA, STARTPOS, TRUMP, ELIMINATION,
};
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate, Hash};
use rand::{thread_rng, Rng};
use std::collections::HashSet;
use std::io::BufReader;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use std::time::{Duration, Instant};
use threadpool::ThreadPool;
use ulci::server::{startup, AnalysisRequest, Request, UlciResult};
use ulci::SearchTime;

const CHAMPION: &str = "./target/release/nott";

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
  ("elimination", ELIMINATION),
  ("endgame", "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1"),
];

const MATCH_SIZE: usize = 200;
const MOVE_COUNT: usize = 100;

const CHAMP_TIME: SearchTime = SearchTime::Increment(5000, 50);
const CHALLENGE_TIME: SearchTime = SearchTime::Increment(5000, 50);

struct GameInfo {
  result: GameResult,
  champ_moves: [u32; MOVE_COUNT],
  challenge_moves: [u32; MOVE_COUNT],
  champ_depth: [u32; MOVE_COUNT],
  challenge_depth: [u32; MOVE_COUNT],
  champ_time: [Duration; MOVE_COUNT],
  challenge_time: [Duration; MOVE_COUNT],
  positions: HashSet<Hash>,
}

enum GameResult {
  ChampWin,
  Draw,
  ChallengeWin,
}

fn spawn_engine(path: &'static str, requests: Receiver<Request>, results: &Sender<UlciResult>) {
  let mut engine = Command::new(path)
    .stdin(Stdio::piped())
    .stdout(Stdio::piped())
    .spawn()
    .expect("Loading engine failed");
  let stdin = engine.stdin.take().expect("Loading engine stdin failed");
  let stdout = engine.stdout.take().expect("Loading engine stdout failed");
  startup(requests, results, BufReader::new(stdout), stdin, false);
  // To avoid your computer being infected by thousands of zombies
  engine.wait().expect("Waiting failed");
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

fn process_move(
  name: &'static str,
  results: &Receiver<UlciResult>,
  board: &mut Board,
  moves: &mut Vec<Move>,
  current_board: &mut Board,
  time: &mut [Duration; MOVE_COUNT],
  total_depth: &mut [u32; MOVE_COUNT],
  move_count: &mut [u32; MOVE_COUNT],
  search_time: &mut SearchTime,
) {
  let move_time = Instant::now();
  let mut depth = 0;
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
        depth = u32::from(results.depth);
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
        let elapsed = move_time.elapsed();
        let millis = elapsed.as_millis();
        match search_time {
          SearchTime::Increment(time, inc) => {
            let excess = millis.saturating_sub(*time);
            if excess > 0 {
              println!(
                "{name} took {} extra time in posiiton {}",
                format_time(excess),
                current_board.to_string()
              );
            }
            *time = time.saturating_sub(millis) + *inc;
          }
          SearchTime::Other(limits) => {
            let excess = millis.saturating_sub(limits.time);
            if excess >= 25 {
              println!(
                "{name} took {} extra time in posiiton {}",
                format_time(excess),
                current_board.to_string()
              );
            }
          }
          SearchTime::Infinite => (),
        }
        let index = (usize::from(current_board.moves() - 1)).min(MOVE_COUNT - 1);
        time[index] += elapsed;
        total_depth[index] += depth;
        move_count[index] += 1;
        break;
      }
      UlciResult::Startup(_, _) | UlciResult::Info(_, _) => (),
    }
  }
}

fn play_game(board: CompressedBoard, champion_side: bool, results: &Sender<GameInfo>) {
  let (champ_requests, champ_results) = load_engine(CHAMPION);
  let (challenge_requests, challenge_results) = load_engine(CHALLENGER);
  let mut champ_time = [Duration::ZERO; MOVE_COUNT];
  let mut challenge_time = [Duration::ZERO; MOVE_COUNT];
  let (mut champ_moves, mut challenge_moves) = ([0; MOVE_COUNT], [0; MOVE_COUNT]);
  let (mut champ_depth, mut challenge_depth) = ([0; MOVE_COUNT], [0; MOVE_COUNT]);
  let mut positions = HashSet::new();
  let mut board = board.load_from_thread();
  let mut moves = Vec::new();
  let mut current_board = board.clone();
  let mut champ_tc = CHAMP_TIME;
  let mut challenge_tc = CHALLENGE_TIME;
  while current_board.state() == Gamestate::InProgress {
    if current_board.to_move() ^ champion_side {
      challenge_requests
        .send(Request::Analysis(AnalysisRequest {
          fen: board.to_string(),
          moves: moves.clone(),
          time: challenge_tc,
          searchmoves: Vec::new(),
        }))
        .ok();
      process_move(
        "challenger",
        &challenge_results,
        &mut board,
        &mut moves,
        &mut current_board,
        &mut challenge_time,
        &mut challenge_depth,
        &mut challenge_moves,
        &mut challenge_tc,
      );
    } else {
      champ_requests
        .send(Request::Analysis(AnalysisRequest {
          fen: board.to_string(),
          moves: moves.clone(),
          time: champ_tc,
          searchmoves: Vec::new(),
        }))
        .ok();
      process_move(
        "champion",
        &champ_results,
        &mut board,
        &mut moves,
        &mut current_board,
        &mut champ_time,
        &mut champ_depth,
        &mut champ_moves,
        &mut champ_tc,
      );
    }
    if current_board.moves() <= 4 {
      positions.insert(current_board.hash());
    }
  }
  let result = match current_board.state() {
    Gamestate::InProgress => unreachable!(),
    Gamestate::Checkmate(winner) | Gamestate::Elimination(winner) => {
      if champion_side ^ winner {
        GameResult::ChallengeWin
      } else {
        GameResult::ChampWin
      }
    }
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      GameResult::Draw
    }
  };
  results
    .send(GameInfo {
      result,
      champ_moves,
      challenge_moves,
      champ_depth,
      challenge_depth,
      champ_time,
      challenge_time,
      positions,
    })
    .ok();
}

fn test_position(name: &str, board: &Board) {
  println!("Testing {name}");
  let cores = std::thread::available_parallelism().unwrap().get();
  let pool = ThreadPool::new(cores - 1);
  let mut champion_side: bool = thread_rng().gen();
  let (tx, rx) = channel();
  for _ in 0..MATCH_SIZE {
    champion_side = !champion_side;
    let tx = tx.clone();
    let compressed_board = board.send_to_thread();
    pool.execute(move || play_game(compressed_board, champion_side, &tx));
  }
  // to make sure it actually finishes
  drop(tx);
  let (mut win, mut draw, mut loss) = (0, 0, 0);
  let mut champ_time = [Duration::ZERO; MOVE_COUNT];
  let mut challenge_time = [Duration::ZERO; MOVE_COUNT];
  let (mut champ_moves, mut challenge_moves) = ([0; MOVE_COUNT], [0; MOVE_COUNT]);
  let (mut champ_depth, mut challenge_depth) = ([0; MOVE_COUNT], [0; MOVE_COUNT]);
  let mut positions = HashSet::new();
  for result in &rx {
    match result.result {
      GameResult::ChampWin => win += 1,
      GameResult::Draw => draw += 1,
      GameResult::ChallengeWin => loss += 1,
    }
    for (i, value) in result.champ_time.iter().enumerate() {
      champ_time[i] += *value;
    }
    for (i, value) in result.challenge_time.iter().enumerate() {
      challenge_time[i] += *value;
    }
    for (i, value) in result.champ_moves.iter().enumerate() {
      champ_moves[i] += *value;
    }
    for (i, value) in result.challenge_moves.iter().enumerate() {
      challenge_moves[i] += *value;
    }
    for (i, value) in result.champ_depth.iter().enumerate() {
      champ_depth[i] += *value;
    }
    for (i, value) in result.challenge_depth.iter().enumerate() {
      challenge_depth[i] += *value;
    }
    for position in result.positions {
      positions.insert(position);
    }
  }
  assert_eq!(win + draw + loss, MATCH_SIZE);
  println!(
    "+{win} ={draw} -{loss}, {} moves",
    champ_moves.iter().sum::<u32>() + challenge_moves.iter().sum::<u32>()
  );
  println!(
    "Champion: {:.2} average depth, Challenger: {:.2} average depth",
    champ_depth.iter().sum::<u32>() as f32 / champ_moves.iter().sum::<u32>() as f32,
    challenge_depth.iter().sum::<u32>() as f32 / challenge_moves.iter().sum::<u32>() as f32
  );
  println!("{} unique positions", positions.len());
}

fn main() {
  for (name, position) in POSITIONS {
    let mut board = Board::new(position).expect("Loading board failed");
    test_position(name, &board);
    board.friendly_fire = true;
    test_position(&format!("friendly {name}"), &board);
  }
}
