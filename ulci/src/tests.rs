use crate::Score;

#[test]
fn win_ordering() {
  assert!(Score::Win(7) < Score::Win(5));
  assert!(Score::Win(7) == Score::Win(7));
}

#[test]
fn loss_ordering() {
  assert!(Score::Loss(7) > Score::Loss(5));
  assert!(Score::Loss(7) == Score::Loss(7));
}

#[test]
fn centipawn_ordering() {
  assert!(Score::Centipawn(7) > Score::Centipawn(-7));
  assert!(Score::Centipawn(7) == Score::Centipawn(7));
}

#[test]
fn mixed_ordering() {
  assert!(Score::Win(7) > Score::Loss(5));
  assert!(Score::Win(7) > Score::Centipawn(5));
  assert!(Score::Loss(7) < Score::Centipawn(5));
}
