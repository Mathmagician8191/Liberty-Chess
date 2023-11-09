use liberty_chess::moves::Move;
use liberty_chess::positions::get_startpos;
use liberty_chess::Board;
use oxidation::search;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::channel;
use std::thread::spawn;
use std::time::Instant;
use ulci::client::{startup, Message};
use ulci::{ClientInfo, SearchTime};

fn main() {
  let (tx, rx) = channel();
  let info = ClientInfo {
    name: "Oxidation".to_owned(),
    username: None,
    author: "Mathmagician".to_owned(),
    options: HashMap::new(),
  };
  let input = BufReader::new(stdin());
  let output = stdout();
  let mut position = get_startpos();
  let mut debug = false;
  spawn(move || startup(&tx, &info, input, output));
  while let Ok(message) = rx.recv() {
    match message {
      Message::SetDebug(new_debug) => debug = new_debug,
      Message::UpdatePosition(board) => position = Board::load_from_thread(*board),
      Message::Go(settings) => {
        let moves = position.generate_legal();
        let moves = moves.iter().filter_map(|board| board.last_move);
        let mut moves: Vec<Move> = if settings.moves.is_empty() {
          moves.collect()
        } else {
          moves.filter(|m| settings.moves.contains(m)).collect()
        };
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
          let start = Instant::now();
          moves.shuffle(&mut thread_rng());
          match settings.time {
            SearchTime::FixedTime(_) | SearchTime::Increment(_, _) => {
              let pv = search(&start, &position, 1, &moves, &mut 0);
              println!("bestmove {}", pv[0].to_string());
            }
            SearchTime::Nodes(depth) => {
              let pv = search(&start, &position, depth as u16, &moves, &mut 0);
              println!("bestmove {}", pv[0].to_string());
            }
            SearchTime::Depth(max_depth) => {
              let pv = search(&start, &position, max_depth, &moves, &mut 0);
              println!("bestmove {}", pv[0].to_string());
            }
            SearchTime::Infinite => {
              let mut depth = 0;
              let mut nodes = 0;
              let pv = 'stop: loop {
                depth += 1;
                let pv = search(&start, &position, depth, &moves, &mut nodes);
                while let Ok(message) = rx.try_recv() {
                  match message {
                    Message::SetDebug(new_debug) => debug = new_debug,
                    Message::UpdatePosition(_) => {
                      if debug {
                        println!("info string servererror search in progress");
                      }
                    }
                    Message::Go(_) => {
                      if debug {
                        println!("info string servererror already searching");
                      }
                    }
                    Message::Stop => break 'stop pv,
                    Message::UpdateOption(_) => (),
                  }
                }
              };
              println!("bestmove {}", pv[0].to_string());
            }
          }
        }
      }
      Message::Stop => {
        println!("info string servererror not currently searching");
      }
      Message::UpdateOption(_) => (),
    }
  }
}
