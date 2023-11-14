use liberty_chess::moves::Move;
use liberty_chess::parsing::from_chars;
use liberty_chess::positions::get_startpos;
use liberty_chess::ALL_PIECES;
use oxidation::{search, SearchConfig, QDEPTH_NAME};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::channel;
use std::thread::spawn;
use ulci::client::{startup, Message};
use ulci::{ClientInfo, IntOption, OptionValue, SearchTime, UlciOption};

const QDEPTH: u8 = 3;

fn main() {
  let (tx, rx) = channel();
  let mut options = HashMap::new();
  options.insert(
    QDEPTH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: QDEPTH as usize,
      min: 0,
      max: usize::from(u8::MAX),
    }),
  );
  let info = ClientInfo {
    name: "Oxidation".to_owned(),
    username: None,
    author: "Mathmagician".to_owned(),
    options,
    pieces: from_chars(ALL_PIECES),
  };
  let mut qdepth = QDEPTH;
  let input = BufReader::new(stdin());
  let output = stdout();
  let mut position = get_startpos();
  let mut debug = false;
  spawn(move || startup(&tx, &info, input, output));
  while let Ok(message) = rx.recv() {
    match message {
      Message::SetDebug(new_debug) => debug = new_debug,
      Message::UpdatePosition(board) => position = board.load_from_thread(),
      Message::Go(settings) => {
        let (capture, other) = position.generate_legal_buckets();
        let capture = capture.iter().filter_map(|board| board.last_move);
        let other = other.iter().filter_map(|board| board.last_move);
        let (mut capture, mut other): (Vec<Move>, Vec<Move>) = if settings.moves.is_empty() {
          (capture.collect(), other.collect())
        } else {
          (
            capture.filter(|m| settings.moves.contains(m)).collect(),
            other.filter(|m| settings.moves.contains(m)).collect(),
          )
        };
        if capture.is_empty() && other.is_empty() {
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
          capture.shuffle(&mut thread_rng());
          other.shuffle(&mut thread_rng());
          let mut moves = capture;
          moves.append(&mut other);
          let settings = match settings.time {
            SearchTime::FixedTime(time) => SearchConfig::new(
              &mut qdepth,
              u8::MAX,
              time.as_millis(),
              usize::MAX,
              &rx,
              &mut debug,
            ),
            SearchTime::Increment(time, inc) => {
              let time = time.as_millis().saturating_sub(100);
              let time = time.min(time / 20 + inc.as_millis() / 2);
              let time = 1.max(time);
              SearchConfig::new(&mut qdepth, u8::MAX, time, usize::MAX, &rx, &mut debug)
            }
            SearchTime::Nodes(nodes) => {
              SearchConfig::new(&mut qdepth, u8::MAX, u128::MAX, nodes, &rx, &mut debug)
            }
            SearchTime::Depth(max_depth) => SearchConfig::new(
              &mut qdepth,
              max_depth,
              u128::MAX,
              usize::MAX,
              &rx,
              &mut debug,
            ),
            SearchTime::Infinite => {
              SearchConfig::new(&mut qdepth, u8::MAX, u128::MAX, usize::MAX, &rx, &mut debug)
            }
          };
          let pv = search(settings, &position, moves);
          println!("bestmove {}", pv[0].to_string());
        }
      }
      Message::Stop => {
        println!("info string servererror not currently searching");
      }
      Message::UpdateOption(name, value) => {
        if name == QDEPTH_NAME {
          match value {
            OptionValue::UpdateInt(value) => qdepth = value as u8,
            OptionValue::SendTrigger
            | OptionValue::UpdateBool(_)
            | OptionValue::UpdateRange(_)
            | OptionValue::UpdateString(_) => {
              if debug {
                println!("info string servererror incorrect option type");
              }
            }
          }
        }
      }
    }
  }
}
