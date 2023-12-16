use liberty_chess::moves::Move;
use liberty_chess::parsing::from_chars;
use liberty_chess::positions::get_startpos;
use liberty_chess::ALL_PIECES;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::collections::HashMap;
use std::io::{stdin, stdout, BufReader};
use std::sync::mpsc::channel;
use std::thread::spawn;
use ulci::client::{startup, Message};
use ulci::{ClientInfo, SearchTime, VERSION};

fn main() {
  let (tx, rx) = channel();
  let info = ClientInfo {
    version: VERSION,
    name: "Random mover".to_owned(),
    username: None,
    author: "Mathmagician".to_owned(),
    options: HashMap::new(),
    pieces: from_chars(ALL_PIECES),
    depth: 0,
  };
  let input = BufReader::new(stdin());
  let output = stdout();
  let mut position = get_startpos();
  let mut debug = false;
  let mut selected_move = None;
  spawn(move || startup(&tx, &info, input, output));
  while let Ok(message) = rx.recv() {
    match message {
      Message::SetDebug(new_debug) => debug = new_debug,
      Message::UpdatePosition(board) => position = board.load_from_thread(),
      Message::Go(settings) => {
        let moves = position.generate_legal();
        let moves = moves.iter().filter_map(|board| board.last_move);
        let moves: Vec<Move> = if settings.moves.is_empty() {
          moves.collect()
        } else {
          moves.filter(|m| settings.moves.contains(m)).collect()
        };
        selected_move = moves.choose(&mut thread_rng()).copied();
        if let Some(chosen_move) = selected_move {
          match settings.time {
            SearchTime::Increment(_, _) | SearchTime::Other(_) => {
              println!(
                "info depth 1 score cp 0 time 0 nodes 1 nps 1 pv {}",
                chosen_move.to_string()
              );
              println!("bestmove {}", chosen_move.to_string());
              selected_move = None;
            }
            SearchTime::Infinite => println!(
              "info depth 1 score cp 0 time 0 nodes 1 nps 1 pv {}",
              chosen_move.to_string()
            ),
          }
        } else if debug {
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
          println!("bestmove 0000");
        }
      }
      Message::Stop => {
        if let Some(chosen_move) = selected_move {
          println!("bestmove {}", chosen_move.to_string());
          selected_move = None;
        } else if debug {
          println!("info string servererror not currently searching");
        }
      }
      Message::UpdateOption(_, _) | Message::Eval | Message::Bench(_) | Message::NewGame => (),
    }
  }
}
