use array2d::Array2D;
use liberty_chess::moves::Move;

type HistoryInternals = [Array2D<(i32, Option<Move>)>; 18];

pub struct History {
  white_data: HistoryInternals,
  black_data: HistoryInternals,
}

fn get_data(width: usize, height: usize) -> Array2D<(i32, Option<Move>)> {
  Array2D::filled_with((0, None), height, width)
}

impl History {
  pub fn new(width: usize, height: usize) -> Self {
    let white_data = [(); 18].map(|()| get_data(width, height));
    let black_data = [(); 18].map(|()| get_data(width, height));
    Self {
      white_data,
      black_data,
    }
  }

  // new search, clear the history table
  pub fn clear(&mut self, width: usize, height: usize) {
    for element in &mut self.white_data {
      *element = get_data(width, height);
    }
    for element in &mut self.black_data {
      *element = get_data(width, height);
    }
  }

  pub fn new_position(&mut self, width: usize, height: usize) {
    let array = &self.white_data[0];
    if width != array.num_columns() || height != array.num_rows() {
      self.clear(width, height);
    } else {
      for array in self.white_data.iter_mut() {
        for (item, _) in array.elements_iter_mut() {
          *item /= 2;
        }
      }
      for array in self.black_data.iter_mut() {
        for (item, _) in array.elements_iter_mut() {
          *item /= 2;
        }
      }
    }
  }

  pub fn bonus(&mut self, side: bool, piece: u8, square: (usize, usize), depth: u8) {
    let depth = i32::from(depth);
    let piece = usize::from(piece - 1);
    let bonus = depth * depth;
    if side {
      self.white_data[piece][square].0 += bonus;
    } else {
      self.black_data[piece][square].0 += bonus;
    }
  }

  pub fn malus(&mut self, side: bool, piece: u8, square: (usize, usize), depth: u8) {
    let depth = i32::from(depth);
    let piece = usize::from(piece - 1);
    let malus = depth * depth;
    if side {
      self.white_data[piece][square].0 -= malus;
    } else {
      self.black_data[piece][square].0 -= malus;
    }
  }

  #[must_use]
  pub fn get(&self, side: bool, piece: u8, square: (usize, usize)) -> i32 {
    let piece = usize::from(piece - 1);
    if side {
      self.white_data[piece][square].0
    } else {
      self.black_data[piece][square].0
    }
  }

  #[must_use]
  pub fn get_countermove(&self, side: bool, piece: u8, square: (usize, usize)) -> Option<Move> {
    let piece = usize::from(piece - 1);
    if side {
      self.white_data[piece][square].1
    } else {
      self.black_data[piece][square].1
    }
  }

  pub fn store_countermove(&mut self, side: bool, piece: u8, square: (usize, usize), mv: Move) {
    let piece = usize::from(piece - 1);
    if side {
      self.white_data[piece][square].1 = Some(mv);
    } else {
      self.black_data[piece][square].1 = Some(mv);
    }
  }
}
