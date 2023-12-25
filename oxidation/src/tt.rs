use liberty_chess::moves::Move;
use liberty_chess::{Board, ExtraFlags, Hash};
use std::cmp::max;
use ulci::Score;

#[derive(Clone, Copy, Eq, PartialEq)]
pub enum ScoreType {
  Exact,
  LowerBound,
  UpperBound,
}

#[derive(Clone, Copy)]
pub struct Entry {
  pub hash: Hash,
  pub depth: u8,
  pub movecount: u32,
  pub scoretype: ScoreType,
  pub score: Score,
  pub bestmove: Option<Move>,
}

pub struct TranspositionTable {
  entries: Box<[Option<Entry>]>,
  flags: Option<ExtraFlags>,
  // the number of entries full
  capacity: usize,
}

impl TranspositionTable {
  // Initialise a tt based on a size in megabytes
  pub fn new(megabytes: usize) -> Self {
    let size = megabytes * 31_250;
    let entries = vec![None; size].into_boxed_slice();
    Self {
      entries,
      flags: None,
      capacity: 0,
    }
  }

  pub fn get(
    &self,
    hash: Hash,
    movecount: u32,
    alpha: Score,
    beta: Score,
    depth: u8,
  ) -> (Option<Score>, Option<Move>) {
    let mut ttmove = None;
    if self.entries.len() > 0 {
      let index = hash as usize % self.entries.len();
      if let Some(entry) = &self.entries[index] {
        if entry.hash == hash {
          ttmove = entry.bestmove;
          if entry.depth >= depth {
            let mut entry = *entry;
            if movecount != entry.movecount {
              match entry.score {
                Score::Win(ref mut moves) | Score::Loss(ref mut moves) => {
                  if movecount > entry.movecount {
                    *moves += movecount - entry.movecount;
                  } else {
                    *moves = moves.saturating_sub(entry.movecount - movecount);
                  }
                }
                Score::Centipawn(_) => (),
              }
            }
            return match entry.scoretype {
              ScoreType::Exact => (Some(entry.score), ttmove),
              ScoreType::LowerBound if entry.score >= beta => (Some(beta), ttmove),
              ScoreType::UpperBound if entry.score <= alpha => (Some(alpha), ttmove),
              _ => (None, ttmove),
            };
          }
        }
      }
    }
    (None, ttmove)
  }

  pub fn store(&mut self, entry: Entry) {
    if self.entries.len() > 0 {
      let index = entry.hash as usize % self.entries.len();
      if self.entries[index].is_none() {
        self.capacity += 1;
      }
      self.entries[index] = Some(entry);
    }
  }

  // Clears the table if the flags change
  // Call whenever the position to search changes
  // Returns whether the table was cleared
  pub fn new_position(&mut self, position: &Board) -> bool {
    let flags = ExtraFlags::new(position);
    if let Some(old_flags) = &self.flags {
      if flags != *old_flags {
        self.flags = Some(flags);
        self.clear();
        return true;
      }
    }
    self.flags = Some(flags);
    false
  }

  pub fn clear(&mut self) {
    if self.capacity > 0 {
      for entry in self.entries.iter_mut() {
        *entry = None;
      }
      self.capacity = 0;
    }
  }

  pub fn capacity(&self) -> usize {
    self.capacity * 1000 / max(self.entries.len(), 1)
  }
}
