use crate::helpers::NumericalInput;
use crate::MAX_TIME;
use eframe::egui::Context;
use enum_iterator::Sequence;
use liberty_chess::moves::Move;
use liberty_chess::threading::CompressedBoard;
use liberty_chess::{Board, Gamestate};
use oxidation::glue::process_position;
use oxidation::{random_move, State, HASH_SIZE, QDEPTH, VERSION_NUMBER};
use rand::{thread_rng, Rng};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::spawn;
use ulci::client::Message;
use ulci::server::UlciResult;
use ulci::{Limits as OtherLimits, Score, SearchTime};

#[cfg(feature = "clock")]
use crate::clock::convert;
#[cfg(feature = "clock")]
use std::time::Duration;

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
      Self::Increment(_, _) => "Limit both players by clock",
      #[cfg(feature = "clock")]
      Self::Handicap(_, _, _, _) => "Limit players by different amounts of time",
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
}

impl ToString for PlayerType {
  fn to_string(&self) -> String {
    match self {
      Self::RandomEngine => "Random Mover".to_owned(),
      Self::BuiltIn(_, _) => format!("Oxidation v{VERSION_NUMBER}"),
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
}

impl PlayerData {
  pub fn new(player: &PlayerType, ctx: &Context) -> Self {
    match player {
      PlayerType::RandomEngine => Self::RandomEngine,
      PlayerType::BuiltIn(qdepth, hash_size) => {
        let (send_request, recieve_request) = channel();
        let (send_result, recieve_result) = channel();
        let hash_size = hash_size.get_value();
        let qdepth = qdepth.get_value() as u8;
        let (send_message, receive_message) = channel();
        let ctx = ctx.clone();
        spawn(move || {
          let mut state = State::new(hash_size);
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
        Self::BuiltIn(EngineInterface {
          tx: send_request,
          rx: recieve_result,
          send_message,
          status: false,
        })
      }
    }
  }

  pub fn get_bestmove(
    &mut self,
    board: &Board,
    searchtime: SearchTime,
  ) -> (Option<Move>, Option<Score>) {
    match self {
      Self::RandomEngine => (random_move(board), None),
      Self::BuiltIn(interface) => interface.get_move(board, searchtime),
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
      while let Ok(message) = self.rx.try_recv() {
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
          UlciResult::Startup(_, _) | UlciResult::Info(_, _) => (),
        }
      }
    } else if board.state() == Gamestate::InProgress && !board.promotion_available() {
      // send request
      self.tx.send((board.send_to_thread(), searchtime)).ok();
      self.status = true;
    }
    (result, score)
  }

  pub fn cancel_move(&mut self) {
    if self.status {
      self.send_message.send(Message::Stop).ok();
      // wait for results
      while let Ok(message) = self.rx.recv() {
        match message {
          UlciResult::AnalysisStopped(_) => self.status = false,
          UlciResult::Analysis(_) | UlciResult::Startup(_, _) | UlciResult::Info(_, _) => (),
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
