use liberty_chess::clock::format_time;
use liberty_chess::parsing::from_chars;
use liberty_chess::positions::{
  get_startpos, AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE,
  LIBERTY_CHESS, LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::{Board, ALL_PIECES};
use oxidation::evaluate::evaluate;
use oxidation::parameters::DEFAULT_PARAMETERS;
use oxidation::search::SEARCH_PARAMETERS;
use oxidation::{
  bench, divide, search, Output, SearchConfig, State, HASH_SIZE, MAX_QDEPTH, MULTI_PV_COUNT,
  QDEPTH, QDEPTH_NAME, VERSION_NUMBER,
};
use std::collections::{HashMap, HashSet};
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::{channel, Sender};
use std::thread::spawn;
use std::time::Instant;
use ulci::client::{startup, Message};
use ulci::{
  ClientInfo, IntOption, OptionValue, RangeOption, Score, SupportedFeatures, UlciOption, V1Features,
};

const BENCH_DEPTH: i8 = 9;

const HASH_NAME: &str = "Hash";
const MULTI_PV_NAME: &str = "MultiPV";
const VARIANT_NAME: &str = "UCI_Variant";

// i8 is an offset for bench depth
const BENCH_POSITIONS: &[(&str, i8)] = &[
  (STARTPOS, 0),
  (CAPABLANCA_RECTANGLE, 0),
  (CAPABLANCA, 0),
  (LIBERTY_CHESS, -1),
  (MINI, 2),
  (MONGOL, 0),
  (AFRICAN, -1),
  (NARNIA, 1),
  (TRUMP, -2),
  (LOADED_BOARD, -4),
  (DOUBLE_CHESS, -2),
  (HORDE, -1),
  (ELIMINATION, 0),
  ("4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1", 0),
];

fn startup_client(tx: &Sender<Message>) {
  let mut options = HashMap::new();
  options.insert(
    QDEPTH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: usize::from(QDEPTH),
      min: 0,
      max: usize::from(MAX_QDEPTH),
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
  options.insert(
    MULTI_PV_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: usize::from(MULTI_PV_COUNT),
      min: 1,
      max: 1 << 10,
    }),
  );
  let mut variants = HashSet::new();
  variants.insert("chess".to_owned());
  variants.insert("horde".to_owned());
  options.insert(
    VARIANT_NAME.to_owned(),
    UlciOption::Range(RangeOption {
      default: "chess".to_owned(),
      options: variants,
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
  let mut pv_lines = MULTI_PV_COUNT;
  let mut position = get_startpos();
  let mut state = State::new(hash_size, &position, SEARCH_PARAMETERS, DEFAULT_PARAMETERS);
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
        let searchmoves = settings.moves;
        let mut settings =
          SearchConfig::new_time(&position, &mut qdepth, settings.time, &rx, &mut debug);
        let pv = search(
          &mut state,
          &mut settings,
          &mut position,
          &searchmoves,
          pv_lines,
          Output::String(stdout()),
        );
        println!(
          "bestmove {}",
          pv.first().map_or("0000".to_string(), ToString::to_string)
        );
      }
      Message::Stop => {
        println!("info error not currently searching");
      }
      Message::UpdateOption(name, value) => match &*name {
        QDEPTH_NAME => match value {
          OptionValue::UpdateInt(value) => qdepth = value as u8,
          _ => println!("info error incorrect option type"),
        },
        HASH_NAME => match value {
          OptionValue::UpdateInt(value) => {
            if value != hash_size {
              hash_size = value;
              state = State::new(hash_size, &position, SEARCH_PARAMETERS, DEFAULT_PARAMETERS);
            }
          }
          _ => println!("info error incorrect option type"),
        },
        MULTI_PV_NAME => match value {
          OptionValue::UpdateInt(value) => {
            pv_lines = value as u16;
          }
          _ => println!("info error incorrect option type"),
        },
        // Does not do anything, just there for servers that expect it
        VARIANT_NAME => (),
        _ => (),
      },
      Message::Eval => {
        println!(
          "info score {}",
          Score::Centipawn(evaluate(&state, &position))
            .show_uci(position.moves(), position.to_move()),
        );
      }
      Message::Bench(depth) => {
        if depth < 5 {
          println!("info error minimum bench depth 5");
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
