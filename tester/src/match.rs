use liberty_chess::clock::format_time;
use liberty_chess::moves::Move;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate};
use oxidation::random_move;
use rand::{thread_rng, Rng};
use std::collections::{HashMap, HashSet};
use std::fs::write;
use std::io::BufReader;
use std::ops::AddAssign;
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use std::time::Instant;
use tester::POSITIONS;
use threadpool::ThreadPool;
use ulci::server::{startup, AnalysisRequest, Request, UlciResult};
use ulci::SearchTime;

const CHAMPION: &str = "./target/release/oxidation";

const CHALLENGER: &str = "./target/release/oxidation";

const GAME_PAIR_COUNT: usize = 160;
const RANDOM_MOVE_COUNT: usize = 4;

const CHAMP_TIME: SearchTime = SearchTime::Increment(8000, 80);
const CHALLENGE_TIME: SearchTime = CHAMP_TIME;

struct GameInfo {
  result: GameResult,
  points: u32,
  champ_moves: (u32, u32, u32),
  challenge_moves: (u32, u32, u32),
  champ_depth: (u32, u32, u32),
  challenge_depth: (u32, u32, u32),
  positions: HashSet<String>,
}

enum GameResult {
  ChampWin,
  Draw,
  ChallengeWin,
}

fn sum_tuple<T: AddAssign>(accumulator: &mut (T, T, T), element: (T, T, T)) {
  accumulator.0 += element.0;
  accumulator.1 += element.1;
  accumulator.2 += element.2;
}

fn total_tuple<T: AddAssign>(tuple: (T, T, T)) -> T {
  let mut result = tuple.0;
  result += tuple.1;
  result += tuple.2;
  result
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
  move_threshold: u32,
  current_board: &mut Board,
  total_depth: &mut (u32, u32, u32),
  move_count: &mut (u32, u32, u32),
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
            break;
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
        let moves = board.moves();
        if moves > 2 * move_threshold {
          total_depth.2 += depth;
          move_count.2 += 1;
        } else if moves > move_threshold {
          total_depth.1 += depth;
          move_count.1 += 1;
        } else {
          total_depth.0 += depth;
          move_count.0 += 1;
        }
        break;
      }
      UlciResult::Startup(_, _) | UlciResult::Info(_, _) => (),
    }
  }
}

fn play_game(
  board: CompressedBoard,
  move_count: u32,
  champion_side: bool,
  results: &Sender<GameInfo>,
) {
  let (champ_requests, champ_results) = load_engine(CHAMPION);
  let (challenge_requests, challenge_results) = load_engine(CHALLENGER);
  let (mut champ_moves, mut challenge_moves) = ((0, 0, 0), (0, 0, 0));
  let (mut champ_depth, mut challenge_depth) = ((0, 0, 0), (0, 0, 0));
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
        move_count,
        &mut current_board,
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
        move_count,
        &mut current_board,
        &mut champ_depth,
        &mut champ_moves,
        &mut champ_tc,
      );
    }
    positions.insert(current_board.to_string());
  }
  let (result, points) = match current_board.state() {
    Gamestate::InProgress => unreachable!(),
    Gamestate::Checkmate(winner) | Gamestate::Elimination(winner) => (
      if champion_side ^ winner {
        GameResult::ChallengeWin
      } else {
        GameResult::ChampWin
      },
      if winner { 2 } else { 0 },
    ),
    Gamestate::Material | Gamestate::Move50 | Gamestate::Repetition | Gamestate::Stalemate => {
      (GameResult::Draw, 1)
    }
  };
  results
    .send(GameInfo {
      result,
      points,
      champ_moves,
      challenge_moves,
      champ_depth,
      challenge_depth,
      positions,
    })
    .ok();
}

fn test_position(
  name: &str,
  board: &Board,
  moves: u32,
  positions: &mut HashMap<String, (u32, u32)>,
) {
  println!("Testing {name}");
  let cores = std::thread::available_parallelism().unwrap().get();
  let pool = ThreadPool::new(cores - 1);
  let champion_side: bool = thread_rng().gen();
  let (tx, rx) = channel();
  for _ in 0..GAME_PAIR_COUNT {
    let tx = tx.clone();
    let tx2 = tx.clone();
    let mut board = board.clone();
    for _ in 1..RANDOM_MOVE_COUNT {
      if let Some(randommove) = random_move(&board) {
        if let Some(new_board) = board.move_if_legal(randommove) {
          board = new_board;
        }
      }
    }
    let compressed_board = board.send_to_thread();
    let compressed_board_2 = compressed_board.clone();
    pool.execute(move || play_game(compressed_board, moves, champion_side, &tx));
    pool.execute(move || play_game(compressed_board_2, moves, !champion_side, &tx2));
  }
  // to make sure it actually finishes
  drop(tx);
  let (mut win, mut draw, mut loss) = (0, 0, 0);
  let (mut champ_moves, mut challenge_moves) = ((0, 0, 0), (0, 0, 0));
  let (mut champ_depth, mut challenge_depth) = ((0, 0, 0), (0, 0, 0));
  for result in &rx {
    match result.result {
      GameResult::ChampWin => win += 1,
      GameResult::Draw => draw += 1,
      GameResult::ChallengeWin => loss += 1,
    };
    let game_score = result.points;
    for position in result.positions {
      if let Some(result) = positions.get_mut(&position) {
        result.0 += 1;
        result.1 += game_score;
      } else {
        positions.insert(position, (1, game_score));
      }
    }
    sum_tuple(&mut champ_moves, result.champ_moves);
    sum_tuple(&mut challenge_moves, result.challenge_moves);
    sum_tuple(&mut champ_depth, result.champ_depth);
    sum_tuple(&mut challenge_depth, result.challenge_depth);
  }
  assert_eq!(win + draw + loss, GAME_PAIR_COUNT * 2);
  let move_count = total_tuple(champ_moves) + total_tuple(challenge_moves);
  let average_move_count = move_count as usize / GAME_PAIR_COUNT / 2;
  println!("+{win} ={draw} -{loss}, {average_move_count} moves per game");
  println!(
    "Average opening depth: Champion: {:.2}, Challenger: {:.2}",
    champ_depth.0 as f32 / champ_moves.0 as f32,
    challenge_depth.0 as f32 / challenge_moves.0 as f32
  );
  println!(
    "Average middlegame depth: Champion: {:.2}, Challenger: {:.2}",
    champ_depth.1 as f32 / champ_moves.1 as f32,
    challenge_depth.1 as f32 / challenge_moves.1 as f32
  );
  println!(
    "Average endgame depth: Champion: {:.2}, Challenger: {:.2}",
    champ_depth.2 as f32 / champ_moves.2 as f32,
    challenge_depth.2 as f32 / challenge_moves.2 as f32
  );
}

fn main() {
  for (name, position, moves, _) in POSITIONS {
    let mut positions = HashMap::new();
    let mut board = Board::new(position).expect("Loading board failed");
    test_position(name, &board, *moves, &mut positions);
    board.friendly_fire = true;
    test_position(&format!("friendly {name}"), &board, *moves, &mut positions);
    let data = positions
      .iter()
      .map(|(position, (games, score))| format!("{position};{games};{score}"))
      .collect::<Vec<String>>()
      .join("\n");
    write(format!("target/release/{name}.txt"), data).expect("Writing file failed");
  }
}
