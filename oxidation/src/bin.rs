use liberty_chess::parsing::from_chars;
use liberty_chess::positions::get_startpos;
use liberty_chess::ALL_PIECES;
use oxidation::{get_move_order, search, SearchConfig, QDEPTH, QDEPTH_NAME, HASH_SIZE, HASH_NAME, State};
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::channel;
use std::thread::spawn;
use ulci::client::{startup, Message};
use ulci::{ClientInfo, IntOption, OptionValue, UlciOption};

fn main() {
  let (tx, rx) = channel();
  let mut options = HashMap::new();
  options.insert(
    QDEPTH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: usize::from(QDEPTH),
      min: 0,
      max: usize::from(u8::MAX),
    }),
  );
  options.insert(
    HASH_NAME.to_owned(),
    UlciOption::Int(IntOption {
      default: HASH_SIZE,
      min: 0,
      max: 1 << 32,
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
  let mut hash_size = HASH_SIZE;
  let mut state = State::new(hash_size);
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
        let moves = get_move_order(&position, &settings.moves);
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
          let settings = SearchConfig::new_time(&mut qdepth, settings.time, &rx, &mut debug);
          let pv = search(&mut state, settings, &position, moves, Some(stdout()));
          println!("bestmove {}", pv[0].to_string());
        }
      }
      Message::Stop => {
        println!("info string servererror not currently searching");
      }
      Message::UpdateOption(name, value) => {
        match &*name {
          QDEPTH_NAME => {
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
          HASH_NAME => {
            match value {
              OptionValue::UpdateInt(value) => {
                if value != hash_size {
                  hash_size = value;
                  state = State::new(hash_size);
                }
              }
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
          _ => (),
        }
      }
    }
  }
}
