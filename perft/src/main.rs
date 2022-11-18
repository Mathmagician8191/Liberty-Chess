use liberty_chess::Board;
use std::sync::mpsc::channel;
use std::time::{Duration, Instant};

#[cfg(feature = "parallel")]
use threadpool::ThreadPool;

// Updated 10 Nov 2022
// 5600x benchmarks - multithreaded
// 1 million = 1.4s
// 5 million = 5.6s
// 10 million = 10.5s
// 30 million = 27s
// 100 million = 53s
// 200 million = 208s
// max = 14 1/2 mins
// Updated 18 Nov 2022
// i5 8400 benchmarks - multithreaded
// 2 million = 2.8s
// 5 million = 9.7s
// 10 million = 18.5s
// 30 million = 48s
// 100 million = 97s
// 200 million = 380s
// max = 26 1/2 mins
const LIMIT: usize = 2_000_000;

fn print_time(fen: &str, time: Duration, depth: usize, nodes: usize) {
  let secs = time.as_secs();
  let millis = time.as_millis();
  let kilonodes = nodes / usize::max(millis as usize, 1);
  let time = if secs >= 30 {
    format!("{} s", secs)
  } else {
    format!("{} ms", millis)
  };
  println!(
    "{} {} for depth {} ({} knodes/s)",
    fen, time, depth, kilonodes
  );
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
  let pool = ThreadPool::default();

  for (i, result) in results.iter().enumerate().take(max) {
    let fen = board.to_string();
    let result = *result;
    let closure = move || assert_eq!(perft(&Board::new(&fen).unwrap(), i), result);
    #[cfg(feature = "parallel")]
    pool.execute(closure);
    #[cfg(not(feature = "parallel"))]
    closure();
  }

  let (tx, rx) = channel();
  let moves = board.generate_legal();
  let num_moves = moves.len();
  for board in moves {
    let tx = tx.clone();
    let fen = board.to_string();
    let closure = move || tx.send(perft(&Board::new(&fen).unwrap(), max - 1)).unwrap();
    #[cfg(feature = "parallel")]
    pool.execute(closure);
    #[cfg(not(feature = "parallel"))]
    closure();
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
  perft_test(
    "rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8",
    &[1, 44, 1_486, 62_379, 2_103_487, 89_941_194],
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

  //loaded board
  perft_test(
    "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1",
    &[1, 28, 778, 21_974, 617_017, 17_962_678, 527_226_103],
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

  println!("{} ms", start.elapsed().as_millis());
}
