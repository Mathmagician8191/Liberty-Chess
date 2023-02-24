#![forbid(unsafe_code)]
#![warn(missing_docs)]
//! A helpful program to test and benchmark the move generation

use liberty_chess::Board;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

#[cfg(feature = "parallel")]
use std::sync::mpsc::Sender;
#[cfg(feature = "parallel")]
use threadpool::ThreadPool;

// Updated 7 Februrary 2023
// 5600x benchmarks - multithreaded
// 5 million = 5.7s
// 10 million = 9.9s
// 30 million = 26.5s
// 100 million = 53.6s
// 150 million = 131s
// 200 million = 209 s
// 500 million = 384s
// max = 955s
// Updated 18 Nov 2022
// i5 8400 benchmarks - multithreaded
// 2 million = 2.8s
// 5 million = 9.7s
// 10 million = 18.5s
// 30 million = 48s
// 100 million = 97s
// 200 million = 380s
// max = 26 1/2 mins
const LIMIT: usize = 5_000_000;

fn format_time(millis: u128) -> String {
  let secs = millis / 1000;
  if secs >= 180 {
    format!("{secs} s")
  } else if secs >= 20 {
    format!("{secs}.{} s", (millis / 100) % 10)
  } else {
    format!("{millis} ms")
  }
}

fn print_time(fen: &str, time: Duration, depth: usize, nodes: usize) {
  let millis = time.as_millis();
  let kilonodes = nodes / usize::max(millis as usize, 1);
  let time = format_time(millis);
  println!("{time} for depth {depth} ({kilonodes} knodes/s) {fen}");
}

fn perft(board: &Board, depth: usize) -> usize {
  if depth > 0 {
    let mut result = 0;
    for position in board.generate_legal() {
      result += perft(&position, depth - 1);
    }
    result
  } else {
    1
  }
}

#[cfg(feature = "parallel")]
fn perft_process_final(pool: &ThreadPool, tx: Sender<usize>, board: &Board, depth: usize) {
  let fen = board.to_string();
  let closure = move || tx.send(perft(&Board::new(&fen).unwrap(), depth)).unwrap();
  pool.execute(closure);
}

#[cfg(feature = "parallel")]
fn perft_process_other(pool: &ThreadPool, board: &Board, depth: usize, result: usize) {
  let fen = board.to_string();
  let closure = move || assert_eq!(perft(&Board::new(&fen).unwrap(), depth), result);
  pool.execute(closure);
}

fn perft_test(fen: &'static str, results: &[usize]) {
  let board = Board::new(fen).unwrap();
  assert_eq!(board.to_string(), fen);
  let start = Instant::now();
  let mut max = 0;
  let mut nodes = 0;
  for (i, result) in results.iter().enumerate() {
    if result <= &LIMIT {
      max = i;
      nodes += result;
    } else {
      break;
    }
  }

  #[cfg(feature = "parallel")]
  let cores = std::thread::available_parallelism().unwrap().get();
  #[cfg(feature = "parallel")]
  let pool = ThreadPool::new(cores);

  for (i, result) in results.iter().enumerate().take(max) {
    #[cfg(feature = "parallel")]
    perft_process_other(&pool, &board, i, *result);
    #[cfg(not(feature = "parallel"))]
    assert_eq!(perft(&board, i), *result);
  }

  let (tx, rx) = channel();
  let moves = board.generate_legal();
  #[cfg(feature = "parallel")]
  let extra_cores = results[1] < 2 * cores;
  #[cfg(not(feature = "parallel"))]
  let extra_cores = false;
  let num_moves = if extra_cores { results[2] } else { moves.len() };
  for board in moves {
    #[cfg(feature = "parallel")]
    {
      if extra_cores {
        for new_board in board.generate_legal() {
          perft_process_final(&pool, tx.clone(), &new_board, max - 2);
        }
      } else {
        perft_process_final(&pool, tx.clone(), &board, max - 1);
      }
    }
    #[cfg(not(feature = "parallel"))]
    tx.send(perft(&board, max - 1)).unwrap();
  }
  #[cfg(feature = "parallel")]
  pool.join();
  assert_eq!(rx.iter().take(num_moves).sum::<usize>(), results[max]);
  print_time(fen, start.elapsed(), max, nodes);
}

fn main() {
  let start = Instant::now();

  // standard chess
  perft_test(
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1",
    &[1, 20, 400, 8_902, 197_281, 4_865_609, 119_060_324],
  );

  // positions 2-6 from https://www.chessprogramming.org/Perft_Results
  perft_test(
    "r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1",
    &[1, 48, 2_039, 97_862, 4_085_603, 193_690_690],
  );
  perft_test(
    "8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1",
    &[1, 14, 191, 2_812, 43_238, 674_624, 11_030_083, 178_633_661],
  );
  perft_test(
    "r3k2r/Pppp1ppp/1b3nbN/nP6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1",
    &[1, 6, 264, 9_467, 422_333, 15_833_292, 706_045_033],
  );
  // results are modified due to El Vaticano
  perft_test(
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    &[1, 44, 1_486, 62_379, 2_103_487, 89_941_236],
  );
  perft_test(
    "r4rk1/1pp1qppp/p1np1n2/2b1p1B1/2B1P1b1/P1NP1N2/1PP1QPPP/R4RK1 w - - 0 10",
    &[1, 46, 2_079, 89_890, 3_894_594, 164_075_551],
  );

  // capablanca's chess
  perft_test(
    "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1",
    &[1, 28, 784, 25_228, 805_128, 28_741_319, 1_015_802_437],
  );
  perft_test(
    "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3",
    &[1, 38, 1_444, 60_046, 2_486_600, 111_941_832],
  );

  //liberty chess - not tested with external sources
  perft_test(
    "ruabhqkhbcur/wlzenxxnezlw/pppppppppppp/12/12/12/12/12/12/PPPPPPPPPPPP/WLZENXXNEZLW/RUABHQKHBCUR w KQkq - 0 1 3,3 qcaehurwbznxl",
    &[1, 194, 37_464, 7_294_214],
  );

  //mini chess
  perft_test(
    "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1",
    &[1, 7, 49, 457, 4_065, 44_137, 476_690, 5_914_307],
  );

  // mongol chess
  perft_test(
    "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1 - iznl",
    &[1, 28, 784, 21_958, 614_381, 17_398_402, 491_118_153],
  );

  //african chess - not tested with external sources
  perft_test(
    "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1 - enzl",
    &[1, 28, 784, 21_900, 606_601, 16_950_392, 469_862_168],
  );

  //narnia chess - not tested with external sources
  perft_test(
    "uuqkkquu/pppppppp/8/8/8/8/PPPPPPPP/UUQKKQUU w - - 0 1 - u",
    &[1, 22, 484, 12_630, 328_732, 9_831_732, 291_534_968],
  );

  //trump chess - not tested with external sources
  perft_test(
    "rwwwkwwr/pppppppp/8/8/8/8/PPPPPPPP/RWWWKWWR w KQkq - 0 1 - mrw",
    &[1, 176, 30_856, 5_410_950],
  );

  //loaded board - results modified by El Vaticano
  perft_test(
    "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1 1",
    &[1, 28, 778, 21_974, 618_165, 18_025_422, 531_219_743],
  );

  //double chess - not tested with external sources
  perft_test(
    "rnbqkbnrrnbqkbnr/pppppppppppppppp/16/16/16/16/PPPPPPPPPPPPPPPP/RNBQKBNRRNBQKBNR w KQkq - 0 1",
    &[1, 40, 1_592, 68_142, 2_898_457, 132_653_171],
  );

  //friendly fire
  perft_test(
    "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1 - qrbn ff",
    &[1, 39, 1_519, 63_034, 2_598_922, 112_776_461],
  );

  // test positions from totally normal chess
  perft_test(
    "ciamkaic/pppppppp/8/8/8/8/PPPPPPPP/CIAMKAIC w - - 0 1 - mcai",
    &[1, 32, 1_026, 38_132, 1_401_550, 56_909_620],
  );
  perft_test(
    "hixakxih/pppppppp/8/8/8/8/PPPPPPPP/HIXAKXIH w - - 0 1 - ahix",
    &[1, 30, 899, 29_509, 958_995, 33_463_252],
  );
  perft_test(
    "rnobqkbonr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNOBQKBONR w KQkq - 0 1 3 qcarbno",
    &[1, 154, 23_632, 3_612_238],
  );
  perft_test(
    "rnabmkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABMKBCNR w KQkq - 0 1 3 mqcarbn",
    &[1, 40, 1_600, 71_502, 3_178_819],
  );

  println!("{}", format_time(start.elapsed().as_millis()));
}
