use liberty_chess::clock::Clock;
use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::{Board, Gamestate};
use rand::distributions::Alphanumeric;
use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};
use server::handle_connections;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread::{sleep, spawn};
use std::time::Duration;
use ulci::server::{AnalysisRequest, Request, UlciResult};
use ulci::{load_engine, SearchTime};

/// The test positions for the match
const POSITIONS: &[&str] = &[
  STARTPOS,
  CAPABLANCA_RECTANGLE,
  CAPABLANCA,
  LIBERTY_CHESS,
  MINI,
  MONGOL,
  AFRICAN,
  NARNIA,
  TRUMP,
  LOADED_BOARD,
  DOUBLE_CHESS,
  HORDE,
  ELIMINATION,
  "4k3/pppppppp/8/8/8/8/PPPPPPPP/4K3 w - - 0 1",
];

const FRIENDLY_FIRE_CHANCE: f64 = 1.0;

const GAME_LIMIT: usize = 2;

const WHITE_ENGINE: Option<&str> = None;
const BLACK_ENGINE: Option<&str> = None;

const KIBBUTZ_ENGINE: Option<&str> = Some("target/release/oxidation-0.5.2");

enum SpectatorMessage {
  Request(Request),
  Spectator(Sender<Request>),
  Kibbutz(UlciResult),
}

fn process_spectators(mut spectators: Vec<Sender<Request>>, messages: Receiver<SpectatorMessage>) {
  let mut last_request = None;
  while let Ok(message) = messages.recv() {
    match message {
      SpectatorMessage::Request(request) => {
        spectators.retain(|spectator| spectator.send(request.clone()).is_ok());
        last_request = Some(request);
      }
      SpectatorMessage::Spectator(spectator) => {
        if let Some(ref request) = last_request {
          spectator.send(request.clone()).ok();
        }
        spectators.push(spectator);
      }
      SpectatorMessage::Kibbutz(result) => {
        if let UlciResult::Analysis(result) = result {
          spectators.retain(|spectator| {
            spectator
              .send(Request::AnalysisResult(result.clone()))
              .is_ok()
          });
        }
      }
    }
  }
}

fn run_match(
  (mut tx_1, mut rx_1): (Sender<Request>, Receiver<UlciResult>),
  (mut tx_2, mut rx_2): (Sender<Request>, Receiver<UlciResult>),
  spectators: Sender<SpectatorMessage>,
  kibbutz_tx: Option<Sender<Request>>,
) -> Option<()> {
  for _ in 0..GAME_LIMIT {
    let fen = POSITIONS
      .choose(&mut thread_rng())
      .expect("Could not find position");
    let mut position = Board::new(fen).unwrap();
    if thread_rng().gen_bool(FRIENDLY_FIRE_CHANCE) {
      position.friendly_fire = true;
    }
    let mut base_position = position.clone();
    let mut moves = Vec::new();
    tx_1
      .send(Request::Position(
        base_position.to_string(),
        moves.clone(),
        false,
      ))
      .ok()?;
    tx_2
      .send(Request::Position(
        base_position.to_string(),
        moves.clone(),
        false,
      ))
      .ok()?;
    let mut clock = Clock::new_symmetric(
      Duration::from_secs(1200),
      Duration::from_secs(15),
      position.to_move(),
    );
    clock.toggle_pause();
    while position.state() == Gamestate::InProgress {
      tx_1
        .send(Request::Analysis(AnalysisRequest {
          fen: base_position.to_string(),
          moves: moves.clone(),
          time: SearchTime::from_clock(&mut clock),
          searchmoves: Vec::new(),
          new_game: false,
        }))
        .ok()?;
      spectators
        .send(SpectatorMessage::Request(Request::Position(
          base_position.to_string(),
          moves.to_vec(),
          false,
        )))
        .ok();
      spectators
        .send(SpectatorMessage::Request(Request::Clock(
          SearchTime::from_clock(&mut clock),
        )))
        .ok();
      if let Some(ref kibbutz) = kibbutz_tx {
        kibbutz.send(Request::StopAnalysis).ok();
        kibbutz
          .send(Request::Analysis(AnalysisRequest {
            fen: base_position.to_string(),
            moves: moves.clone(),
            searchmoves: Vec::new(),
            time: SearchTime::Infinite,
            new_game: false,
          }))
          .ok();
      }
      loop {
        match rx_1.recv().ok()? {
          UlciResult::AnalysisStopped(r#move) => {
            if let Some(board) = position.move_if_legal(r#move) {
              if board.halfmoves() == 0 {
                base_position = position;
                moves.clear();
              }
              position = board;
              moves.push(r#move);
              clock.switch_clocks();
            }
            break;
          }
          UlciResult::Analysis(result) => {
            spectators.send(SpectatorMessage::Kibbutz(UlciResult::Analysis(result))).ok();
          }
          _ => (),
        }
      }
      if position.state() != Gamestate::InProgress {
        break;
      }
      tx_2
        .send(Request::Analysis(AnalysisRequest {
          fen: base_position.to_string(),
          moves: moves.clone(),
          time: SearchTime::from_clock(&mut clock),
          searchmoves: Vec::new(),
          new_game: false,
        }))
        .ok()?;
      spectators
        .send(SpectatorMessage::Request(Request::Position(
          base_position.to_string(),
          moves.to_vec(),
          false,
        )))
        .ok();
      spectators
        .send(SpectatorMessage::Request(Request::Clock(
          SearchTime::from_clock(&mut clock),
        )))
        .ok();
      if let Some(ref kibbutz) = kibbutz_tx {
        kibbutz.send(Request::StopAnalysis).ok();
        kibbutz
          .send(Request::Analysis(AnalysisRequest {
            fen: base_position.to_string(),
            moves: moves.clone(),
            searchmoves: Vec::new(),
            time: SearchTime::Infinite,
            new_game: false,
          }))
          .ok();
      }
      loop {
        match rx_2.recv().ok()? {
          UlciResult::AnalysisStopped(r#move) => {
            if let Some(board) = position.move_if_legal(r#move) {
              if board.halfmoves() == 0 {
                base_position = position;
                moves.clear();
              }
              position = board;
              moves.push(r#move);
              clock.switch_clocks();
            }
            break;
          }
          UlciResult::Analysis(result) => {
            spectators.send(SpectatorMessage::Kibbutz(UlciResult::Analysis(result))).ok();
          }
          _ => (),
        }
      }
    }
    tx_1
      .send(Request::Position(
        base_position.to_string(),
        moves.clone(),
        false,
      ))
      .ok()?;
    tx_2
      .send(Request::Position(
        base_position.to_string(),
        moves.clone(),
        false,
      ))
      .ok()?;
    spectators
      .send(SpectatorMessage::Request(Request::Position(
        base_position.to_string(),
        moves.to_vec(),
        false,
      )))
      .ok();
    sleep(Duration::from_secs(10));
    (tx_1, tx_2) = (tx_2, tx_1);
    (rx_1, rx_2) = (rx_2, rx_1);
  }
  None
}

fn main() {
  let password_1: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(6)
    .map(char::from)
    .collect();
  println!("Password 1: {password_1}");
  let password_2: String = thread_rng()
    .sample_iter(&Alphanumeric)
    .take(6)
    .map(char::from)
    .collect();
  println!("Password 2: {password_2}");
  let mut player_1 = WHITE_ENGINE.map(load_engine);
  let mut player_2 = BLACK_ENGINE.map(load_engine);
  let mut spectators = Vec::new();
  let (tx, rx) = channel();
  spawn(|| handle_connections(tx));
  while let Ok((tx, rx, client)) = rx.recv() {
    let name = client.username;
    if name == Some(password_1.clone()) {
      println!("Found player 1");
      player_1 = Some((tx, rx));
      if player_2.is_some() {
        break;
      }
    } else if name == Some(password_2.clone()) {
      println!("Found player 2");
      player_2 = Some((tx, rx));
      if player_1.is_some() {
        break;
      }
    } else {
      println!("Found spectator");
      spectators.push(tx);
      if player_1.is_some() && player_2.is_some() {
        break;
      }
    }
  }
  if let (Some(player_1), Some(player_2)) = (player_1, player_2) {
    println!("Starting match");
    let (spectator_tx, spectator_rx) = channel();
    let spectator_tx_copy = spectator_tx.clone();
    let mut kibbutz_tx = None;
    if let Some((tx, rx)) = KIBBUTZ_ENGINE.map(load_engine) {
      kibbutz_tx = Some(tx);
      let spectator_tx_copy = spectator_tx.clone();
      spawn(move || {
        while let Ok(message) = rx.recv() {
          spectator_tx_copy
            .send(SpectatorMessage::Kibbutz(message))
            .ok();
        }
      });
    }
    spawn(move || {
      while let Ok((spectator, _, _)) = rx.recv() {
        spectator_tx_copy
          .send(SpectatorMessage::Spectator(spectator))
          .ok();
      }
    });
    spawn(|| process_spectators(spectators, spectator_rx));
    run_match(player_1, player_2, spectator_tx, kibbutz_tx);
  } else {
    println!("Something went wrong!");
  }
}
