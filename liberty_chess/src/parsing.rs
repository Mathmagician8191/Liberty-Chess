use crate::{
  Board, Piece, AMAZON, ARCHBISHOP, BISHOP, CAMEL, CENTAUR, CHAMPION, CHANCELLOR, ELEPHANT, KING,
  KNIGHT, MANN, NIGHTRIDER, OBSTACLE, PAWN, QUEEN, ROOK, SQUARE, WALL, ZEBRA,
};
use array2d::Array2D;
use std::str::FromStr;

/// An enum to represent the reasons for an L-FEN to be invalid.
#[derive(Debug)]
pub enum FenError {
  /// An unrecognised piece was encountered
  InvalidPiece(char),
  /// The board has a non-uniform width
  NonRectangular,
  /// The board has a width or height less than 2
  Size,
}

impl ToString for FenError {
  fn to_string(&self) -> String {
    match self {
      Self::InvalidPiece(c) => format!("Invalid piece found: {c}"),
      Self::NonRectangular => "Non-rectangular board found".to_owned(),
      Self::Size => "Board must be between 2x2 and 65536x65536".to_owned(),
    }
  }
}

impl FromStr for Board {
  type Err = FenError;

  fn from_str(fen: &str) -> Result<Self, Self::Err> {
    Self::new(fen)
  }
}

impl ToString for Board {
  fn to_string(&self) -> String {
    // save board layout
    let mut rows = Vec::new();
    for row in self.pieces.rows_iter() {
      let mut squares = 0;
      let mut output = String::new();
      for piece in row {
        if piece == &SQUARE {
          squares += 1;
        } else {
          if squares > 0 {
            output += &squares.to_string();
            squares = 0;
          }
          output.push(to_char(*piece));
        }
      }
      if squares > 0 {
        output += &squares.to_string();
      }
      rows.push(output);
    }

    rows.reverse();

    let mut result = rows.join("/");

    // save side to move
    result += if self.to_move { " w " } else { " b " };

    // save castling rights
    let mut needs_castling = true;
    for i in 0..4 {
      if self.castling[i] {
        needs_castling = false;
        result.push(match i {
          0 => 'K',
          1 => 'Q',
          2 => 'k',
          3 => 'q',
          _ => unreachable!(),
        });
      }
    }

    if needs_castling {
      result.push('-');
    }

    result.push(' ');

    // save en passant
    match self.en_passant {
      Some([column, row_min, row_max]) => result += &to_indices(column, row_min, row_max),
      None => result.push('-'),
    }

    // save halfmove clock and full move count
    result.push(' ');
    result += &self.halfmoves.to_string();
    result.push(' ');
    result += &self.moves.to_string();

    // save optional fields in reverse order
    // because previous ones are required
    let mut optional = Vec::new();

    if self.friendly_fire {
      optional.push("ff".to_owned());
    }

    let custom_promotion =
      self.friendly_fire || self.shared_data.promotion_options != [QUEEN, ROOK, BISHOP, KNIGHT];

    // save promotion options
    if custom_promotion {
      let mut promotion = String::new();
      for piece in &self.shared_data.promotion_options {
        promotion.push(to_char(-1 * piece));
      }
      optional.push(promotion);
    }

    // assemble misc options (also reversed)
    let mut misc = Vec::new();
    let mut misc_fields = if self.king_column == self.width() - 1 {
      false
    } else {
      misc.push(self.king_column.to_string());
      true
    };

    if misc_fields || self.queen_column != 0 {
      misc.push(self.queen_column.to_string());
      misc_fields = true;
    }

    if misc_fields || self.castle_row != 0 {
      misc.push((self.castle_row + 1).to_string());
      misc_fields = true;
    }

    if misc_fields || self.pawn_row != 2 {
      misc.push(self.pawn_row.to_string());
      misc_fields = true;
    }

    if misc_fields || self.pawn_moves != 2 {
      misc.push(self.pawn_moves.to_string());
    }

    if misc.is_empty() && custom_promotion {
      misc.push("-".to_owned());
    }

    if !misc.is_empty() {
      misc.reverse();
      optional.push(misc.join(","));
    }

    if !optional.is_empty() {
      optional.reverse();
      result.push(' ');
      result += &optional.join(" ");
    }

    result
  }
}

/// Converts a group of characters into pieces
///
/// Ignores invalid pieces
#[must_use]
pub fn from_chars(chars: &str) -> Vec<Piece> {
  let mut pieces = Vec::with_capacity(chars.len());
  for c in chars.chars() {
    if let Ok(piece) = to_piece(c) {
      pieces.push(piece.abs());
    }
  }
  pieces
}

/// Converts a character to a `Piece`
///
/// # Errors
///
/// Will return `Err` if `c` is not a recognised piece
pub const fn to_piece(c: char) -> Result<Piece, FenError> {
  let multiplier: Piece = if c.is_ascii_uppercase() { 1 } else { -1 };

  let piece_type = match c.to_ascii_lowercase() {
    'p' => PAWN,
    'n' => KNIGHT,
    'b' => BISHOP,
    'r' => ROOK,
    'q' => QUEEN,
    'k' => KING,
    'a' => ARCHBISHOP,
    'c' => CHANCELLOR,
    'l' => CAMEL,
    'z' => ZEBRA,
    'x' => MANN,
    'i' => NIGHTRIDER,
    'h' => CHAMPION,
    'u' => CENTAUR,
    'm' => AMAZON,
    'e' => ELEPHANT,
    'o' => OBSTACLE,
    'w' => WALL,
    _ => return Err(FenError::InvalidPiece(c.to_ascii_lowercase())),
  };
  Ok(multiplier * piece_type)
}

/// Converts a piece to a character
#[must_use]
pub const fn to_char(piece: Piece) -> char {
  let c = match piece.abs() {
    PAWN => 'p',
    KNIGHT => 'n',
    BISHOP => 'b',
    ROOK => 'r',
    QUEEN => 'q',
    KING => 'k',
    ARCHBISHOP => 'a',
    CHANCELLOR => 'c',
    CAMEL => 'l',
    ZEBRA => 'z',
    MANN => 'x',
    NIGHTRIDER => 'i',
    CHAMPION => 'h',
    CENTAUR => 'u',
    AMAZON => 'm',
    ELEPHANT => 'e',
    OBSTACLE => 'o',
    WALL => 'w',
    _ => unreachable!(),
  };
  if piece > 0 {
    c.to_ascii_uppercase()
  } else {
    c
  }
}

/// Convert a `Piece` into a string representing the type of piece it is.
#[must_use]
pub fn to_name(piece: Piece) -> &'static str {
  match piece.abs() {
    PAWN => "Pawn",
    KNIGHT => "Knight",
    BISHOP => "Bishop",
    ROOK => "Rook",
    QUEEN => "Queen",
    KING => "King",
    ARCHBISHOP => "Archbishop",
    CHANCELLOR => "Chancellor",
    CAMEL => "Camel",
    ZEBRA => "Zebra",
    MANN => "Mann",
    NIGHTRIDER => "Nightrider",
    CHAMPION => "Champion",
    CENTAUR => "Centaur",
    AMAZON => "Amazon",
    ELEPHANT => "Elephant",
    OBSTACLE => "Obstacle",
    WALL => "Wall",
    _ => unreachable!(),
  }
}

pub(crate) fn update_column(column: &mut usize, c: char) {
  *column *= 26;
  *column += c as usize + 1 - 'a' as usize;
}

pub(crate) fn update_row(row: &mut usize, c: char) {
  *row *= 10;
  *row += c as usize - '0' as usize;
}

pub(crate) fn get_indices(algebraic: &str) -> Option<[usize; 3]> {
  if algebraic == "-" {
    return None;
  }
  let mut column = 0;
  let mut row = 0;
  let mut row_start = 0;
  let mut found_dash = false;
  let iterator = algebraic.chars();
  for c in iterator {
    match c {
      _ if c.is_ascii_lowercase() => update_column(&mut column, c),
      _ if c.is_ascii_digit() => {
        if found_dash {
          update_row(&mut row_start, c);
        } else {
          update_row(&mut row, c);
        }
      }
      '-' => found_dash = true,
      _ => (),
    }
  }
  if column == 0 || row == 0 {
    None
  } else if row_start == 0 {
    Some([column - 1, row - 1, row - 1])
  } else {
    Some([column - 1, row - 1, row_start - 1])
  }
}

const fn get_letter(letter: usize) -> char {
  (letter as u8 + b'a') as char
}

/// Convert a file to its representation as letters
#[must_use]
pub fn to_letters(mut column: usize) -> Vec<char> {
  let mut result = Vec::new();
  while column > 26 - usize::from(result.is_empty()) {
    if !result.is_empty() {
      column -= 1;
    }
    let c = get_letter(column % 26);
    result.push(c);
    column /= 26;
  }
  if !result.is_empty() {
    column -= 1;
  }
  result.push(get_letter(column));
  result.reverse();
  result
}

pub(crate) fn to_indices(column: usize, row_min: usize, row_max: usize) -> String {
  let result = to_letters(column);

  if row_min == row_max {
    format!("{}{}", result.iter().collect::<String>(), row_min + 1)
  } else {
    format!(
      "{}{}-{}",
      result.iter().collect::<String>(),
      row_min + 1,
      row_max + 1
    )
  }
}

// returns a board with default values for parameters
pub(crate) fn process_board(
  board: &str,
) -> Result<
  (
    Array2D<Piece>,
    Vec<(usize, usize)>,
    Vec<(usize, usize)>,
    usize,
    usize,
  ),
  FenError,
> {
  let rows: Vec<&str> = board.split('/').collect();

  let height = rows.len();
  let mut width: Option<usize> = None;
  let mut pieces = Vec::new();
  let mut white_kings = Vec::new();
  let mut black_kings = Vec::new();
  let mut white_pieces = 0;
  let mut black_pieces = 0;

  for row in (0..height).rev() {
    let string = rows[row];
    let mut squares = 0;
    let mut vec = Vec::new();

    for c in string.chars() {
      if c.is_ascii_digit() {
        squares *= 10;
        squares += c as usize - '0' as usize;
      } else {
        if squares > 0 {
          vec.append(&mut vec![0; squares]);
          squares = 0;
        }
        let piece = to_piece(c)?;
        if piece.abs() == KING {
          if piece > 0 {
            white_kings.push((height - row - 1, vec.len()));
          } else {
            black_kings.push((height - row - 1, vec.len()));
          }
        } else if piece > 0 {
          white_pieces += 1;
        } else {
          black_pieces += 1;
        }
        vec.push(piece);
      }
    }
    if squares > 0 {
      vec.append(&mut vec![0; squares]);
    }
    if let Some(i) = width {
      if vec.len() != i {
        Err(FenError::NonRectangular)?;
      }
    } else {
      width = Some(vec.len());
    }
    pieces.push(vec);
  }

  let width = width.unwrap_or(0);
  if width < 2 || height < 2 || width > 65536 || height > 65536 {
    Err(FenError::Size)?;
  }

  Ok((
    Array2D::from_rows(&pieces),
    white_kings,
    black_kings,
    white_pieces,
    black_pieces,
  ))
}
