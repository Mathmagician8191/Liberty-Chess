use crate::{search, Output, SearchConfig, State};
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
  state: &mut State,
  multipv: u16,
) -> Option<()> {
  let mut position = board.load_from_thread();
  state.new_position(&position);
  let mut debug = false;
  while receive_message.try_recv().is_ok() {}
  let mut config = SearchConfig::new_time(&position, searchtime, receive_message, &mut debug);
  let pv = search(
    state,
    &mut config,
    &mut position,
    &[],
    multipv,
    Output::Channel(tx),
  );
  tx.send(UlciResult::AnalysisStopped(pv[0])).ok()?;
  Some(())
}
