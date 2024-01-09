use liberty_chess::clock::Clock;
use liberty_chess::positions::{
  AFRICAN, CAPABLANCA, CAPABLANCA_RECTANGLE, DOUBLE_CHESS, ELIMINATION, HORDE, LIBERTY_CHESS,
  LOADED_BOARD, MINI, MONGOL, NARNIA, STARTPOS, TRUMP,
};
use liberty_chess::{Board, Gamestate};
use parking_lot::Mutex;
use std::io::BufReader;
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::Arc;
use std::thread::{sleep, spawn};
use std::time::Duration;
use ulci::server::{startup_server, AnalysisRequest, Request, UlciResult};
use ulci::{load_engine, OptionValue, SearchTime};

const PORT: u16 = 25565;

/// The test positions for the match
pub const POSITIONS: &[&str] = &[
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

fn run_client(name: String, tx: Sender<Request>, rx: Receiver<UlciResult>) -> Option<()> {
  for position in POSITIONS {
    let mut position = Board::new(position).unwrap();
    let mut base_position = position.clone();
    let mut moves = Vec::new();
    let mut clock = Clock::new(
      [
        Duration::from_secs(1200),
        Duration::from_secs(8),
        Duration::from_secs(20),
        Duration::from_millis(80),
      ],
      position.to_move(),
    );
    let (engine_tx, engine_rx) = load_engine("./target/release/oxidation");
    clock.toggle_pause();
    while position.state() == Gamestate::InProgress {
      tx.send(Request::Analysis(AnalysisRequest {
        fen: base_position.to_string(),
        moves: moves.clone(),
        time: SearchTime::from_clock(&mut clock),
        searchmoves: Vec::new(),
        new_game: false,
      }))
      .ok()?;
      loop {
        if let UlciResult::AnalysisStopped(r#move) = rx.recv().ok()? {
          println!("{name} played {}", r#move.to_string());
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
      }
      if position.state() != Gamestate::InProgress {
        break;
      }
      engine_tx
        .send(Request::Analysis(AnalysisRequest {
          fen: base_position.to_string(),
          moves: moves.clone(),
          time: SearchTime::from_clock(&mut clock),
          searchmoves: Vec::new(),
          new_game: false,
        }))
        .ok()?;
      loop {
        if let UlciResult::AnalysisStopped(r#move) = engine_rx.recv().ok()? {
          println!("Oxidation ({name}) played {}", r#move.to_string());
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
      }
    }
    tx.send(Request::Position(base_position.to_string(), moves, false))
      .ok()?;
    sleep(Duration::from_secs(10));
  }
  Some(())
}

fn handle_connection(
  stream: TcpStream,
  connections: &Arc<Mutex<Vec<Sender<Request>>>>,
) -> Option<()> {
  let name = match stream.peer_addr() {
    Ok(ip) => {
      println!("{ip} Connected");
      ip.to_string()
    }
    Err(_) => {
      println!("Unknown Connected");
      "Unknown".to_string()
    }
  };
  let name2 = name.clone();
  let stream_2 = stream.try_clone().ok()?;
  let (tx, rx) = channel();
  connections.lock().push(tx.clone());
  let (tx_2, rx_2) = channel();
  spawn(move || {
    startup_server(rx, &tx_2, BufReader::new(stream), stream_2, false, || ());
    println!("{name} Disconnected");
  });
  spawn(move || run_client(name2, tx, rx_2));
  Some(())
}

fn handle_connections(connections: Arc<Mutex<Vec<Sender<Request>>>>) {
  let listener = TcpListener::bind(format!("0.0.0.0:{PORT}"))
    .unwrap_or_else(|_| panic!("Failed to bind to port {PORT}"));

  for stream in listener.incoming().flatten() {
    if handle_connection(stream, &connections).is_none() {
      println!("try_clone broke");
    }
  }
}

fn main() {
  let connections = Arc::new(Mutex::new(Vec::new()));
  let new_connections = connections.clone();
  spawn(|| handle_connections(new_connections));
  loop {
    let mut lock = connections.lock();
    if !lock.is_empty() {
      lock.retain(|connection| {
        connection
          .send(Request::SetOption(String::new(), OptionValue::SendTrigger))
          .is_ok()
      });
      println!("Sending to {} users", lock.len());
    }
    drop(lock);
    sleep(Duration::from_millis(10000));
  }
}
