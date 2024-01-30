use rand::seq::SliceRandom;
use rand::{thread_rng, Rng};

/// Randomly generates a board given the specified parameters
pub fn generate(width: usize, height: usize, piece_options: &str, spawn_king: bool) -> String {
  // The gap between the white and black pieces
  let gap = height - 4;

  // The available pieces to choose from
  let pieces = piece_options.to_lowercase().chars().collect::<Vec<char>>();

  let mut rng = thread_rng();

  // Get the pieces on the board
  let mut pieces: Vec<char> = (0..width)
    .map(|_| *pieces.choose(&mut rng).unwrap_or(&'n'))
    .collect();

  // Add a king to the board
  if spawn_king {
    pieces[rng.gen_range(0..width)] = 'k';
  }

  let pieces = pieces.iter().collect::<String>();

  // Build and return the final L-FEN
  let mut result = pieces.clone();
  result.push('/');
  result += &"p".repeat(width);
  result.push('/');
  result += &(width.to_string() + "/").repeat(gap);
  result += &"P".repeat(width);
  result.push('/');
  result += &pieces.to_uppercase();
  result += " w KQkq - 0 1 - ";

  // make piece options promotion options
  result += piece_options;

  result
}
