use liberty_chess::moves::Move;
use liberty_chess::Board;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::io::{stdin, stdout, BufReader};
use std::thread;
use std::{collections::HashMap, sync::mpsc::channel};
use ulci::{startup_client, ClientInfo, ClientMessage, SearchTime, STARTPOS};

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
  let mut position = Board::new(STARTPOS).unwrap();
  let mut debug = false;
  let mut selected_move = None;
  thread::spawn(move || startup_client(tx, info, input, output));
  while let Ok(message) = rx.recv() {
    match message {
      ClientMessage::SetDebug(new_debug) => debug = new_debug,
      ClientMessage::UpdatePosition(board) => position = Board::load_from_thread(*board),
      ClientMessage::Go(settings) => {
        let moves = position.generate_legal();
        let moves = moves.iter().filter_map(|board| board.last_move);
        let moves: Vec<Move> = if !settings.moves.is_empty() {
          moves.filter(|m| settings.moves.contains(m)).collect()
        } else {
          moves.collect()
        };
        selected_move = moves.choose(&mut thread_rng()).copied();
        if let Some(chosen_move) = selected_move {
          match settings.time {
            SearchTime::FixedTime(_) | SearchTime::Increment(_, _) => {
              println!(
                "info depth 1 score cp 0 time 0 nodes 1 nps 1 pv {}",
                chosen_move.to_string()
              );
              println!("bestmove {}", chosen_move.to_string());
              selected_move = None;
            }
            SearchTime::Depth(depth) => {
              println!(
                "info depth {depth} score cp 0 time 0 nodes 1 nps 1 pv {}",
                chosen_move.to_string()
              );
              println!("bestmove {}", chosen_move.to_string());
              selected_move = None;
            }
            SearchTime::Nodes(nodes) => {
              println!(
                "info depth 1 score cp 0 time 0 nodes {nodes} nps 1 pv {}",
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
              "info servererror no legal moves from position {}",
              position.to_string()
            );
          } else {
            println!(
              "info servererror all search moves are illegal in position {}",
              position.to_string()
            );
          }
        }
      }
      ClientMessage::Stop => {
        if let Some(chosen_move) = selected_move {
          println!("bestmove {}", chosen_move.to_string());
          selected_move = None;
        } else if debug {
          println!("info servererror not currently searching");
        }
      }
      ClientMessage::UpdateOption(_) => (),
    }
  }
}
