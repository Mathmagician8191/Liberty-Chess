use crate::history::History;
use crate::parameters::Parameters;
use liberty_chess::moves::Move;
use liberty_chess::Board;

enum Stage {
  TTmove,
  PendingGeneration,
  Captures,
  Killer,
  CounterMove,
  SortQuiets,
  Quiets,
}

pub struct MovePicker {
  stage: Stage,
  ttmove: Option<Move>,
  killer: Option<Move>,
  searched_countermove: Option<Move>,
  captures: Vec<(Move, u8, u8)>,
  quiets: Vec<Move>,
}

impl MovePicker {
  pub fn new() -> Self {
    Self {
      stage: Stage::TTmove,
      ttmove: None,
      killer: None,
      searched_countermove: None,
      captures: Vec::new(),
      quiets: Vec::new(),
    }
  }

  pub fn init(&mut self, ttmove: Option<Move>) {
    self.stage = Stage::TTmove;
    self.ttmove = ttmove;
    self.searched_countermove = None;
    self.captures.clear();
    self.quiets.clear();
  }

  pub fn store_killer(&mut self, killer: Move) {
    self.killer = Some(killer);
  }

  // Returns pseudolegal move and whether the move is a capture
  pub fn pick_move(
    &mut self,
    history: &History,
    parameters: &Parameters<i32>,
    board: &Board,
  ) -> Option<(Move, bool)> {
    loop {
      match self.stage {
        Stage::TTmove => {
          self.stage = Stage::PendingGeneration;
          if let Some(ttmove) = self.ttmove {
            if board.check_pseudolegal(ttmove.start(), ttmove.end()) {
              let capture = board.get_piece(ttmove.end());
              let is_capture = capture != 0 && ((capture > 0) != board.to_move());
              return Some((ttmove, is_capture));
            }
          }
        }
        Stage::PendingGeneration => {
          self.stage = Stage::Captures;
          board.generate_pseudolegal(&mut self.captures, &mut self.quiets);
          self.captures.sort_by_key(|(_, piece, capture)| {
            100 * parameters.pieces[usize::from(*capture - 1)].0
              - parameters.pieces[usize::from(*piece - 1)].0
          });
        }
        Stage::Captures => {
          if let Some((capture, _, _)) = self.captures.pop() {
            if Some(capture) != self.ttmove {
              return Some((capture, true));
            }
          } else {
            self.stage = Stage::Killer;
          }
        }
        Stage::Killer => {
          self.stage = Stage::CounterMove;
          if let Some(killer) = self.killer {
            let capture = board.get_piece(killer.end());
            let is_capture = capture != 0 && ((capture > 0) != board.to_move());
            if Some(killer) != self.ttmove
              && !is_capture
              && board.check_pseudolegal(killer.start(), killer.end())
            {
              return Some((killer, false));
            }
          }
        }
        Stage::CounterMove => {
          self.stage = Stage::SortQuiets;
          if let Some(last_move) = board.last_move {
            let piece = board.get_piece(last_move.end()).unsigned_abs();
            if let Some(countermove) =
              history.get_countermove(board.to_move(), piece, last_move.end())
            {
              let capture = board.get_piece(countermove.end());
              let is_capture = capture != 0 && ((capture > 0) != board.to_move());
              if Some(countermove) != self.ttmove
                && Some(countermove) != self.killer
                && !is_capture
                && board.check_pseudolegal(countermove.start(), countermove.end())
              {
                self.searched_countermove = Some(countermove);
                return Some((countermove, false));
              }
            }
          }
        }
        Stage::SortQuiets => {
          self.stage = Stage::Quiets;
          self.quiets.sort_by_key(|mv| {
            history.get(
              board.to_move(),
              board.get_piece(mv.start()).unsigned_abs(),
              mv.end(),
            )
          });
        }
        Stage::Quiets => {
          if let Some(quiet) = self.quiets.pop() {
            let some_quiet = Some(quiet);
            if some_quiet != self.ttmove
              && some_quiet != self.killer
              && some_quiet != self.searched_countermove
            {
              return Some((quiet, false));
            }
          } else {
            return None;
          }
        }
      }
    }
  }
}
