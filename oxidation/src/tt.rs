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

impl From<CompactEntry> for Entry {
  fn from(value: CompactEntry) -> Self {
    let (scoretype, score) = match value.flags {
      Flags::ExactCentipawn => (ScoreType::Exact, Score::Centipawn(value.raw_score as i32)),
      Flags::LowerCentipawn => (
        ScoreType::LowerBound,
        Score::Centipawn(value.raw_score as i32),
      ),
      Flags::UpperCentipawn => (
        ScoreType::UpperBound,
        Score::Centipawn(value.raw_score as i32),
      ),
      Flags::ExactWin => (ScoreType::Exact, Score::Win(value.raw_score)),
      Flags::LowerWin => (ScoreType::LowerBound, Score::Win(value.raw_score)),
      Flags::UpperWin => (ScoreType::UpperBound, Score::Win(value.raw_score)),
      Flags::ExactLoss => (ScoreType::Exact, Score::Loss(value.raw_score)),
      Flags::LowerLoss => (ScoreType::LowerBound, Score::Loss(value.raw_score)),
      Flags::UpperLoss => (ScoreType::UpperBound, Score::Loss(value.raw_score)),
    };
    Self {
      hash: value.hash as Hash >> 32,
      depth: value.depth,
      movecount: 0,
      scoretype,
      score,
      bestmove: value.bestmove,
    }
  }
}

#[derive(Clone, Copy)]
pub enum Flags {
  ExactCentipawn,
  ExactWin,
  ExactLoss,
  LowerCentipawn,
  LowerWin,
  LowerLoss,
  UpperCentipawn,
  UpperWin,
  UpperLoss,
}

#[derive(Clone, Copy)]
pub struct CompactEntry {
  hash: u32,
  bestmove: Option<Move>,
  raw_score: u32,
  flags: Flags,
  depth: u8,
}

impl From<Entry> for CompactEntry {
  fn from(value: Entry) -> Self {
    let (raw_score, flags) = match (value.score, value.scoretype) {
      (Score::Centipawn(score), ScoreType::Exact) => (score as u32, Flags::ExactCentipawn),
      (Score::Centipawn(score), ScoreType::LowerBound) => (score as u32, Flags::LowerCentipawn),
      (Score::Centipawn(score), ScoreType::UpperBound) => (score as u32, Flags::UpperCentipawn),
      (Score::Win(moves), ScoreType::Exact) => (moves - value.movecount, Flags::ExactWin),
      (Score::Win(moves), ScoreType::LowerBound) => (moves - value.movecount, Flags::LowerWin),
      (Score::Win(moves), ScoreType::UpperBound) => (moves - value.movecount, Flags::UpperWin),
      (Score::Loss(moves), ScoreType::Exact) => (moves - value.movecount, Flags::ExactLoss),
      (Score::Loss(moves), ScoreType::LowerBound) => (moves - value.movecount, Flags::LowerLoss),
      (Score::Loss(moves), ScoreType::UpperBound) => (moves - value.movecount, Flags::UpperLoss),
    };
    Self {
      hash: (value.hash >> 32) as u32,
      bestmove: value.bestmove,
      raw_score,
      flags,
      depth: value.depth,
    }
  }
}

pub struct TranspositionTable {
  entries: Box<[Option<CompactEntry>]>,
  flags: ExtraFlags,
  // the number of entries full
  capacity: usize,
}

impl TranspositionTable {
  // Initialise a tt based on a size in megabytes
  pub fn new(megabytes: usize, board: &Board) -> Self {
    let size = megabytes * 65536;
    let entries = vec![None; size].into_boxed_slice();
    Self {
      entries,
      flags: ExtraFlags::new(board),
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
        if entry.hash == (hash >> 32) as u32 {
          ttmove = entry.bestmove;
          if entry.depth >= depth {
            let mut entry = Entry::from(*entry);
            match entry.score {
              Score::Win(ref mut moves) | Score::Loss(ref mut moves) => {
                *moves += movecount;
              }
              Score::Centipawn(_) => (),
            }
            let cutoff = match entry.scoretype {
              ScoreType::Exact => true,
              ScoreType::LowerBound if entry.score >= beta => true,
              ScoreType::UpperBound if entry.score <= alpha => true,
              _ => false,
            };
            let cutoff = if cutoff { Some(entry.score) } else { None };
            return (cutoff, ttmove);
          }
        }
      }
    }
    (None, ttmove)
  }

  pub fn store(&mut self, entry: Entry) {
    if self.entries.len() > 0 {
      let index = entry.hash as usize % self.entries.len();
      if let Some(old_entry) = self.entries[index] {
        if old_entry.hash != (entry.hash >> 32) as u32
          || entry.scoretype == ScoreType::Exact
          || entry.depth.saturating_add(1) >= old_entry.depth
        {
          self.entries[index] = Some(CompactEntry::from(entry));
        }
      } else {
        self.capacity += 1;
        self.entries[index] = Some(CompactEntry::from(entry));
      }
    }
  }

  // Clears the table if the flags change
  // Call whenever the position to search changes
  // Returns whether the table was cleared
  pub fn new_position(&mut self, position: &Board) -> bool {
    let flags = ExtraFlags::new(position);
    if flags != self.flags {
      self.clear(flags);
      return true;
    }
    false
  }

  pub fn clear(&mut self, flags: ExtraFlags) {
    self.flags = flags;
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
