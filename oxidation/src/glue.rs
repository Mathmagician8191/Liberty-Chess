use crate::{get_move_order, search, SearchConfig, State};
use liberty_chess::threading::CompressedBoard;
use std::sync::mpsc::{Receiver, Sender};
use ulci::client::Message;
use ulci::server::UlciResult;
use ulci::SearchTime;

/// Start up an engine embedded in the program
///
/// Has limited error handling
///
/// Blocks the current thread
pub fn startup(
  rx: &Receiver<(CompressedBoard, SearchTime)>,
  tx: &Sender<UlciResult>,
  receive_message: &Receiver<Message>,
  megabytes: usize,
  mut qdepth: u8,
) -> Option<()> {
  let mut state = State::new(megabytes);
  while let Ok((board, searchtime)) = rx.recv() {
    let position = board.load_from_thread();
    let mut debug = false;
    while receive_message.try_recv().is_ok() {}
    let config = SearchConfig::new_time(&mut qdepth, searchtime, receive_message, &mut debug);
    let moves = get_move_order(&position, &Vec::new());
    let pv = search(&mut state, config, &position, moves, None);
    tx.send(UlciResult::AnalysisStopped(pv[0])).ok()?;
  }
  Some(())
}
