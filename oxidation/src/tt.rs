use std::cmp::max;

use liberty_chess::moves::Move;
use liberty_chess::{Hash, ExtraFlags, Board};
use ulci::Score;

#[derive(Clone, Copy, PartialEq)]
pub enum ScoreType {
  Exact(Score),
  LowerBound(Score),
  UpperBound(Score),
}

#[derive(Clone, Copy)]
pub struct Entry {
  pub hash: Hash,
  pub depth: u8,
  pub movecount: u16,
  pub score: ScoreType,
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
    let size = megabytes * 12_500;
    let entries = vec![None; size].into_boxed_slice();
    Self {
      entries,
      flags: None,
      capacity: 0,
    }
  }

  pub fn get(&self, hash: Hash, movecount: u16) -> Option<Entry> {
    let index = hash as usize % self.entries.len();
    if let Some(entry) = &self.entries[index] {
      if entry.hash == hash {
        let mut entry = *entry;
        if movecount != entry.movecount {
          let score = match entry.score {
            ScoreType::Exact(ref mut score) | ScoreType::LowerBound(ref mut score) | ScoreType::UpperBound(ref mut score) => score,
          };
          match score {
            Score::Win(moves) | Score::Loss(moves) => {
              if movecount > entry.movecount {
                *moves += movecount - entry.movecount;
              } else {
                *moves = moves.saturating_sub(entry.movecount - movecount);
              }
            }
            _ => (),
          }
        }
        return Some(entry)
      }
    }
    None
  }

  pub fn store(&mut self, entry: Entry) {
    let index = entry.hash as usize % self.entries.len();
    if self.entries[index].is_none() {
      self.capacity += 1;
    }
    self.entries[index] = Some(entry);
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
