use crate::{
  convert_words, process_info, write, write_mutex, AnalysisResult, ClientInfo, IntOption,
  OptionValue, RangeOption, Score, SearchTime, SupportedFeatures, UlciOption, V1Features,
};
use liberty_chess::moves::Move;
use liberty_chess::parsing::to_piece;
use liberty_chess::{BISHOP, KING, KNIGHT, PAWN, QUEEN, ROOK};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::spawn;

/// A request for some ULCI action
#[derive(Clone)]
pub enum Request {
  /// The server needs some analysis from the client
  Analysis(AnalysisRequest),
  /// Stop the analysis
  StopAnalysis,
  /// The server wants to show the client a new position
  Position(String, Vec<Move>, bool),
  /// The server wants to update an option
  SetOption(String, OptionValue),
  /// The server is updating the clock
  Clock(SearchTime),
  /// The server has results for the client
  AnalysisResult(AnalysisResult),
}

/// A request for analysis
#[derive(Clone)]
pub struct AnalysisRequest {
  /// The base position to analyse
  pub fen: String,
  /// Moves from the base position to the current position
  pub moves: Vec<Move>,
  /// The time to analyse for
  pub time: SearchTime,
  /// Which moves to analyse (empty Vec = analyse all)
  pub searchmoves: Vec<Move>,
  /// Should ucinewgame be sent
  pub new_game: bool,
}

/// The results from the client
pub enum UlciResult {
  /// Analysis results
  Analysis(AnalysisResult),
  /// Analysis is over, return bestmove
  AnalysisStopped(Move),
  /// The client is ready, send client info
  Startup(ClientInfo),
  /// Information for the server
  Info(InfoType, String),
}

impl Default for AnalysisResult {
  fn default() -> Self {
    Self {
      pv: Vec::new(),
      score: Score::Centipawn(0),
      depth: 1,
      nodes: 1,
      time: 0,
      wdl: None,
      pv_line: 1,
    }
  }
}

/// The type of info sent by the client
pub enum InfoType {
  /// A string message
  String,
  /// The client is reporting an error
  Error,
}

/// Start up a ULCI server
///
/// Has limited error handling
///
/// Blocks the current thread
pub fn startup_server(
  requests: Receiver<Request>,
  results: &Sender<UlciResult>,
  mut input: impl BufRead,
  mut out: impl Write + Send + 'static,
  debug: bool,
  // To make the GUI work without polling or creating more threads
  completion: impl Fn(),
) -> Option<()> {
  let mut buffer = String::new();
  let client_info = setup(results, &mut input, &mut out, debug, &mut buffer)?;
  results.send(UlciResult::Startup(client_info)).ok();
  completion();
  let (tx, rx) = channel();
  let out = Arc::new(Mutex::new(out));
  let new_out = out.clone();
  spawn(move || process_server(&requests, &tx, &new_out));
  process_analysis(&rx, results, input, &out, buffer, completion)
}

fn setup(
  results: &Sender<UlciResult>,
  input: &mut impl BufRead,
  out: &mut (impl Write + Send + 'static),
  debug: bool,
  buffer: &mut String,
) -> Option<ClientInfo> {
  write(out, "uci")?;
  let mut features = SupportedFeatures::default();
  let mut pieces = vec![PAWN, KNIGHT, BISHOP, ROOK, QUEEN, KING];
  let mut name = String::new();
  let mut username = None;
  let mut author = String::new();
  let mut options = HashMap::new();
  while let Ok(chars) = input.read_line(buffer) {
    if chars == 0 {
      return None;
    }
    let mut words = buffer.split_whitespace();
    match words.next() {
      Some("id") => match words.next() {
        Some("version") => {
          if let Some(version) = words.next().and_then(|w| w.parse::<u32>().ok()) {
            if version >= 1 {
              features.v1 = V1Features::all();
            }
          }
        }
        Some("feature") => {
          if let Some(word) = words.next() {
            match word {
              "boardsize" => features.v1.board_sizes = true,
              "pawnconfig" => features.v1.pawn_moves = true,
              "castling" => features.v1.castling = true,
              "multiplekings" => features.v1.multiple_kings = true,
              "promotion" => features.v1.promotion_options = true,
              "friendlyfire" => features.v1.friendly_fire = true,
              _ => (),
            }
          }
        }
        Some("pieces") => {
          if let Some(word) = words.next() {
            pieces = word.chars().flat_map(to_piece).map(i8::abs).collect();
          }
        }
        Some("name") => name = convert_words(words),
        Some("username") => username = Some(convert_words(words)),
        Some("author") => author = convert_words(words),
        Some(_) | None => (),
      },
      Some("option") => {
        if words.next() == Some("name") {
          let mut name_words = Vec::new();
          for word in words.by_ref() {
            if word == "type" {
              break;
            }
            name_words.push(word);
          }
          let option_name = name_words.join(" ");
          if let Some(word) = words.next() {
            match word {
              "check" => {
                let default = words.next() == Some("default") && words.next() == Some("true");
                options.insert(option_name, UlciOption::Bool(default));
              }
              "spin" => {
                let mut default = None;
                let mut min = None;
                let mut max = None;
                while let Some(word) = words.next() {
                  match word {
                    "default" => default = words.next().and_then(|w| w.parse().ok()).or(default),
                    "min" => min = words.next().and_then(|w| w.parse().ok()).or(min),
                    "max" => max = words.next().and_then(|w| w.parse().ok()).or(max),
                    _ => (),
                  }
                }
                if let Some(default) = default {
                  options.insert(
                    option_name,
                    UlciOption::Int(IntOption {
                      default,
                      min: min.unwrap_or(usize::MIN),
                      max: max.unwrap_or(usize::MAX),
                    }),
                  );
                }
              }
              // Limitation: default has to be the first argument
              "combo" => {
                if words.next() == Some("default") {
                  let mut default = String::new();
                  for word in words.by_ref() {
                    if word == "var" {
                      break;
                    }
                    default += word;
                  }
                  let mut choices = HashSet::new();
                  loop {
                    let mut option = String::new();
                    for word in words.by_ref() {
                      if word == "var" {
                        break;
                      }
                      option += word;
                    }
                    if option.is_empty() {
                      break;
                    }
                    choices.insert(option);
                  }
                  options.insert(
                    option_name,
                    UlciOption::Range(RangeOption {
                      default,
                      options: choices,
                    }),
                  );
                }
              }
              "button" => {
                options.insert(option_name, UlciOption::Trigger);
              }
              "string" => {
                if words.next() == Some("default") {
                  options.insert(option_name, UlciOption::String(convert_words(words)));
                }
              }
              _ => (),
            }
          }
        }
      }
      Some("info") => {
        for message in process_info(words) {
          results.send(message).ok();
        }
      }
      Some("uciok") => break,
      Some(_) | None => (),
    }
    buffer.clear();
  }
  buffer.clear();
  if debug {
    write(out, "debug on")?;
  }
  Some(ClientInfo {
    features,
    name,
    username,
    author,
    options,
    pieces,
    // not relevant for the server
    depth: 0,
  })
}

fn process_server(
  requests: &Receiver<Request>,
  tx: &Sender<AnalysisRequest>,
  out: &Arc<Mutex<impl Write>>,
) -> Option<()> {
  while let Ok(request) = requests.recv() {
    match request {
      Request::Analysis(request) => {
        tx.send(request).ok()?;
      }
      Request::StopAnalysis => {
        write_mutex(out, "stop")?;
      }
      Request::Position(fen, moves, newgame) => {
        if newgame {
          write_mutex(out, "ucinewgame")?;
        }
        let mut output = format!("position fen {fen}");
        if !moves.is_empty() {
          output += " moves ";
          output += &moves
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(" ");
        }
        write_mutex(out, output)?;
      }
      Request::SetOption(name, option) => {
        write_mutex(
          out,
          match option {
            OptionValue::UpdateString(value) => {
              format!("setoption name {name} value {value}")
            }
            OptionValue::UpdateInt(value) => format!("setoption name {name} value {value}"),
            OptionValue::UpdateBool(value) => format!("setoption name {name} value {value}"),
            OptionValue::UpdateRange(value) => format!("setoption name {name} value {value}"),
            OptionValue::SendTrigger => format!("setoption name {name}"),
          },
        )?;
      }
      Request::Clock(time) => {
        write_mutex(out, format!("clock{}", time.to_string()))?;
      }
      Request::AnalysisResult(result) => {
        // TODO: WDL
        write_mutex(
          out,
          format!(
            "info depth {} score {} time {} nodes {} pv {}\n",
            result.depth,
            // TODO: fix
            result.score.show_uci(0, true),
            result.time,
            result.nodes,
            result
              .pv
              .iter()
              .map(Move::to_string)
              .collect::<Vec<String>>()
              .join(" ")
          ),
        )?;
      }
    }
  }
  Some(())
}

fn process_analysis(
  rx: &Receiver<AnalysisRequest>,
  tx: &Sender<UlciResult>,
  mut input: impl BufRead,
  out: &Arc<Mutex<impl Write>>,
  mut buffer: String,
  completion: impl Fn(),
) -> Option<()> {
  while let Ok(request) = rx.recv() {
    let moves = if request.moves.is_empty() {
      String::new()
    } else {
      format!(
        " moves {}",
        request
          .moves
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(" ")
      )
    };
    if request.new_game {
      write_mutex(out, "ucinewgame")?;
      write_mutex(out, "isready")?;
      while let Ok(chars) = input.read_line(&mut buffer) {
        if chars == 0 {
          return None;
        }
        if buffer.eq("readyok\n") {
          break;
        }
        buffer.clear();
      }
    }
    write_mutex(out, format!("position fen {}{moves}", request.fen))?;
    buffer.clear();
    let moves = if request.searchmoves.is_empty() {
      String::new()
    } else {
      format!(
        " moves {}",
        request
          .searchmoves
          .iter()
          .map(ToString::to_string)
          .collect::<Vec<String>>()
          .join(" ")
      )
    };
    write_mutex(out, format!("go{}{moves}", request.time.to_string()))?;
    while let Ok(chars) = input.read_line(&mut buffer) {
      if chars == 0 {
        return None;
      }
      let mut words = buffer.split_whitespace();
      if let Some(word) = words.next() {
        match word {
          "info" => {
            for message in process_info(words) {
              tx.send(message).ok();
            }
            completion();
          }
          "bestmove" => {
            if let Some(bestmove) = words.next().and_then(|m| m.parse().ok()) {
              tx.send(UlciResult::AnalysisStopped(bestmove)).ok()?;
              completion();
              buffer.clear();
              break;
            }
          }
          _ => (),
        }
      }
      buffer.clear();
    }
  }
  Some(())
}
