use crate::themes::Colours;
use crate::{LibertyChessGUI, Screen};
use eframe::egui::{
  pos2, Align2, Color32, Context, FontId, PointerButton, Pos2, Rect, Response, Rounding, Sense,
  Shape, Ui, Vec2,
};
use liberty_chess::{to_letters, Board, Piece};

#[cfg(feature = "sound")]
use liberty_chess::Gamestate;
#[cfg(feature = "sound")]
use sound::Effect;

#[cfg(feature = "music")]
use crate::get_dramatic;

//UV that does nothing
const UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
const NUMBER_SCALE: f32 = 5.0;

pub(crate) fn draw_board(
  gui: &mut LibertyChessGUI,
  ctx: &Context,
  ui: &mut Ui,
  gamestate: &Board,
  clickable: bool,
  flipped: bool,
) {
  let rows = gamestate.height();
  let cols = gamestate.width();
  let (size, board_size) = get_size(ctx, rows as f32, cols as f32);
  let sense = if clickable {
    Sense::click_and_drag()
  } else {
    Sense::hover()
  };
  let (response, painter) = ui.allocate_painter(board_size, sense);
  let board_rect = response.rect;
  painter.rect_filled(board_rect, Rounding::none(), Colours::WhiteSquare.value());
  if let Some(location) = response.interact_pointer_pos() {
    let hover = get_hovered(board_rect, location, size as usize, flipped, gamestate);
    register_response(gui, gamestate, &response, hover);
  }
  let (dragged, offset) = if let Some((coords, offset)) = gui.drag {
    (Some(coords), offset)
  } else {
    (None, Pos2::default())
  };
  let numbers = size >= NUMBER_SCALE && gui.config.get_numbers();
  let mut dragged_image = None;
  let mut images = Vec::new();
  let mut text = Vec::new();
  for i in (0..rows).rev() {
    let (min_y, max_y) = (i as f32, (i + 1) as f32);
    let (min_y, max_y) = if flipped {
      (
        min_y.mul_add(size, board_rect.min.y),
        max_y.mul_add(size, board_rect.min.y),
      )
    } else {
      (
        max_y.mul_add(-size, board_rect.max.y),
        min_y.mul_add(-size, board_rect.max.y),
      )
    };
    if numbers {
      text.push((
        pos2(board_rect.max.x, min_y),
        (i + 1).to_string(),
        cols + i + 1,
        Align2::RIGHT_TOP,
      ));
    }
    for j in 0..cols {
      let coords = (i, j);
      let black_square = (i + j) % 2 == 0;
      let rect = Rect {
        min: pos2((j as f32).mul_add(size, board_rect.min.x), min_y),
        max: pos2(((j + 1) as f32).mul_add(size, board_rect.min.x), max_y),
      };
      let mut colour = if black_square {
        Colours::BlackSquare
      } else {
        Colours::WhiteSquare
      };
      if gamestate.attacked_kings().contains(&&coords) {
        colour = Colours::Check;
      } else if let Some(last_move) = gamestate.last_move {
        if coords == last_move.start() || coords == last_move.end() {
          colour = Colours::Moved;
        }
      }
      let piece = gamestate.get_piece(coords);
      let (selected, piece_rect) = if let Some(dragged) = dragged {
        let mut rect = rect;
        if dragged == coords {
          rect = rect.translate(offset.to_vec2());
          let center = rect.center().clamp(board_rect.min, board_rect.max);
          rect.set_center(center);
        }
        (Some(dragged), rect)
      } else {
        (gui.selected, rect)
      };
      if let Some(start) = selected {
        if start == coords {
          colour = Colours::Selected;
        } else if gamestate.check_pseudolegal(start, coords)
          && gamestate.get_legal(start, coords).is_some()
        {
          colour = if piece == 0 {
            if black_square {
              Colours::ValidBlack
            } else {
              Colours::ValidWhite
            }
          } else if black_square {
            Colours::ThreatenedBlack
          } else {
            Colours::ThreatenedWhite
          }
        }
      }
      if colour != Colours::WhiteSquare {
        painter.rect_filled(rect, Rounding::none(), colour.value());
      }
      if piece != 0 {
        let texture = gui.get_image(painter.ctx(), piece, size as u32);
        let image = Shape::image(texture, piece_rect, UV, Color32::WHITE);
        if dragged == Some(coords) {
          dragged_image = Some(image);
        } else {
          images.push(image);
        }
      };
    }
  }
  painter.extend(images);
  if let Some(image) = dragged_image {
    painter.add(image);
  }
  if numbers {
    for i in 0..cols {
      text.push((
        pos2((i as f32).mul_add(size, board_rect.min.x), board_rect.max.y),
        to_letters(i).iter().collect::<String>(),
        if flipped { rows + i + 1 } else { rows + i },
        Align2::LEFT_BOTTOM,
      ));
    }
  }
  for (pos, text, i, align) in &text {
    painter.text(
      *pos,
      *align,
      text,
      FontId::proportional(size / NUMBER_SCALE),
      if i % 2 == 0 {
        Colours::WhiteSquare
      } else {
        Colours::BlackSquare
      }
      .value(),
    );
  }
}

fn get_size(ctx: &Context, rows: f32, cols: f32) -> (f32, Vec2) {
  let available_size = ctx.available_rect().size();
  let row_size = (available_size.y / rows).floor();
  let column_size = (available_size.x / cols).floor();
  let size = f32::max(1.0, f32::min(row_size, column_size));
  let board_size = Vec2 {
    x: size * cols,
    y: size * rows,
  };
  (size, board_size)
}

fn get_hovered(
  board_rect: Rect,
  location: Pos2,
  size: usize,
  flipped: bool,
  gamestate: &Board,
) -> Option<((usize, usize), i8)> {
  if board_rect.contains(location) {
    let x = (location.x - board_rect.min.x) as usize / size;
    let y = if flipped {
      location.y - board_rect.min.y
    } else {
      board_rect.max.y - location.y
    } as usize
      / size;
    let coords = (y, x);
    gamestate.fetch_piece(coords).map(|piece| (coords, *piece))
  } else {
    None
  }
}

fn register_response(
  gui: &mut LibertyChessGUI,
  gamestate: &Board,
  response: &Response,
  hover: Option<((usize, usize), Piece)>,
) {
  if let Some((coords, piece)) = hover {
    let capture = piece != 0;
    let valid_piece = capture && gamestate.to_move() == (piece > 0);
    if response.clicked() {
      if let Some(selected) = gui.selected {
        attempt_move(
          gui,
          gamestate,
          selected,
          coords,
          #[cfg(feature = "sound")]
          capture,
        );
      } else if valid_piece {
        gui.selected = Some(coords);
      }
    }
    if response.drag_started() && response.dragged_by(PointerButton::Primary) && valid_piece {
      gui.drag = Some((coords, Pos2::default()));
    }
  }
  if let Some((start, ref mut offset)) = gui.drag {
    *offset += response.drag_delta();
    if response.drag_released() {
      #[cfg(feature = "sound")]
      if let Some((coords, piece)) = hover {
        if start != coords {
          let capture = piece != 0;
          attempt_move(gui, gamestate, start, coords, capture);
        }
      }
      #[cfg(not(feature = "sound"))]
      if let Some((coords, _)) = hover {
        if start != coords {
          attempt_move(gui, gamestate, start, coords);
        }
      }
      gui.drag = None;
    }
  }
}

fn attempt_move(
  gui: &mut LibertyChessGUI,
  gamestate: &Board,
  selected: (usize, usize),
  coords: (usize, usize),
  #[cfg(feature = "sound")] capture: bool,
) {
  #[cfg(feature = "sound")]
  let mut effect = Effect::Illegal;
  if gamestate.check_pseudolegal(selected, coords) {
    if let Some(mut newstate) = gamestate.get_legal(selected, coords) {
      if !newstate.promotion_available() {
        newstate.update();
        #[cfg(feature = "clock")]
        if let Some(clock) = &mut gui.clock {
          clock.switch_clocks();
        }
      }
      gui.undo.push(gamestate.clone());
      #[cfg(feature = "sound")]
      {
        effect = match newstate.state() {
          Gamestate::Checkmate(_) | Gamestate::Elimination(_) => Effect::Victory,
          Gamestate::Stalemate
          | Gamestate::Repetition
          | Gamestate::Move50
          | Gamestate::Material => Effect::Draw,
          Gamestate::InProgress => {
            if newstate.attacked_kings().is_empty() {
              if capture {
                Effect::Capture
              } else {
                Effect::Move
              }
            } else {
              Effect::Check
            }
          }
        };
      }
      #[cfg(feature = "music")]
      {
        let dramatic = get_dramatic(&newstate) + if capture { 0.5 } else { 0.0 };
        if let Some(ref mut player) = gui.audio_engine {
          player.set_dramatic(dramatic);
        }
      }
      gui.screen = Screen::Game(Box::new(newstate));
    }
  }
  #[cfg(feature = "sound")]
  if let Some(player) = &mut gui.audio_engine {
    player.play(&effect);
  }
  gui.selected = None;
}
