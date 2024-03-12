use liberty_chess::clock::format_time;
use liberty_chess::parsing::from_chars;
use liberty_chess::positions::{
  get_startpos, AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE,
  LIBERTY_CHESS, LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::{Board, ALL_PIECES};
use oxidation::evaluate::evaluate;
use oxidation::parameters::DEFAULT_PARAMETERS;
use oxidation::{
  bench, divide, get_move_order, search, Output, SearchConfig, State, HASH_SIZE, QDEPTH,
  QDEPTH_NAME, VERSION_NUMBER,
};
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::thread::spawn;
use std::time::Instant;
use ulci::client::{startup, Message};
use ulci::{ClientInfo, IntOption, OptionValue, SupportedFeatures, UlciOption, V1Features};

const BENCH_DEPTH: i8 = 9;

const HASH_NAME: &str = "Hash";

// i8 is an offset for bench depth
const BENCH_POSITIONS: &[(&str, i8)] = &[
  (STARTPOS, 0),
  (CAPABLANCA_RECTANGLE, -1),
  (CAPABLANCA, -2),
  (LIBERTY_CHESS, -3),
  (MINI, 1),
  (MONGOL, -1),
  (AFRICAN, -1),
  (NARNIA, 0),
  (TRUMP, -3),
  (LOADED_BOARD, -4),
  (DOUBLE_CHESS, -2),
  (HORDE, -1),
  (ELIMINATION, 0),
  ("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1", -1),
];

fn startup_client(tx: &Sender<Message>) {
  let mut options = HashMap::new();
  options.insert(
    QDEPTH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: usize::from(QDEPTH),
      min: 0,
      max: 50,
    }),
  );
  options.insert(
    HASH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: HASH_SIZE,
      min: 0,
      max: 1 << 28,
    }),
  );
  let info = ClientInfo {
    features: SupportedFeatures {
      v1: V1Features::all(),
    },
    name: format!("Oxidation v{VERSION_NUMBER}"),
    username: None,
    author: "Mathmagician".to_owned(),
    options,
    pieces: from_chars(ALL_PIECES),
    depth: BENCH_DEPTH,
  };
  let input = BufReader::new(stdin());
  startup(tx, &info, input, stdout(), false);
}

fn main() {
  let (tx, rx) = channel();
  spawn(move || startup_client(&tx));
  let mut qdepth = QDEPTH;
  let mut hash_size = HASH_SIZE;
  let mut position = get_startpos();
  let mut state = State::new(hash_size, &position, DEFAULT_PARAMETERS);
  let mut debug = false;
  while let Ok(message) = rx.recv() {
    match message {
      Message::SetDebug(new_debug) => debug = new_debug,
      Message::UpdatePosition(board) => {
        position = board.load_from_thread();
        if state.new_position(&position) && debug {
          println!("info string Hash cleared");
        }
      }
      Message::Go(settings) => {
        let moves = get_move_order(&state, &position, &settings.moves);
        if moves.is_empty() {
          if debug {
            if settings.moves.is_empty() {
              println!(
                "info string servererror no legal moves from position {}",
                position.to_string()
              );
            } else {
              println!(
                "info string servererror all search moves are illegal in position {}",
                position.to_string()
              );
            }
          }
          println!("bestmove 0000");
        } else {
          let settings =
            SearchConfig::new_time(&position, &mut qdepth, settings.time, &rx, &mut debug);
          let pv = search(
            &mut state,
            settings,
            &mut position,
            moves,
            Output::String(stdout()),
          );
          println!("bestmove {}", pv[0].to_string());
        }
      }
      Message::Stop => {
        println!("info string servererror not currently searching");
      }
      Message::UpdateOption(name, value) => match &*name {
        QDEPTH_NAME => match value {
          OptionValue::UpdateInt(value) => qdepth = value as u8,
          _ => {
            if debug {
              println!("info string servererror incorrect option type");
            }
          }
        },
        HASH_NAME => match value {
          OptionValue::UpdateInt(value) => {
            if value != hash_size {
              hash_size = value;
              state = State::new(hash_size, &position, DEFAULT_PARAMETERS);
            }
          }
          _ => {
            if debug {
              println!("info string servererror incorrect option type");
            }
          }
        },
        _ => (),
      },
      Message::Eval => {
        println!(
          "info string score {}",
          evaluate(&state, &position).show_uci(position.moves(), position.to_move()),
        );
      }
      Message::Bench(depth) => {
        if depth < 5 {
          println!("info string servererror minimum bench depth 5");
        } else {
          let start = Instant::now();
          state.new_game(&position);
          let mut nodes = 0;
          for (position, depth_offset) in BENCH_POSITIONS {
            let depth = (depth + depth_offset) as u8;
            let mut board = Board::new(position).expect("Loading bench position {position} failed");
            nodes += bench(
              &mut state,
              &mut board,
              depth,
              &mut qdepth,
              &mut debug,
              &rx,
              Output::String(stdout()),
            );
            board.friendly_fire = true;
            nodes += bench(
              &mut state,
              &mut board,
              depth,
              &mut qdepth,
              &mut debug,
              &rx,
              Output::String(stdout()),
            );
          }
          let millis = start.elapsed().as_millis();
          println!(
            "Total time: {} Nodes: {nodes} NPS: {}",
            format_time(millis),
            nodes * 1000 / millis as usize,
          );
        }
      }
      Message::NewGame => state.new_game(&position),
      Message::Perft(depth) => divide(&position, depth),
      Message::IsReady => println!("readyok"),
      Message::Clock(_) | Message::Info(_) => (),
    }
  }
}
