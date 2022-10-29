use enum_iterator::Sequence;
use liberty_chess::Board;

#[derive(Clone, Copy, PartialEq, Sequence)]
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
}

impl HelpPage {
  pub fn title(self) -> &'static str {
    match self {
      HelpPage::PawnForward => "Pawns",
      HelpPage::PawnCapture => "Pawns 2",
      HelpPage::PawnDouble => "Pawns 3",
      HelpPage::Knight => "Knight",
      HelpPage::Bishop => "Bishop",
      HelpPage::Rook => "Rook",
      HelpPage::Queen => "Queen",
      HelpPage::King => "King",
      HelpPage::Archbishop => "Archbishop",
      HelpPage::Chancellor => "Chancellor",
      HelpPage::Camel => "Camel",
      HelpPage::Zebra => "Zebra",
      HelpPage::Mann => "Mann",
      HelpPage::Nightrider => "Nightrider",
      HelpPage::Champion => "Champion",
      HelpPage::Centaur => "Centaur",
      HelpPage::Amazon => "Amazon",
      HelpPage::Elephant => "Elephant",
      HelpPage::Obstacle => "Obstacle",
      HelpPage::Wall => "Wall",
      HelpPage::EnPassant => "En passant",
    }
  }

  pub fn board(self) -> Board {
    match self {
      HelpPage::PawnForward => Board::new("7/7/7/7/3P3/7/7 w").unwrap(),
      HelpPage::PawnCapture => Board::new("7/7/2ppp2/3P3/7/7/7 w").unwrap(),
      HelpPage::PawnDouble => Board::new("7/7/7/7/7/3P3/7 w").unwrap(),
      HelpPage::Knight => Board::new("7/7/7/3N3/7/7/7 w").unwrap(),
      HelpPage::Bishop => Board::new("7/ppppppp/7/3B3/7/7/7 w").unwrap(),
      HelpPage::Rook => Board::new("7/ppppppp/7/3R3/7/7/7 w").unwrap(),
      HelpPage::Queen => Board::new("7/ppppppp/7/3Q3/7/7/7 w").unwrap(),
      HelpPage::King => Board::new("7/7/7/3K3/7/7/7 w").unwrap(),
      HelpPage::Archbishop => Board::new("7/ppppppp/7/3A3/7/7/7 w").unwrap(),
      HelpPage::Chancellor => Board::new("7/ppppppp/7/3C3/7/7/7 w").unwrap(),
      HelpPage::Camel => Board::new("7/7/7/3L3/7/7/7 w").unwrap(),
      HelpPage::Zebra => Board::new("7/7/7/3Z3/7/7/7 w").unwrap(),
      HelpPage::Mann => Board::new("7/7/7/3X3/7/7/7 w").unwrap(),
      HelpPage::Nightrider => Board::new("9/9/9/9/4I4/9/9/9/9 w").unwrap(),
      HelpPage::Champion => Board::new("7/7/7/3H3/7/7/7 w").unwrap(),
      HelpPage::Centaur => Board::new("7/7/7/3U3/7/7/7 w").unwrap(),
      HelpPage::Amazon => Board::new("7/ppppppp/7/3M3/7/7/7 w").unwrap(),
      HelpPage::Elephant => Board::new("7/7/7/3E3/7/7/7 w").unwrap(),
      HelpPage::Obstacle => Board::new("7/ppppppp/7/3O3/7/7/7 w").unwrap(),
      HelpPage::Wall => Board::new("7/ppppppp/7/3W3/7/7/7 w").unwrap(),
      HelpPage::EnPassant => Board::new("7/pp1pppp/7/2pP3/7/7/7 w - c5").unwrap(),
    }
  }

  pub fn selected(self) -> (usize, usize) {
    match self {
      HelpPage::PawnForward => (2, 3),
      HelpPage::PawnCapture => (3, 3),
      HelpPage::PawnDouble => (1, 3),
      HelpPage::Knight => (3, 3),
      HelpPage::Bishop => (3, 3),
      HelpPage::Rook => (3, 3),
      HelpPage::Queen => (3, 3),
      HelpPage::King => (3, 3),
      HelpPage::Archbishop => (3, 3),
      HelpPage::Chancellor => (3, 3),
      HelpPage::Camel => (3, 3),
      HelpPage::Zebra => (3, 3),
      HelpPage::Mann => (3, 3),
      HelpPage::Nightrider => (4, 4),
      HelpPage::Champion => (3, 3),
      HelpPage::Centaur => (3, 3),
      HelpPage::Amazon => (3, 3),
      HelpPage::Elephant => (3, 3),
      HelpPage::Obstacle => (3, 3),
      HelpPage::Wall => (3, 3),
      HelpPage::EnPassant => (3, 3),
    }
  }

  pub fn description(self) -> &'static str {
    match self {
      HelpPage::PawnForward => "The pawn moves one square forward.",
      HelpPage::PawnCapture => "The pawn cannot capture forwards, but can move diagonally to capture.",
      HelpPage::PawnDouble => "The pawn can move multiple squares on its first move. The number of squares depends on the gamemode.",
      HelpPage::Knight => "The Knight jumps a set number of squares in each direction, including over other pieces.",
      HelpPage::Bishop => "The Bishop moves diagonally, but cannot go past another piece. The Bishop is confined to squares of the same colour it started on.",
      HelpPage::Rook => "The Rook moves horizontally or vertically, but cannot go past another piece.",
      HelpPage::Queen => "The Queen moves as the combination of the Bishop and the Rook.",
      HelpPage::King => "The King moves one square in any direction, and has a special move called castling (covered later). Putting the King in a position where it cannot escape is the object of the game.",
      HelpPage::Archbishop => "The Archbishop moves as the combination of the Bishop and the Knight.",
      HelpPage::Chancellor => "The Archbishop moves as the combination of the Rook and the Knight.",
      HelpPage::Camel => "The Camel moves like the Knight, only a different number of squares to it. The Camel is confined to squares of a the same colour it started on.",
      HelpPage::Zebra => "The Zebra moves like the Knight, only a different number of squares to it.",
      HelpPage::Mann => "The Mann moves one square in any direction.",
      HelpPage::Nightrider => "The Nightrider can make multiple knight jumps at once in the same direction, but cannot go past another piece on one of the knight-jump destination squares.",
      HelpPage::Champion => "The Champion can go 1 or 2 spaces in any direction, and can jump over other pieces. However, it cannot make a Knight move.",
      HelpPage::Centaur => "The Centaur moves as the combination of the Knight and the Mann.",
      HelpPage::Amazon => "The Amazon moves as a combination of the Queen and the Knight.",
      HelpPage::Elephant => "The Elephant moves like a Mann, but is immune to capture from pieces other than another Elephant or a King.",
      HelpPage::Obstacle => "The Obstacle can teleport to any empty square on the board, but cannot capture other pieces.",
      HelpPage::Wall => "The Wall moves like the Obstacle, but it can only be captured by an Elephant or King",
      HelpPage::EnPassant => "When a pawn moves more than one space, another pawn can capture it as if it had only moved one. This option is only available on the next move.",
    }
  }

  pub fn moved(self) -> Option<[(usize, usize); 2]> {
    match self {
      HelpPage::EnPassant => Some([(3, 2), (5, 2)]),
      _ => None,
    }
  }
}
