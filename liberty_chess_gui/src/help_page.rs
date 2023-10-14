use enum_iterator::Sequence;
use liberty_chess::moves::Move;
use liberty_chess::Board;

#[derive(Clone, Copy, Eq, PartialEq, Sequence)]
pub enum HelpPage {
  PawnForward,
  PawnCapture,
  PawnDouble,
  Knight,
  Bishop,
  Rook,
  Queen,
  King,
  Archbishop,
  Chancellor,
  Camel,
  Zebra,
  Mann,
  Nightrider,
  Champion,
  Centaur,
  Amazon,
  Elephant,
  Obstacle,
  Wall,
  EnPassant,
  ElVaticano,
  Castling,
  Check,
}

impl HelpPage {
  pub const fn title(self) -> &'static str {
    match self {
      Self::PawnForward => "Pawns",
      Self::PawnCapture => "Pawns 2",
      Self::PawnDouble => "Pawns 3",
      Self::Knight => "Knight",
      Self::Bishop => "Bishop",
      Self::Rook => "Rook",
      Self::Queen => "Queen",
      Self::King => "King",
      Self::Archbishop => "Archbishop",
      Self::Chancellor => "Chancellor",
      Self::Camel => "Camel",
      Self::Zebra => "Zebra",
      Self::Mann => "Mann",
      Self::Nightrider => "Nightrider",
      Self::Champion => "Champion",
      Self::Centaur => "Centaur",
      Self::Amazon => "Amazon",
      Self::Elephant => "Elephant",
      Self::Obstacle => "Obstacle",
      Self::Wall => "Wall",
      Self::EnPassant => "En passant",
      Self::ElVaticano => "El Vaticano",
      Self::Castling => "Castling",
      Self::Check => "Check",
    }
  }

  pub fn board(self) -> Board {
    match self {
      Self::PawnForward => Board::new("7/7/7/7/3P3/7/7 w").unwrap(),
      Self::PawnCapture => Board::new("7/7/2ppp2/3P3/7/7/7 w").unwrap(),
      Self::PawnDouble => Board::new("7/7/7/7/7/3P3/7 w").unwrap(),
      Self::Knight => Board::new("7/7/7/3N3/7/7/7 w").unwrap(),
      Self::Bishop => Board::new("7/ppppppp/7/3B3/7/7/7 w").unwrap(),
      Self::Rook => Board::new("7/ppppppp/7/3R3/7/7/7 w").unwrap(),
      Self::Queen => Board::new("7/ppppppp/7/3Q3/7/7/7 w").unwrap(),
      Self::King => Board::new("7/7/7/3K3/7/7/7 w").unwrap(),
      Self::Archbishop => Board::new("7/ppppppp/7/3A3/7/7/7 w").unwrap(),
      Self::Chancellor => Board::new("7/ppppppp/7/3C3/7/7/7 w").unwrap(),
      Self::Camel => Board::new("7/7/7/3L3/7/7/7 w").unwrap(),
      Self::Zebra => Board::new("7/7/7/3Z3/7/7/7 w").unwrap(),
      Self::Mann => Board::new("7/7/7/3X3/7/7/7 w").unwrap(),
      Self::Nightrider => Board::new("9/9/9/9/4I4/9/9/9/9 w").unwrap(),
      Self::Champion => Board::new("7/7/7/3H3/7/7/7 w").unwrap(),
      Self::Centaur => Board::new("7/7/7/3U3/7/7/7 w").unwrap(),
      Self::Amazon => Board::new("7/ppppppp/7/3M3/7/7/7 w").unwrap(),
      Self::Elephant => Board::new("7/7/7/3E3/7/7/7 w").unwrap(),
      Self::Obstacle => Board::new("7/ppppppp/7/3O3/7/7/7 w").unwrap(),
      Self::Wall => Board::new("7/ppppppp/7/3W3/7/7/7 w").unwrap(),
      Self::EnPassant => {
        let mut board = Board::new("7/pp1pppp/7/2pP3/7/7/7 w - c5").unwrap();
        board.last_move = Some(Move::new((3, 2), (5, 2)));
        board
      }
      Self::ElVaticano => Board::new("7/7/7/2BpB2/7/7/7 w - c5").unwrap(),
      Self::Castling => Board::new("8/8/8/8/8/8/8/R3K2R w KQ").unwrap(),
      Self::Check => Board::new("3r3/7/7/7/7/7/3K3 w").unwrap(),
    }
  }

  pub const fn selected(self) -> (usize, usize) {
    match self {
      Self::PawnForward => (2, 3),
      Self::PawnDouble => (1, 3),
      Self::Nightrider => (4, 4),
      Self::ElVaticano => (3, 2),
      Self::Castling => (0, 4),
      Self::Check => (0, 3),
      _ => (3, 3),
    }
  }

  pub const fn description(self) -> &'static str {
    match self {
      Self::PawnForward => "The pawn moves one square forward.",
      Self::PawnCapture => "The pawn cannot capture forwards, but can move diagonally to capture.",
      Self::PawnDouble => "The pawn can move multiple squares on its first move. The number of squares depends on the gamemode.",
      Self::Knight => "The Knight jumps a set number of squares in each direction, including over other pieces.",
      Self::Bishop => "The Bishop moves diagonally, but cannot go past another piece. The Bishop is confined to squares of the same colour it started on.",
      Self::Rook => "The Rook moves horizontally or vertically, but cannot go past another piece.",
      Self::Queen => "The Queen moves as the combination of the Bishop and the Rook.",
      Self::King => "The King moves one square in any direction, and has a special move called castling (covered later). Putting the King in a position where it cannot escape is the object of the game.",
      Self::Archbishop => "The Archbishop moves as the combination of the Bishop and the Knight.",
      Self::Chancellor => "The Chancellor moves as the combination of the Rook and the Knight.",
      Self::Camel => "The Camel moves like the Knight, only a different number of squares to it. The Camel is confined to squares of a the same colour it started on.",
      Self::Zebra => "The Zebra moves like the Knight, only a different number of squares to it.",
      Self::Mann => "The Mann moves one square in any direction.",
      Self::Nightrider => "The Nightrider can make multiple knight jumps at once in the same direction, but cannot go past another piece on one of the knight-jump destination squares.",
      Self::Champion => "The Champion can go 1 or 2 spaces in any direction, and can jump over other pieces. However, it cannot make a Knight move.",
      Self::Centaur => "The Centaur moves as the combination of the Knight and the Mann.",
      Self::Amazon => "The Amazon moves as a combination of the Queen and the Knight.",
      Self::Elephant => "The Elephant moves like a Mann, but is immune to capture except by another Elephant or a King.",
      Self::Obstacle => "The Obstacle can teleport to any empty square on the board, but cannot capture other pieces.",
      Self::Wall => "The Wall moves like the Obstacle, but is immune to capture except by an Elephant or King",
      Self::EnPassant => "When a pawn moves more than one space, another pawn can capture it as if it had only moved one. This option is only available on the next move.",
      Self::ElVaticano => "2 bishops that are 2 squares apart orthogonally can capture the piece between them. This is represented by one bishop capturing the other one.",
      Self::Castling => "If a King hasn't moved, it can move 2 squares to castle with another piece. The piece that can be castled with is configurable, and the piece moves to the other side of the king.",
      Self::Check => "If a King is in danger of being captured, it is in check. The King must get out of check on the next move. If that is not possible, the game ends in checkmate.",
    }
  }
}
