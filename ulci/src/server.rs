use crate::{
  write, write_mutex, ClientInfo, IntOption, OptionValue, RangeOption, Score, SearchTime,
  UlciOption,
};
use liberty_chess::moves::Move;
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::str::SplitWhitespace;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::spawn;

/// A request for some ULCI action
pub enum Request {
  /// The server needs some analysis from the client
  Analysis(AnalysisRequest),
  /// Stop the analysis
  StopAnalysis,
  /// The server wants to update an option
  SetOption(OptionValue),
  /// Shut down the server and client
  Quit,
}

/// A request for analysis
pub struct AnalysisRequest {
  /// The base position to analyse
  pub fen: String,
  /// Moves from the base position to the current position
  pub moves: Vec<Move>,
  /// The time to analyse for
  pub time: SearchTime,
  /// Which moves to analyse (empty Vec = analyse all)
  pub searchmoves: Vec<Move>,
}

/// The results from the client
pub enum UlciResult {
  /// Analysis results
  Analysis(AnalysisResult),
  /// Analysis is over, return bestmove
  AnalysisStopped(Move),
  /// The client is ready, send client info and version
  Startup(ClientInfo, usize),
  /// Information for the server
  Info(InfoType, String),
}

/// The result from the analysis
pub struct AnalysisResult {
  /// Principal Variation
  ///
  /// the first element is the bestmove
  pub pv: Vec<Move>,
  /// Evaluation of current position
  pub score: Score,
  /// Depth evaluated
  pub depth: u16,
  /// Nodes evaluated
  pub nodes: usize,
}

impl Default for AnalysisResult {
  fn default() -> Self {
    Self {
      pv: Vec::new(),
      score: Score::Centipawn(0.0),
      depth: 1,
      nodes: 1,
    }
  }
}

/// The type of info sent by the client
pub enum InfoType {
  /// A string message
  String,
  /// The client has detected an error
  ClientError,
  /// The client claims the server has made an error
  ServerError,
}

fn convert_words(words: SplitWhitespace) -> String {
  words.collect::<Vec<&str>>().join(" ")
}

fn process_info(mut words: SplitWhitespace, tx: &Sender<UlciResult>) {
  let mut result = AnalysisResult::default();
  let mut modified = false;
  while let Some(word) = words.next() {
    match word {
      "string" => {
        tx.send(UlciResult::Info(InfoType::String, convert_words(words)))
          .ok();
        return;
      }
      "clienterror" => {
        tx.send(UlciResult::Info(
          InfoType::ClientError,
          convert_words(words),
        ))
        .ok();
        return;
      }
      "servererror" => {
        tx.send(UlciResult::Info(
          InfoType::ServerError,
          convert_words(words),
        ))
        .ok();
        return;
      }
      "depth" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          result.depth = value;
          modified = true;
        }
      }
      "nodes" => {
        if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
          result.nodes = value;
          modified = true;
        }
      }
      "score" => {
        if let Some(word) = words.next() {
          match word {
            "mate" => {
              if let Some(word) = words.next() {
                if let Some(word) = word.strip_prefix('-') {
                  if let Ok(moves) = word.parse() {
                    result.score = Score::Loss(moves);
                    modified = true;
                  }
                } else if let Ok(moves) = word.parse() {
                  result.score = Score::Win(moves);
                  modified = true;
                }
              }
            }
            "cp" => {
              if let Some(score) = words.next().and_then(|w| w.parse::<i64>().ok()) {
                result.score = Score::Centipawn(score as f64);
                modified = true;
              }
            }
            "wdl" => {
              if let (Some(win), Some(draw), Some(loss)) = (
                words.next().and_then(|w| w.parse().ok()),
                words.next().and_then(|w| w.parse().ok()),
                words.next().and_then(|w| w.parse().ok()),
              ) {
                result.score = Score::WDL(win, draw, loss);
                modified = true;
              }
            }
            _ => (),
          }
        }
      }
      // TODO fix: only works as the last option
      "pv" => {
        modified = true;
        break;
      }
      _ => (),
    }
  }
  if modified {
    result.pv = words.flat_map(str::parse).collect();
    tx.send(UlciResult::Analysis(result)).ok();
  }
}

/// Start up a ULCI server
///
/// Has limited error handling
///
/// Blocks the current thread
pub fn startup(
  requests: Receiver<Request>,
  results: &Sender<UlciResult>,
  mut input: impl BufRead,
  mut out: impl Write + Send + 'static,
  debug: bool,
) -> Option<()> {
  write(&mut out, "uci");
  let mut version = 0;
  let mut name = String::new();
  let mut username = None;
  let mut author = String::new();
  let mut options = HashMap::new();
  let mut buffer = String::new();
  while let Ok(chars) = input.read_line(&mut buffer) {
    if chars == 0 {
      return None;
    }
    let mut words = buffer.split_whitespace();
    match words.next() {
      Some("id") => match words.next() {
        Some("version") => {
          if let Some(value) = words.next().and_then(|w| w.parse().ok()) {
            version = value;
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
            } else {
              name_words.push(word);
            }
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
                    "default" => {
                      default = words.next().and_then(|w| w.parse().ok()).or(default);
                    }
                    "min" => {
                      min = words.next().and_then(|w| w.parse().ok()).or(min);
                    }
                    "max" => {
                      max = words.next().and_then(|w| w.parse().ok()).or(max);
                    }
                    _ => (),
                  }
                }
                if let Some(default) = default {
                  options.insert(
                    option_name,
                    UlciOption::Int(IntOption { default, min, max }),
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
                    } else {
                      default += word;
                    }
                  }
                  let mut choices = HashSet::new();
                  loop {
                    let mut option = String::new();
                    for word in words.by_ref() {
                      if word == "var" {
                        break;
                      } else {
                        option += word;
                      }
                    }
                    if option.is_empty() {
                      break;
                    } else {
                      choices.insert(option);
                    }
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
        process_info(words, results);
      }
      Some("uciok") => break,
      Some(_) | None => (),
    }
    buffer.clear();
  }
  buffer.clear();
  if debug {
    write(&mut out, "debug on");
  }
  results
    .send(UlciResult::Startup(
      ClientInfo {
        name,
        username,
        author,
        options,
      },
      version,
    ))
    .ok();
  let (tx, rx) = channel();
  let out = Arc::new(Mutex::new(out));
  let new_out = out.clone();
  spawn(move || process_server(&requests, &tx, &new_out));
  process_analysis(&rx, results, input, &out, buffer)
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
      Request::StopAnalysis => write_mutex(out, "stop"),
      Request::SetOption(option) => {
        write_mutex(
          out,
          match option {
            OptionValue::UpdateString(name, value) => {
              format!("setoption name {name} value {value}")
            }
            OptionValue::UpdateInt(name, value) => format!("setoption name {name} value {value}"),
            OptionValue::UpdateBool(name, value) => format!("setoption name {name} value {value}"),
            OptionValue::UpdateRange(name, value) => format!("setoption name {name} value {value}"),
            OptionValue::SendTrigger(name) => format!("setoption name {name}"),
          },
        );
      }
      Request::Quit => write_mutex(out, "quit"),
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
    write_mutex(out, format!("position fen {}{moves}", request.fen));
    write_mutex(out, "isready");
    while let Ok(chars) = input.read_line(&mut buffer) {
      if chars == 0 {
        return None;
      }
      if buffer.eq("readyok\n") {
        break;
      }
      buffer.clear();
    }
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
    write_mutex(
      out,
      match request.time {
        SearchTime::FixedTime(time) => format!("go movetime {}{moves}", time.as_millis()),
        SearchTime::Increment(time, inc) => {
          let time = time.as_millis();
          let inc = inc.as_millis();
          format!("go wtime {time} winc {inc} btime {time} binc {time}{moves}")
        }
        SearchTime::Depth(depth) => format!("go depth {depth}{moves}"),
        SearchTime::Nodes(nodes) => format!("go nodes {nodes}{moves}"),
        SearchTime::Infinite => format!("go infinite{moves}"),
      },
    );
    while let Ok(chars) = input.read_line(&mut buffer) {
      if chars == 0 {
        return None;
      }
      let mut words = buffer.split_whitespace();
      if let Some(word) = words.next() {
        match word {
          "info" => process_info(words, tx),
          "bestmove" => {
            if let Some(bestmove) = words.next().and_then(|m| m.parse().ok()) {
              tx.send(UlciResult::AnalysisStopped(bestmove)).ok();
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
