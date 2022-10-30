use liberty_chess::Board;
use std::io;

fn main() {
  loop {
    let mut input = String::new();
    let size = io::stdin()
      .read_line(&mut input)
      .expect("Failed to read from stdin");
    let trimmed = input.trim();
    if size == 0 || trimmed.starts_with('q') {
      break;
    }
    println!("{:?}", Board::new(&input));
  }
}
