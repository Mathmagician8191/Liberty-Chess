use crate::Board;

/// The starting position
pub const STARTPOS: &str = "rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w KQkq - 0 1";
/// 10x8 Capablanca's chess
pub const CAPABLANCA_RECTANGLE: &str =
  "rnabqkbcnr/pppppppppp/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 - qcarbn";
/// 10x10 Capablanca's chess
pub const CAPABLANCA: &str =
  "rnabqkbcnr/pppppppppp/10/10/10/10/10/10/PPPPPPPPPP/RNABQKBCNR w KQkq - 0 1 3 qcarbn";
/// Liberty chess
pub const LIBERTY_CHESS: &str = "ruabhqkhbcur/wlzenxxnezlw/pppppppppppp/12/12/12/12/12/12/PPPPPPPPPPPP/WLZENXXNEZLW/RUABHQKHBCUR w KQkq - 0 1 3,3 qcaehurwbznxl";
/// Mini chess
pub const MINI: &str = "qkbnr/ppppp/5/5/PPPPP/QKBNR w Kk - 0 1 1";
/// Mongol chess
pub const MONGOL: &str = "nnnnknnn/pppppppp/8/8/8/8/PPPPPPPP/NNNNKNNN w - - 0 1 - iznl";
/// African chess
pub const AFRICAN: &str = "lnzekznl/pppppppp/8/8/8/8/PPPPPPPP/LNZEKZNL w - - 0 1 - enzl";
/// Narnia chess
pub const NARNIA: &str = "uuqkkquu/pppppppp/8/8/8/8/PPPPPPPP/UUQKKQUU w - - 0 1 - u";
/// Trump chess
pub const TRUMP: &str = "rwwwkwwr/pppppppp/8/8/8/8/PPPPPPPP/RWWWKWWR w KQkq - 0 1 - mrw";
/// Loaded board
pub const LOADED_BOARD: &str =
  "rrrqkrrr/bbbbbbbb/nnnnnnnn/pppppppp/PPPPPPPP/NNNNNNNN/BBBBBBBB/RRRQKRRR w KQkq - 0 1";
/// 2 standard starting positions side by side
pub const DOUBLE_CHESS: &str =
  "rnbqkbnrrnbqkbnr/pppppppppppppppp/16/16/16/16/PPPPPPPPPPPPPPPP/RNBQKBNRRNBQKBNR w KQkq - 0 1";
/// Horde
pub const HORDE: &str =
  "rnbqkbnr/pppppppp/8/1PP2PP1/PPPPPPPP/PPPPPPPP/PPPPPPPP/PPPPPPPP w kq - 0 1";

/// The starting position
#[must_use]
// Should never panic
#[allow(clippy::missing_panics_doc)]
pub fn get_startpos() -> Board {
  Board::new(STARTPOS).unwrap()
}
