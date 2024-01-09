use crate::helpers::NumericalInput;
use crate::MAX_TIME;
use eframe::egui::Context;
use enum_iterator::Sequence;
use liberty_chess::moves::Move;
use liberty_chess::parsing::from_chars;
use liberty_chess::positions::get_startpos;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate, ALL_PIECES};
use oxidation::glue::process_position;
use oxidation::{random_move, State, HASH_SIZE, QDEPTH, VERSION_NUMBER};
use rand::{thread_rng, Rng};
use std::collections::HashMap;
use std::io::{BufReader, ErrorKind, Write};
use std::net::{SocketAddr, TcpStream};
use std::process::{Command, Stdio};
use std::sync::mpsc::{channel, Receiver, Sender, TryRecvError};
use std::thread::spawn;
use std::time::Duration;
use ulci::client::{startup, Message};
use ulci::server::{startup_server, AnalysisRequest, Request, UlciResult};
use ulci::{ClientInfo, Limits as OtherLimits, Score, SearchTime, SupportedFeatures, V1Features};

#[cfg(feature = "clock")]
use crate::clock::convert;

#[derive(Eq, PartialEq)]
pub enum SearchType {
  #[cfg(feature = "clock")]
  Increment(NumericalInput<u64>, NumericalInput<u64>),
  #[cfg(feature = "clock")]
  Handicap(
    NumericalInput<u64>,
    NumericalInput<u64>,
    NumericalInput<u64>,
    NumericalInput<u64>,
  ),
  Other(Limits),
}

impl ToString for SearchType {
  fn to_string(&self) -> String {
    match self {
      #[cfg(feature = "clock")]
      Self::Increment(..) => "Limit both players by clock",
      #[cfg(feature = "clock")]
      Self::Handicap(..) => "Limit players by different amounts of time",
      Self::Other(_) => "Limit by depth, nodes and/or time",
    }
    .to_owned()
  }
}

impl Default for SearchType {
  fn default() -> Self {
    Self::Other(Limits {
      depth: Some(Self::depth()),
      nodes: Some(Self::nodes()),
      time: None,
    })
  }
}

impl SearchType {
  #[cfg(feature = "clock")]
  pub fn get_value(&self, engine_side: bool) -> (SearchTime, Option<[Duration; 4]>) {
    match self {
      Self::Increment(time, inc) => (
        SearchTime::Increment(
          u128::from(time.get_value() * 60_000),
          u128::from(inc.get_value() * 1000),
        ),
        Some(convert(&[
          time.clone(),
          time.clone(),
          inc.clone(),
          inc.clone(),
        ])),
      ),
      Self::Handicap(human_time, human_inc, engine_time, engine_inc) => {
        let engine = (engine_time, engine_inc);
        let human = (human_time, human_inc);
        let ((white_time, white_inc), (black_time, black_inc)) = if engine_side {
          (engine, human)
        } else {
          (human, engine)
        };
        (
          SearchTime::Increment(
            u128::from(engine_time.get_value() * 60_000),
            u128::from(engine_inc.get_value() * 1000),
          ),
          Some(convert(&[
            white_time.clone(),
            black_time.clone(),
            white_inc.clone(),
            black_inc.clone(),
          ])),
        )
      }
      Self::Other(limits) => (
        SearchTime::Other(OtherLimits {
          depth: limits
            .depth
            .as_ref()
            .map_or(u8::MAX, |v| v.get_value() as u8),
          nodes: limits
            .nodes
            .as_ref()
            .map_or(usize::MAX, NumericalInput::get_value),
          time: limits
            .time
            .as_ref()
            .map_or(u128::MAX, NumericalInput::get_value),
        }),
        None,
      ),
    }
  }

  #[cfg(not(feature = "clock"))]
  pub fn get_value(&self) -> SearchTime {
    match &self {
      Self::Other(limits) => SearchTime::Other(OtherLimits {
        depth: limits
          .depth
          .as_ref()
          .map_or(u8::MAX, |v| v.get_value() as u8),
        nodes: limits
          .nodes
          .as_ref()
          .map_or(usize::MAX, NumericalInput::get_value),
        time: limits
          .time
          .as_ref()
          .map_or(u128::MAX, NumericalInput::get_value),
      }),
    }
  }

  pub fn depth() -> NumericalInput<u16> {
    NumericalInput::new(3, 0, u16::from(u8::MAX))
  }

  pub fn nodes() -> NumericalInput<usize> {
    NumericalInput::new(100_000, 0, usize::MAX)
  }

  pub fn time() -> NumericalInput<u128> {
    NumericalInput::new(1000, 0, u128::from(MAX_TIME * 1000))
  }

  #[cfg(feature = "clock")]
  pub fn increment(time: u64, inc: u64) -> Self {
    Self::Increment(
      NumericalInput::new(time, 0, MAX_TIME),
      NumericalInput::new(inc, 0, MAX_TIME),
    )
  }

  #[cfg(feature = "clock")]
  pub fn handicap(human_time: u64, human_inc: u64, engine_time: u64, engine_inc: u64) -> Self {
    Self::Handicap(
      NumericalInput::new(human_time, 0, MAX_TIME),
      NumericalInput::new(human_inc, 0, MAX_TIME),
      NumericalInput::new(engine_time, 0, MAX_TIME),
      NumericalInput::new(engine_inc, 0, MAX_TIME),
    )
  }
}

#[derive(Eq, PartialEq)]
pub struct Limits {
  pub depth: Option<NumericalInput<u16>>,
  pub nodes: Option<NumericalInput<usize>>,
  pub time: Option<NumericalInput<u128>>,
}

#[derive(Clone, Eq, PartialEq)]
pub enum PlayerType {
  RandomEngine,
  // parameters are qdepth and hash size
  BuiltIn(NumericalInput<u16>, NumericalInput<usize>),
  External(String),
  Multiplayer(String, NumericalInput<u16>, String),
}

impl ToString for PlayerType {
  fn to_string(&self) -> String {
    match self {
      Self::RandomEngine => "Random Mover".to_owned(),
      Self::BuiltIn(..) => format!("Oxidation v{VERSION_NUMBER}"),
      Self::External(_) => "External engine (beta)".to_owned(),
      Self::Multiplayer(..) => "Connect to server (beta)".to_owned(),
    }
  }
}

impl PlayerType {
  pub fn built_in() -> Self {
    Self::BuiltIn(
      NumericalInput::new(u16::from(QDEPTH), 0, u16::from(u8::MAX)),
      NumericalInput::new(HASH_SIZE, 0, 1 << 32),
    )
  }

  #[cfg(feature = "clock")]
  pub const fn is_thinking(&self) -> bool {
    match self {
      Self::RandomEngine => false,
      Self::BuiltIn(..) | Self::External(_) | Self::Multiplayer(..) => true,
    }
  }

  pub const fn custom_thinking_time(&self) -> bool {
    match self {
      Self::RandomEngine | Self::Multiplayer(..) => false,
      Self::BuiltIn(..) | Self::External(_) => true,
    }
  }
}

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum PlayerColour {
  White,
  Black,
  Random,
}

impl ToString for PlayerColour {
  fn to_string(&self) -> String {
    match self {
      Self::White => "White",
      Self::Black => "Black",
      Self::Random => "Random",
    }
    .to_string()
  }
}

impl PlayerColour {
  pub fn get_colour(self) -> bool {
    match self {
      Self::White => true,
      Self::Black => false,
      Self::Random => thread_rng().gen(),
    }
  }
}

pub enum PlayerData {
  RandomEngine,
  BuiltIn(EngineInterface),
  Uci(UciInterface),
  Multiplayer(Connection),
}

impl PlayerData {
  pub fn new(player: &PlayerType, board: &Board, ctx: &Context) -> Result<Self, String> {
    match player {
      PlayerType::RandomEngine => Ok(Self::RandomEngine),
      PlayerType::BuiltIn(qdepth, hash_size) => {
        let (send_request, recieve_request) = channel();
        let (send_result, recieve_result) = channel();
        let hash_size = hash_size.get_value();
        let qdepth = qdepth.get_value() as u8;
        let (send_message, receive_message) = channel();
        let ctx = ctx.clone();
        spawn(move || {
          let mut state = State::new(hash_size, &get_startpos());
          while let Ok((board, searchtime)) = recieve_request.recv() {
            process_position(
              &send_result,
              &receive_message,
              board,
              searchtime,
              qdepth,
              &mut state,
            );
            ctx.request_repaint();
          }
        });
        Ok(Self::BuiltIn(EngineInterface {
          tx: send_request,
          rx: recieve_result,
          send_message,
          status: false,
        }))
      }
      PlayerType::External(path) => {
        let (send_request, recieve_request) = channel();
        let (send_result, recieve_result) = channel();
        let mut engine = Command::new(path)
          .stdin(Stdio::piped())
          .stdout(Stdio::piped())
          .spawn()
          .map_err(|_| "Invalid path".to_owned())?;
        let stdin = engine
          .stdin
          .take()
          .ok_or_else(|| "Could not load stdin".to_owned())?;
        let stdout = BufReader::new(
          engine
            .stdout
            .take()
            .ok_or_else(|| "Could not load stdout".to_owned())?,
        );
        let ctx = ctx.clone();
        spawn(move || {
          startup_server(
            recieve_request,
            &send_result,
            stdout,
            stdin,
            false,
            move || ctx.request_repaint(),
          );
        });
        Ok(Self::Uci(UciInterface {
          tx: send_request,
          rx: recieve_result,
          state: UciState::Pending,
          board: Box::new(board.clone()),
        }))
      }
      PlayerType::Multiplayer(ip, port, name) => {
        let address = format!("{ip}:{}", port.get_value())
          .parse()
          .map_err(|_| "Invalid IP address".to_owned())?;
        let name = name.to_owned();
        let (tx, rx) = channel();
        spawn(move || {
          process_connection(address, tx, name);
        });
        Ok(Self::Multiplayer(Connection {
          connection: rx,
          output: None,
        }))
      }
    }
  }

  // Update and check for bestmove/score update
  pub fn poll(&mut self, board: &Board, searchtime: SearchTime) -> (Option<Move>, Option<Score>) {
    match self {
      Self::RandomEngine => (random_move(board), None),
      Self::BuiltIn(interface) => interface.get_move(board, searchtime),
      Self::Uci(interface) => interface.get_move(board, searchtime),
      Self::Multiplayer(_) => (None, None),
    }
  }

  pub fn cancel_move(&mut self) {
    match self {
      Self::BuiltIn(interface) => interface.cancel_move(),
      Self::Uci(interface) => interface.cancel_move(),
      Self::RandomEngine | Self::Multiplayer(_) => (),
    }
  }
}

pub struct EngineInterface {
  tx: Sender<(CompressedBoard, SearchTime)>,
  rx: Receiver<UlciResult>,
  send_message: Sender<Message>,
  status: bool,
}

impl EngineInterface {
  pub fn get_move(
    &mut self,
    board: &Board,
    searchtime: SearchTime,
  ) -> (Option<Move>, Option<Score>) {
    let (mut result, mut score) = (None, None);
    if self.status {
      // request sent, poll for results
      for message in self.rx.try_iter() {
        match message {
          UlciResult::AnalysisStopped(bestmove) => {
            result = Some(bestmove);
            self.status = false;
          }
          UlciResult::Analysis(result) => {
            let mut result = result.score;
            if board.to_move() {
              result = -result;
            }
            score = Some(result);
          }
          UlciResult::Startup(_) | UlciResult::Info(..) => (),
        }
      }
    } else if board.state() == Gamestate::InProgress && !board.promotion_available() {
      // send request
      self.tx.send((board.send_to_thread(), searchtime)).ok();
      self.status = true;
    }
    (result, score)
  }

  fn cancel_move(&mut self) {
    if self.status {
      self.send_message.send(Message::Stop).ok();
      // wait for results
      while let Ok(message) = self.rx.recv() {
        match message {
          UlciResult::AnalysisStopped(_) => self.status = false,
          UlciResult::Analysis(_) | UlciResult::Startup(_) | UlciResult::Info(..) => (),
        }
      }
    }
  }
}

impl Drop for EngineInterface {
  fn drop(&mut self) {
    self.send_message.send(Message::Stop).ok();
  }
}

pub struct UciInterface {
  tx: Sender<Request>,
  rx: Receiver<UlciResult>,
  pub state: UciState,
  // Hacky solution to preserve the board until the engine has loaded
  pub board: Box<Board>,
}

impl UciInterface {
  pub fn poll(&mut self) {
    match self.state {
      UciState::Pending => loop {
        match self.rx.try_recv() {
          Ok(message) => match message {
            UlciResult::Startup(info) => {
              self.state = if info.supports(&self.board) {
                UciState::Waiting
              } else {
                UciState::Unsupported
              };
            }
            UlciResult::Analysis(_) | UlciResult::AnalysisStopped(_) | UlciResult::Info(..) => (),
          },
          Err(TryRecvError::Disconnected) => {
            self.state = UciState::Crashed;
            break;
          }
          Err(TryRecvError::Empty) => break,
        }
      },
      UciState::Waiting
      | UciState::Analysing
      | UciState::AwaitStop
      | UciState::Unsupported
      | UciState::Crashed => (),
    }
  }

  pub fn get_move(
    &mut self,
    board: &Board,
    searchtime: SearchTime,
  ) -> (Option<Move>, Option<Score>) {
    let (mut result, mut score) = (None, None);
    match self.state {
      UciState::Pending => loop {
        match self.rx.try_recv() {
          Ok(message) => match message {
            UlciResult::Startup(info) => {
              self.state = if info.supports(board) {
                UciState::Waiting
              } else {
                UciState::Unsupported
              };
            }
            UlciResult::Analysis(_) | UlciResult::AnalysisStopped(_) | UlciResult::Info(..) => (),
          },
          Err(TryRecvError::Disconnected) => {
            self.state = UciState::Crashed;
            break;
          }
          Err(TryRecvError::Empty) => break,
        }
      },
      UciState::Waiting => {
        if board.state() == Gamestate::InProgress && !board.promotion_available() {
          // send request
          // TODO: send board history properly
          self
            .tx
            .send(Request::Analysis(AnalysisRequest {
              fen: board.to_string(),
              moves: Vec::new(),
              time: searchtime,
              searchmoves: Vec::new(),
              new_game: false,
            }))
            .ok();
          self.state = UciState::Analysing;
        }
      }
      UciState::Analysing => {
        // request sent, poll for results
        loop {
          match self.rx.try_recv() {
            Ok(message) => match message {
              UlciResult::AnalysisStopped(bestmove) => {
                result = Some(bestmove);
                self.state = UciState::Waiting;
              }
              UlciResult::Analysis(result) => {
                let mut result = result.score;
                if board.to_move() {
                  result = -result;
                }
                score = Some(result);
              }
              UlciResult::Startup(_) | UlciResult::Info(..) => (),
            },
            Err(TryRecvError::Disconnected) => {
              self.state = UciState::Crashed;
              break;
            }
            Err(TryRecvError::Empty) => break,
          }
        }
      }
      UciState::AwaitStop => loop {
        match self.rx.try_recv() {
          Ok(message) => match message {
            UlciResult::AnalysisStopped(_) => self.state = UciState::Waiting,
            UlciResult::Analysis(_) | UlciResult::Startup(_) | UlciResult::Info(..) => (),
          },
          Err(TryRecvError::Disconnected) => {
            self.state = UciState::Crashed;
            break;
          }
          Err(TryRecvError::Empty) => break,
        }
      },
      UciState::Unsupported | UciState::Crashed => (),
    }
    (result, score)
  }

  fn cancel_move(&mut self) {
    if self.state == UciState::Analysing {
      self.tx.send(Request::StopAnalysis).ok();
      self.state = UciState::AwaitStop;
    }
  }
}

impl Drop for UciInterface {
  fn drop(&mut self) {
    self.tx.send(Request::StopAnalysis).ok();
  }
}

#[derive(Eq, PartialEq)]
pub enum UciState {
  Pending,
  Waiting,
  Analysing,
  AwaitStop,
  Unsupported,
  Crashed,
}

pub struct Connection {
  pub connection: Receiver<ConnectionMessage>,
  pub output: Option<TcpStream>,
}

impl Connection {
  pub fn play_move(&self, r#move: Move) {
    self
      .output
      .as_ref()
      .expect("Connection is missing a stream")
      .write_all(format!("bestmove {}\n", r#move.to_string()).as_bytes())
      .ok();
  }
}

pub enum ConnectionMessage {
  Connected(TcpStream),
  Timeout,
  Uci(Message),
}

fn process_connection(
  address: SocketAddr,
  tx: Sender<ConnectionMessage>,
  name: String,
) -> Option<()> {
  match TcpStream::connect_timeout(&address, Duration::from_secs(10)) {
    Ok(connection) => {
      let connection_2 = match connection.try_clone() {
        Ok(connection) => connection,
        Err(_) => {
          return None;
        }
      };
      let connection_3 = match connection.try_clone() {
        Ok(connection) => connection,
        Err(_) => {
          return None;
        }
      };
      tx.send(ConnectionMessage::Connected(connection_3)).ok()?;
      let (uci_tx, rx) = channel();
      spawn(move || {
        startup(
          &uci_tx,
          &ClientInfo {
            features: SupportedFeatures {
              v1: V1Features::all(),
            },
            name: format!("Liberty Chess v{}", env!("CARGO_PKG_VERSION")),
            username: Some(name),
            author: "Mathmagician".to_owned(),
            options: HashMap::new(),
            pieces: from_chars(ALL_PIECES),
            depth: 0,
          },
          BufReader::new(connection),
          connection_2,
        )
      });
      while let Ok(message) = rx.recv() {
        tx.send(ConnectionMessage::Uci(message)).ok()?;
      }
    }
    Err(error) => {
      if error.kind() == ErrorKind::TimedOut {
        tx.send(ConnectionMessage::Timeout).ok()?;
      }
    }
  }
  None
}
