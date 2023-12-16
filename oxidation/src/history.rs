use array2d::Array2D;

type HistoryInternals = [Array2D<u32>; 18];

pub struct History {
  white_data: HistoryInternals,
  black_data: HistoryInternals,
}

fn get_data(width: usize, height: usize) -> Array2D<u32> {
  Array2D::filled_with(0, height, width)
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

  pub fn store(&mut self, side: bool, piece: u8, square: (usize, usize), depth: u8) {
    let depth = u32::from(depth);
    let piece = usize::from(piece - 1);
    let bonus = depth * depth;
    if side {
      self.white_data[piece][square] += bonus;
    } else {
      self.black_data[piece][square] += bonus;
    }
  }

  #[must_use]
  pub fn get(&self, side: bool, piece: u8, square: (usize, usize)) -> u32 {
    let piece = usize::from(piece - 1);
    if side {
      self.white_data[piece][square]
    } else {
      self.black_data[piece][square]
    }
  }
}
