#![forbid(unsafe_code)]
#![warn(missing_docs, unused)]
//! The infrastructure for the ULCI interface, both client and server

use liberty_chess::moves::Move;
use liberty_chess::{Board, CompressedBoard};
use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::io::BufRead;
use std::io::Write;
use std::sync::mpsc::Sender;
use std::time::Duration;

const VERSION: usize = 1;

/// The starting position
pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";

/// The functions tha need to be implemented for the ULCI interface
pub enum ClientMessage {
  /// The server wants to update the value of an option
  UpdateOption(OptionValue),
  /// The server is setting whether verbose output is enabled
  SetDebug(bool),
  /// The server has changed the current position
  UpdatePosition(Box<CompressedBoard>),
  /// The server wants to start a search
  Go(SearchSettings),
  /// The server wants to stop the search
  Stop,
}

/// The value of some option to update
pub enum OptionValue {
  // First string name in option is the option name, second parameter is the value
  /// The value of a string option
  UpdateString(String, String),
  /// The value of an integer option
  UpdateInt(String, usize),
  /// The value of a true/false option
  UpdateBool(String, bool),
  /// The value of an option from a range of possibilities
  UpdateRange(String, String),
  /// A trigger signal for the engine
  SendTrigger(String),
}

/// Settings for a search
pub struct SearchSettings {
  /// The available moves to search
  pub moves: Vec<Move>,
  /// The time control for searching
  pub time: SearchTime,
}

/// The time control for searching
pub enum SearchTime {
  /// Fixed time per move
  FixedTime(Duration),
  /// Time and increment
  Increment(Duration, Duration),
  /// Depth
  Depth(u16),
  /// Nodes
  Nodes(usize),
  /// Infinite
  Infinite,
}

/// An option supported by the client
pub enum UlciOption {
  /// A string option
  String(String),
  /// An integer option
  Int(IntOption),
  /// A true/false option
  Bool(bool),
  /// One of a range of possibilities
  Range(RangeOption),
  /// A trigger button to do something
  Trigger,
}

impl ToString for UlciOption {
  fn to_string(&self) -> String {
    match self {
      Self::String(option) => format!("type string default {option}"),
      Self::Int(option) => option.to_string(),
      Self::Bool(option) => format!("type check default {option}"),
      Self::Range(option) => option.to_string(),
      Self::Trigger => "type button".to_owned(),
    }
  }
}

/// An option with an integer value and optional min/max
pub struct IntOption {
  default: usize,
  min: Option<usize>,
  max: Option<usize>,
}

impl ToString for IntOption {
  fn to_string(&self) -> String {
    let mut result = format!("type spin default {}", self.default);
    if let Some(min) = self.min {
      result += &format!(" min {min}");
    }
    if let Some(max) = self.max {
      result += &format!(" max {max}");
    }
    result
  }
}

/// One of a range of possibilities
pub struct RangeOption {
  default: String,
  options: HashSet<String>,
}

impl ToString for RangeOption {
  fn to_string(&self) -> String {
    let mut result = format!("type combo default {}", self.default);
    for option in &self.options {
      result += &format!(" var {option}");
    }
    result
  }
}

/// The information required for the client
pub struct ClientInfo {
  /// The name of the client
  pub name: String,
  /// The username of a human player, `None` if computer
  pub username: Option<String>,
  /// The author of the client
  pub author: String,
  /// Options for the client
  pub options: HashMap<&'static str, UlciOption>,
}

fn write(writer: &mut impl Write, output: impl Display) {
  writer.write(format!("{output}\n").as_bytes()).ok();
}

fn print_uci(out: &mut impl Write, info: &ClientInfo) {
  write(out, &format!("id version {VERSION}"));
  write(out, &format!("id name {}", info.name));
  if let Some(ref name) = info.username {
    write(out, &format!("id username {name}"));
  }
  write(out, &format!("id author {}", info.author));
  for (name, option) in &info.options {
    write(out, &format!("option name {name} {}", option.to_string()));
  }
  write(out, "uciok");
}

fn get_board() -> Board {
  Board::new(STARTPOS).unwrap()
}

/// Set up a new client that handles some requirements locally and passes the rest on to the engine
/// Blocks the thread it runs on, should be spawned in a new thread
pub fn startup_client(
  client: Sender<ClientMessage>,
  info: ClientInfo,
  mut input: impl BufRead,
  mut out: impl Write,
) -> Option<()> {
  let mut debug = false;
  let mut buffer = String::new();
  let mut board = get_board();
  while let Ok(chars) = input.read_line(&mut buffer) {
    if chars == 0 {
      return None;
    }
    let mut words = buffer.split_whitespace();
    match words.next() {
      Some("uci") => print_uci(&mut out, &info),
      Some("debug") => match words.next() {
        Some("on") => {
          debug = true;
          client.send(ClientMessage::SetDebug(true)).ok()?;
        }
        Some("off") => {
          debug = false;
          client.send(ClientMessage::SetDebug(false)).ok()?;
        }
        Some(value) => {
          if debug {
            write(
              &mut out,
              &format!("info servererror Unrecognised debug setting {value}"),
            );
          }
        }
        None => {
          if debug {
            write(&mut out, "info servererror Missing debug setting");
          }
        }
      },
      Some("isready") => write(&mut out, "readyok"),
      Some("setoption") => {
        let mut malformed = true;
        if Some("name") == words.next() {
          let mut name_words = Vec::new();
          let mut value_words = Vec::new();
          let mut value_encountered = false;
          for word in words {
            if value_encountered {
              value_words.push(word);
            } else if word == "value" {
              value_encountered = true;
            } else {
              name_words.push(word);
            }
          }
          let name = name_words.join(" ");
          let value = value_words.join(" ");
          let borrow_name: &str = &name;
          if let Some(option) = info.options.get(borrow_name) {
            match option {
              UlciOption::String(_) => {
                client
                  .send(ClientMessage::UpdateOption(OptionValue::UpdateString(
                    name, value,
                  )))
                  .ok()?;
              }
              UlciOption::Int(option) => match value.parse::<usize>() {
                Ok(mut value) => {
                  if let Some(min) = option.min {
                    value = value.max(min)
                  }
                  if let Some(max) = option.max {
                    value = value.min(max)
                  }
                  client
                    .send(ClientMessage::UpdateOption(OptionValue::UpdateInt(
                      name, value,
                    )))
                    .ok()?;
                }
                Err(_) => {
                  if debug {
                    write(
                      &mut out,
                      &format!("info servererror {value} is not an integer"),
                    );
                  }
                }
              },
              UlciOption::Bool(_) => match value.parse() {
                Ok(value) => {
                  client
                    .send(ClientMessage::UpdateOption(OptionValue::UpdateBool(
                      name, value,
                    )))
                    .ok()?;
                }
                Err(_) => {
                  if debug {
                    write(
                      &mut out,
                      &format!("info servererror {value} is not a boolean"),
                    );
                  }
                }
              },
              UlciOption::Range(option) => {
                if option.options.contains(&value) {
                  client
                    .send(ClientMessage::UpdateOption(OptionValue::UpdateRange(
                      name, value,
                    )))
                    .ok()?;
                } else if debug {
                  write(
                    &mut out,
                    &format!("info servererror option {name} has no value {value}"),
                  );
                }
              }
              UlciOption::Trigger => {
                client
                  .send(ClientMessage::UpdateOption(OptionValue::SendTrigger(name)))
                  .ok()?;
              }
            }
            malformed = false;
          } else if debug {
            write(
              &mut out,
              &format!("info servererror unrecognised option {name}"),
            );
            malformed = false;
          }
        }
        if malformed && debug {
          write(&mut out, "info servererror malformed setoption command");
        }
      }
      Some("position") => {
        board = match words.next() {
          Some("startpos") => get_board(),
          Some("fen") => {
            let mut fen = String::new();
            for word in words.by_ref() {
              if word == "moves" {
                break;
              } else if fen.is_empty() {
                fen += word;
              } else {
                fen += &format!(" {word}");
              }
            }
            if let Ok(board) = Board::new(&fen) {
              board
            } else {
              if debug {
                write(
                  &mut out,
                  &format!("info servererror invalid position {fen}"),
                );
              }
              buffer.clear();
              continue;
            }
          }
          Some(_) | None => {
            if debug {
              write(&mut out, "info servererror malformed position command");
            }
            buffer.clear();
            continue;
          }
        };
        for word in words {
          if word != "moves" {
            if let Ok(candidate_move) = word.parse() {
              if let Some(new_board) = board.move_if_legal(candidate_move) {
                board = new_board;
              } else if debug {
                write(
                  &mut out,
                  &format!(
                    "info servererror illegal move {} from {}",
                    candidate_move.to_string(),
                    board.to_string()
                  ),
                );
              }
            } else if debug {
              write(
                &mut out,
                &format!("info servererror invalid move {}", word,),
              );
            }
          }
        }
        if debug {
          write(
            &mut out,
            &format!("info string position changed to {}", board.to_string()),
          );
        }
        client
          .send(ClientMessage::UpdatePosition(Box::new(
            board.send_to_thread(),
          )))
          .ok()?;
      }
      // TODO: fix searchmoves
      // it currently has to be specified after setting the search time
      Some("go") => {
        let mut time = SearchTime::Infinite;
        while let Some(word) = words.next() {
          match word {
            "infinite" => time = SearchTime::Infinite,
            "depth" => {
              if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                time = SearchTime::Depth(value)
              } else if debug {
                write(&mut out, "info servererror no depth specified");
              }
            }
            "mate" => {
              if let Some(value) = words.next().and_then(|w| w.parse::<u16>().ok()) {
                time = SearchTime::Depth(value * 2)
              } else if debug {
                write(&mut out, "info servererror no move count specified");
              }
            }
            "nodes" => {
              if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                time = SearchTime::Nodes(value)
              } else if debug {
                write(&mut out, "info servererror no node count specified");
              }
            }
            "movetime" => {
              if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                time = SearchTime::FixedTime(Duration::from_millis(value))
              } else if debug {
                write(&mut out, "info servererror no time specified");
              }
            }
            "wtime" => {
              if board.to_move() {
                if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                  let new_time = Duration::from_millis(value);
                  if let SearchTime::Increment(ref mut time, _) = time {
                    *time = new_time;
                  } else {
                    time = SearchTime::Increment(new_time, Duration::ZERO);
                  }
                } else if debug {
                  write(&mut out, "info servererror no time specified");
                }
              }
            }
            "btime" => {
              if !board.to_move() {
                if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                  let new_time = Duration::from_millis(value);
                  if let SearchTime::Increment(ref mut time, _) = time {
                    *time = new_time;
                  } else {
                    time = SearchTime::Increment(new_time, Duration::ZERO);
                  }
                } else if debug {
                  write(&mut out, "info servererror no time specified");
                }
              }
            }
            "winc" => {
              if board.to_move() {
                if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                  let new_inc = Duration::from_millis(value);
                  if let SearchTime::Increment(_, ref mut inc) = time {
                    *inc = new_inc;
                  } else {
                    time = SearchTime::Increment(Duration::from_secs(1), new_inc)
                  }
                } else if debug {
                  write(&mut out, "info servererror no time specified");
                }
              }
            }
            "binc" => {
              if !board.to_move() {
                if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
                  let new_inc = Duration::from_millis(value);
                  if let SearchTime::Increment(_, ref mut inc) = time {
                    *inc = new_inc;
                  } else {
                    time = SearchTime::Increment(Duration::from_secs(1), new_inc)
                  }
                } else if debug {
                  write(&mut out, "info servererror no time specified");
                }
              }
            }
            "searchmoves" => break,
            _ => {
              if debug {
                write(&mut out, "info servererror unknown go parameter");
              }
            }
          }
        }
        let mut moves = Vec::new();
        for word in words {
          if let Ok(r#move) = word.parse() {
            moves.push(r#move);
          }
        }
        client
          .send(ClientMessage::Go(SearchSettings { moves, time }))
          .ok();
      }
      Some("stop") => client.send(ClientMessage::Stop).ok()?,
      // Commands that can be ignored
      Some("ucinewgame") | Some("info") => (),
      // End the program, the channel being dropped will stop the other thread
      Some("quit") => break,
      // Unrecognised command, log when in debug mode
      Some(command) => {
        if debug {
          write(
            &mut out,
            &format!("info servererror Unrecognised command {command}"),
          );
        }
      }
      // Blank line
      None => (),
    }
    buffer.clear();
  }
  None
}
