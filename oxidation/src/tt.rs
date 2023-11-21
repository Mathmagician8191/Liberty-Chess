use std::cmp::max;

use liberty_chess::moves::Move;
use liberty_chess::{Board, ExtraFlags, Hash};
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
  pub movecount: u16,
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
    movecount: u16,
    alpha: Score,
    beta: Score,
    depth: u8,
  ) -> Option<(Vec<Move>, Score)> {
    if self.entries.len() > 0 {
      let index = hash as usize % self.entries.len();
      if let Some(entry) = &self.entries[index] {
        if entry.depth >= depth && entry.hash == hash {
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
              _ => (),
            }
          }
          let mut pv = Vec::new();
          if let Some(bestmove) = entry.bestmove {
            pv.push(bestmove);
          }
          return match entry.scoretype {
            ScoreType::Exact => Some((pv, entry.score)),
            ScoreType::LowerBound if entry.score >= beta => Some((pv, beta)),
            ScoreType::UpperBound if entry.score <= alpha => Some((pv, alpha)),
            _ => None,
          };
        }
      }
      None
    } else {
      None
    }
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

  fn clear(&mut self) {
    for entry in self.entries.iter_mut() {
      *entry = None;
    }
    self.capacity = 0;
  }

  pub fn capacity(&self) -> usize {
    self.capacity * 1000 / max(self.entries.len(), 1)
  }
}
