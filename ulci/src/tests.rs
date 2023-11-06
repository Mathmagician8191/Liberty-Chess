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
  assert!(Score::Centipawn(7.0) > Score::Centipawn(-7.0));
  assert!(Score::Centipawn(7.0) == Score::Centipawn(7.0));
}

#[test]
fn wdl_ordering() {
  assert!(Score::WDL(300, 400, 300) > Score::WDL(200, 300, 500));
  assert!(Score::WDL(300, 400, 300) == Score::WDL(500, 0, 500));
}

#[test]
fn mixed_ordering() {
  assert!(Score::Win(7) > Score::Loss(5));
  assert!(Score::Win(7) > Score::Centipawn(5.0));
  assert!(Score::Win(7) > Score::WDL(100, 800, 100));
  assert!(Score::Loss(7) < Score::Centipawn(5.0));
  assert!(Score::Loss(7) < Score::WDL(100, 800, 100));
}

#[test]
fn undefined_ordering() {
  assert!(Score::Centipawn(0.0).partial_cmp(&Score::WDL(0, 1000, 0)) == None);
}
