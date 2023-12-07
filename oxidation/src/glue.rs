use crate::{get_move_order, search, Output, SearchConfig, State};
use liberty_chess::threading::CompressedBoard;
use std::sync::mpsc::{Receiver, Sender};
use ulci::client::Message;
use ulci::server::UlciResult;
use ulci::SearchTime;

/// Analyse the given position
///
/// Blocks the current thread
pub fn process_position(
  tx: &Sender<UlciResult>,
  receive_message: &Receiver<Message>,
  board: CompressedBoard,
  searchtime: SearchTime,
  mut qdepth: u8,
  state: &mut State,
) -> Option<()> {
  let position = board.load_from_thread();
  state.new_position(&position);
  let mut debug = false;
  while receive_message.try_recv().is_ok() {}
  let config = SearchConfig::new_time(&mut qdepth, searchtime, receive_message, &mut debug);
  let moves = get_move_order(state, &position, &Vec::new());
  let pv = search(state, config, &position, moves, Output::Channel(tx));
  tx.send(UlciResult::AnalysisStopped(pv[0])).ok()?;
  Some(())
}
