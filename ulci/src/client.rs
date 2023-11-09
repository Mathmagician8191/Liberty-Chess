use crate::ClientInfo;
use crate::{write, OptionValue, SearchSettings, SearchTime, UlciOption, VERSION};
use liberty_chess::positions::get_startpos;
use liberty_chess::{Board, CompressedBoard};
use std::io::BufRead;
use std::io::Write;
use std::str::SplitWhitespace;
use std::sync::mpsc::Sender;
use std::time::Duration;

/// The functions tha need to be implemented for the ULCI interface
pub enum Message {
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

fn process_debug(
  out: &mut impl Write,
  client: &Sender<Message>,
  mut words: SplitWhitespace,
  debug: &mut bool,
) -> Option<()> {
  match words.next() {
    Some("on") => {
      *debug = true;
      client.send(Message::SetDebug(true)).ok()
    }
    Some("off") => {
      *debug = false;
      client.send(Message::SetDebug(false)).ok()
    }
    Some(value) => {
      if *debug {
        write(
          out,
          &format!("info string servererror Unrecognised debug setting {value}"),
        );
      }
      Some(())
    }
    None => {
      if *debug {
        write(out, "info string servererror Missing debug setting");
      }
      Some(())
    }
  }
}

fn setoption(
  out: &mut impl Write,
  client: &Sender<Message>,
  mut words: SplitWhitespace,
  debug: bool,
  info: &ClientInfo,
) -> Option<()> {
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
            .send(Message::UpdateOption(OptionValue::UpdateString(
              name, value,
            )))
            .ok()?;
        }
        UlciOption::Int(option) => match value.parse::<usize>() {
          Ok(mut value) => {
            if let Some(min) = option.min {
              value = value.max(min);
            }
            if let Some(max) = option.max {
              value = value.min(max);
            }
            client
              .send(Message::UpdateOption(OptionValue::UpdateInt(name, value)))
              .ok()?;
          }
          Err(_) => {
            if debug {
              write(
                out,
                &format!("info string servererror {value} is not an integer"),
              );
            }
          }
        },
        UlciOption::Bool(_) => match value.parse() {
          Ok(value) => {
            client
              .send(Message::UpdateOption(OptionValue::UpdateBool(name, value)))
              .ok()?;
          }
          Err(_) => {
            if debug {
              write(
                out,
                &format!("info string servererror {value} is not a boolean"),
              );
            }
          }
        },
        UlciOption::Range(option) => {
          if option.options.contains(&value) {
            client
              .send(Message::UpdateOption(OptionValue::UpdateRange(name, value)))
              .ok()?;
          } else if debug {
            write(
              out,
              &format!("info string servererror option {name} has no value {value}"),
            );
          }
        }
        UlciOption::Trigger => {
          client
            .send(Message::UpdateOption(OptionValue::SendTrigger(name)))
            .ok()?;
        }
      }
      malformed = false;
    } else if debug {
      write(
        out,
        &format!("info string servererror unrecognised option {name}"),
      );
      malformed = false;
    }
  }
  if malformed && debug {
    write(out, "info string servererror malformed setoption command");
  }
  Some(())
}

fn position(
  out: &mut impl Write,
  client: &Sender<Message>,
  board: &mut Board,
  mut words: SplitWhitespace,
  debug: bool,
) -> Option<()> {
  *board = match words.next() {
    Some("startpos") => get_startpos(),
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
        write(
          out,
          &format!("info string servererror invalid position {fen}"),
        );
        // Fatal error, quit the program
        return None;
      }
    }
    Some(_) | None => {
      if debug {
        write(out, "info string servererror malformed position command");
      }
      return Some(());
    }
  };
  for word in words {
    if word != "moves" {
      if let Ok(candidate_move) = word.parse() {
        if let Some(new_board) = board.move_if_legal(candidate_move) {
          *board = new_board;
        } else {
          write(
            out,
            &format!(
              "info string servererror illegal move {} from {}",
              candidate_move.to_string(),
              board.to_string()
            ),
          );
          // Fatal error, quit the program
          return None;
        }
      } else {
        write(out, &format!("info string servererror invalid move {word}"));
        // Fatal error, quit the program
        return None;
      }
    }
  }
  if debug {
    write(
      out,
      &format!("info string position changed to {}", board.to_string()),
    );
  }
  client
    .send(Message::UpdatePosition(Box::new(board.send_to_thread())))
    .ok()
}

fn go(
  out: &mut impl Write,
  client: &Sender<Message>,
  board: &Board,
  mut words: SplitWhitespace,
  debug: bool,
) -> Option<()> {
  let mut time = SearchTime::Infinite;
  while let Some(word) = words.next() {
    match word {
      "infinite" => time = SearchTime::Infinite,
      "depth" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          time = SearchTime::Depth(value);
        } else if debug {
          write(out, "info string servererror no depth specified");
        }
      }
      "mate" => {
        if let Some(value) = words.next().and_then(|w| w.parse::<u16>().ok()) {
          time = SearchTime::Depth(value * 2);
        } else if debug {
          write(out, "info string servererror no move count specified");
        }
      }
      "nodes" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          time = SearchTime::Nodes(value);
        } else if debug {
          write(out, "info string servererror no node count specified");
        }
      }
      "movetime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          time = SearchTime::FixedTime(Duration::from_millis(value));
        } else if debug {
          write(out, "info string servererror no time specified");
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
            write(out, "info string servererror no time specified");
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
            write(out, "info string servererror no time specified");
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
              time = SearchTime::Increment(Duration::from_secs(1), new_inc);
            }
          } else if debug {
            write(out, "info string servererror no time specified");
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
              time = SearchTime::Increment(Duration::from_secs(1), new_inc);
            }
          } else if debug {
            write(out, "info string servererror no time specified");
          }
        }
      }
      "searchmoves" => break,
      _ => {
        if debug {
          write(out, "info string servererror unknown go parameter");
        }
      }
    }
  }
  let mut moves = Vec::new();
  for word in words {
    if let Ok(r#move) = word.parse() {
      moves.push(r#move);
    } else {
      write(out, "info string servererror invalid move specified");
      // Fatal error, quit the program
      return None;
    }
  }
  client
    .send(Message::Go(SearchSettings { moves, time }))
    .ok()
}

/// Set up a new client that handles some requirements locally and passes the rest on to the engine
///
/// Blocks the thread it runs on, should be spawned in a new thread
pub fn startup(
  client: &Sender<Message>,
  info: &ClientInfo,
  mut input: impl BufRead,
  mut out: impl Write,
) -> Option<()> {
  let mut debug = false;
  let mut buffer = String::new();
  let mut board = get_startpos();
  while let Ok(chars) = input.read_line(&mut buffer) {
    if chars == 0 {
      return None;
    }
    let mut words = buffer.split_whitespace();
    match words.next() {
      Some("uci") => print_uci(&mut out, info),
      Some("debug") => process_debug(&mut out, client, words, &mut debug)?,
      Some("isready") => write(&mut out, "readyok"),
      Some("setoption") => setoption(&mut out, client, words, debug, info)?,
      Some("position") => position(&mut out, client, &mut board, words, debug)?,
      Some("go") => go(&mut out, client, &board, words, debug)?,
      Some("stop") => client.send(Message::Stop).ok()?,
      // End the program, the channel being dropped will stop the other thread
      Some("quit") => break,
      // Commands that can be ignored or blank line
      Some("ucinewgame" | "info") | None => (),
      // Unrecognised command, log when in debug mode
      Some(command) => {
        if debug {
          write(
            &mut out,
            &format!("info string servererror Unrecognised command {command}"),
          );
        }
      }
    }
    buffer.clear();
  }
  None
}
