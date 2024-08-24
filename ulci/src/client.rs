use crate::server::UlciResult;
use crate::{
  process_info, write, AnalysisResult, OptionValue, SearchSettings, SearchTime, UlciOption,
  V1Features,
};
use crate::{ClientInfo, Limits};
use liberty_chess::parsing::to_char;
use liberty_chess::positions::get_startpos;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::Board;
use std::io::BufRead;
use std::io::Write;
use std::str::SplitWhitespace;
use std::sync::mpsc::Sender;

/// The functions tha need to be implemented for the ULCI interface
pub enum Message {
  /// The server wants to update the value of an option
  UpdateOption(String, OptionValue),
  /// The server is setting whether verbose output is enabled
  SetDebug(bool),
  /// The server has changed the current position
  UpdatePosition(Box<CompressedBoard>),
  /// The server wants to start a search
  Go(SearchSettings),
  /// The server wants to stop the search
  Stop,
  /// The server wants a static evaluation of the position
  Eval,
  /// The server wants the standardised bench results
  Bench(i8),
  /// Clear the TT
  NewGame,
  /// Perft
  Perft(usize),
  /// The server is updating the clock state
  Clock(SearchTime),
  /// The server has some info
  Info(AnalysisResult),
  /// Respond with ReadyOk
  IsReady,
}

fn print_uci(out: &mut impl Write, info: &ClientInfo) -> Option<()> {
  let v1_features = info.features.v1;
  if v1_features == V1Features::all() {
    write(out, "id version 1")?;
  } else {
    if v1_features.board_sizes {
      write(out, "id feature boardsize")?;
    }
    if v1_features.pawn_moves {
      write(out, "id feature pawnmoves")?;
    }
    if v1_features.castling {
      write(out, "id feature castling")?;
    }
    if v1_features.multiple_kings {
      write(out, "id feature multplekings")?;
    }
    if v1_features.promotion_options {
      write(out, "id feature promotion")?;
    }
    if v1_features.friendly_fire {
      write(out, "id feature friendlyfire")?;
    }
  }
  write(
    out,
    format!(
      "id pieces {}",
      info.pieces.iter().map(|p| to_char(-*p)).collect::<String>()
    ),
  )?;
  write(out, format!("id name {}", info.name))?;
  if let Some(ref name) = info.username {
    write(out, format!("id username {name}"))?;
  }
  write(out, format!("id author {}", info.author))?;
  for (name, option) in &info.options {
    write(out, format!("option name {name} {}", option.to_string()))?;
  }
  write(out, "uciok")?;
  Some(())
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
      write(
        out,
        format!("info error Unrecognised debug setting {value}"),
      )?;
      Some(())
    }
    None => {
      write(out, "info error Missing debug setting")?;
      Some(())
    }
  }
}

fn setoption(
  out: &mut impl Write,
  client: &Sender<Message>,
  mut words: SplitWhitespace,
  info: &ClientInfo,
) -> Option<()> {
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
            .send(Message::UpdateOption(
              name,
              OptionValue::UpdateString(value),
            ))
            .ok()?;
        }
        UlciOption::Int(option) => match value.parse::<usize>() {
          Ok(mut value) => {
            value = value.max(option.min);
            value = value.min(option.max);
            client
              .send(Message::UpdateOption(name, OptionValue::UpdateInt(value)))
              .ok()?;
          }
          Err(_) => {
            write(out, format!("info error {value} is not an integer"))?;
          }
        },
        UlciOption::Bool(_) => match value.parse() {
          Ok(value) => {
            client
              .send(Message::UpdateOption(name, OptionValue::UpdateBool(value)))
              .ok()?;
          }
          Err(_) => {
            write(out, format!("info error {value} is not a boolean"))?;
          }
        },
        UlciOption::Range(option) => {
          if option.options.contains(&value) {
            client
              .send(Message::UpdateOption(name, OptionValue::UpdateRange(value)))
              .ok()?;
          } else {
            write(
              out,
              format!("info error option {name} has no value {value}"),
            )?;
          }
        }
        UlciOption::Trigger => {
          client
            .send(Message::UpdateOption(name, OptionValue::SendTrigger))
            .ok()?;
        }
      }
    } else {
      write(out, format!("info error unrecognised option {name}"))?;
    }
  } else {
    write(out, "info error malformed setoption command")?;
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
        write(out, format!("info error invalid position {fen}"))?;
        // Fatal error, quit the program
        return if !debug { None } else { Some(()) };
      }
    }
    Some(_) | None => {
      write(out, "info error malformed position command")?;
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
            format!(
              "info error illegal move {} from {}",
              candidate_move.to_string(),
              board.to_string()
            ),
          )?;
          // Fatal error, quit the program
          return if !debug { None } else { Some(()) };
        }
      } else {
        write(out, format!("info error invalid move {word}"))?;
        // Fatal error, quit the program
        return if !debug { None } else { Some(()) };
      }
    }
  }
  if debug {
    write(
      out,
      format!("info string position changed to {}", board.to_string()),
    )?;
  }
  client
    .send(Message::UpdatePosition(Box::new(board.send_to_thread())))
    .ok()
}

fn go(
  out: &mut impl Write,
  client: &Sender<Message>,
  mut words: SplitWhitespace,
  debug: bool,
) -> Option<()> {
  let mut time = SearchTime::Infinite;
  while let Some(word) = words.next() {
    match word {
      "infinite" => time = SearchTime::Infinite,
      "depth" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          let depth = usize::from(u8::MAX).min(value);
          let mut limits = if let SearchTime::Other(limits) = time {
            limits
          } else {
            Limits::default()
          };
          limits.depth = depth as u8;
          time = SearchTime::Other(limits);
        } else {
          write(out, "info error no depth specified")?;
        }
      }
      "mate" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          let moves = u32::from(u8::MAX).min(value);
          time = SearchTime::Mate(moves);
        } else {
          write(out, "info error no move count specified")?;
        }
      }
      "nodes" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          let mut limits = if let SearchTime::Other(limits) = time {
            limits
          } else {
            Limits::default()
          };
          limits.nodes = value;
          time = SearchTime::Other(limits);
        } else {
          write(out, "info error no node count specified")?;
        }
      }
      "movetime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          let mut limits = if let SearchTime::Other(limits) = time {
            limits
          } else {
            Limits::default()
          };
          limits.time = value;
          time = SearchTime::Other(limits);
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "wtime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(ref mut wtime, _, _, _) = time {
            *wtime = value;
          } else {
            time = SearchTime::Asymmetric(value, 0, 1000, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "btime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, _, ref mut btime, _) = time {
            *btime = value;
          } else {
            time = SearchTime::Asymmetric(1000, 0, value, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "winc" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, ref mut winc, _, _) = time {
            *winc = value;
          } else {
            time = SearchTime::Asymmetric(1000, value, 1000, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "binc" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, _, _, ref mut binc) = time {
            *binc = value;
          } else {
            time = SearchTime::Asymmetric(1000, 0, 1000, value);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "searchmoves" => break,
      _ => {
        write(out, format!("info error unknown go parameter {word}"))?;
      }
    }
  }
  let mut moves = Vec::new();
  for word in words {
    if let Ok(mv) = word.parse() {
      moves.push(mv);
    } else {
      write(out, "info error invalid move specified")?;
      // Fatal error, quit the program
      return if !debug { None } else { Some(()) };
    }
  }
  client
    .send(Message::Go(SearchSettings { moves, time }))
    .ok()
}

fn clock(out: &mut impl Write, client: &Sender<Message>, mut words: SplitWhitespace) -> Option<()> {
  let mut time = SearchTime::Infinite;
  while let Some(word) = words.next() {
    match word {
      "wtime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(ref mut wtime, _, _, _) = time {
            *wtime = value;
          } else {
            time = SearchTime::Asymmetric(value, 0, 1000, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "btime" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, _, ref mut btime, _) = time {
            *btime = value;
          } else {
            time = SearchTime::Asymmetric(1000, 0, value, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "winc" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, ref mut winc, _, _) = time {
            *winc = value;
          } else {
            time = SearchTime::Asymmetric(1000, value, 1000, 0);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      "binc" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          if let SearchTime::Asymmetric(_, _, _, ref mut binc) = time {
            *binc = value;
          } else {
            time = SearchTime::Asymmetric(1000, 0, 1000, value);
          }
        } else {
          write(out, "info error no time specified")?;
        }
      }
      _ => {
        write(out, "info error unknown clock parameter")?;
      }
    }
  }
  client.send(Message::Clock(time)).ok()
}

/// Set up a new client that handles some requirements locally and passes the rest on to the engine
///
/// Blocks the thread it runs on, should be spawned in a new thread
pub fn startup(
  client: &Sender<Message>,
  info: &ClientInfo,
  mut input: impl BufRead,
  mut out: impl Write,
  is_always_ready: bool,
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
      Some("uci") => print_uci(&mut out, info)?,
      Some("debug") => process_debug(&mut out, client, words, &mut debug)?,
      Some("isready") => {
        if is_always_ready {
          write(&mut out, "readyok")?;
        } else {
          client.send(Message::IsReady).ok()?;
        }
      }
      Some("setoption") => setoption(&mut out, client, words, info)?,
      Some("position") => position(&mut out, client, &mut board, words, debug)?,
      Some("go") => go(&mut out, client, words, debug)?,
      Some("stop") => client.send(Message::Stop).ok()?,
      Some("eval") => client.send(Message::Eval).ok()?,
      Some("ucinewgame") => client.send(Message::NewGame).ok()?,
      Some("perft") => {
        let depth = words
          .next()
          .and_then(|w| w.parse().ok())
          .unwrap_or(1)
          .max(1);
        client.send(Message::Perft(depth)).ok()?;
      }
      Some("bench") => {
        let depth = words
          .next()
          .and_then(|w| w.parse().ok())
          .unwrap_or(info.depth);
        client.send(Message::Bench(depth)).ok()?;
      }
      Some("clock") => clock(&mut out, client, words)?,
      // End the program, the channel being dropped will stop the other thread
      Some("quit") => break,
      // Commands that can be ignored or blank line
      Some("info") => {
        for message in process_info(words) {
          if let UlciResult::Analysis(result) = message {
            client.send(Message::Info(result)).ok();
          }
        }
      }
      None => (),
      // Unrecognised command
      Some(command) => {
        write(
          &mut out,
          format!("info error Unrecognised command {command}"),
        )?;
      }
    }
    buffer.clear();
  }
  None
}
