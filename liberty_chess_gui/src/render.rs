use crate::helpers::unwrap_tuple;
use crate::players::{ConnectionMessage, PlayerData, UciState};
use crate::themes::Colours;
use crate::{LibertyChessGUI, Screen};
use eframe::egui::{
  pos2, Align2, Area, Color32, Context, FontId, PointerButton, Pos2, Rect, Response, Rounding,
  Sense, Shape, Ui, Vec2,
};
use liberty_chess::moves::Move;
use liberty_chess::parsing::to_letters;
use liberty_chess::{Board, Gamestate, Piece};
use std::sync::mpsc::TryRecvError;
use ulci::client::Message;

#[cfg(feature = "clock")]
use liberty_chess::clock::Clock;
#[cfg(feature = "clock")]
use std::time::Duration;
#[cfg(feature = "clock")]
use ulci::SearchTime;

#[cfg(feature = "sound")]
use crate::helpers::update_sound;
#[cfg(feature = "sound")]
use sound::Effect;

#[cfg(feature = "music")]
use crate::get_dramatic;

//UV that does nothing
const UV: Rect = Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0));
const NUMBER_SCALE: f32 = 5.0;

pub(crate) fn draw_game(gui: &mut LibertyChessGUI, ctx: &Context, mut board: Board) {
  let mut clickable;
  clickable = !board.promotion_available() && board.state() == Gamestate::InProgress;
  #[cfg(feature = "clock")]
  if let Some(clock) = &gui.clock {
    if clock.is_flagged() {
      gui.selected = None;
      clickable = false;
    }
  }
  if let Some((player, side)) = &mut gui.player {
    if *side == board.to_move() {
      clickable = false;
      #[cfg(feature = "clock")]
      if let Some(ref mut clock) = gui.clock {
        let (wtime, btime) = clock.get_clocks();
        let new_time = if board.to_move() { wtime } else { btime };
        if let SearchTime::Increment(ref mut time, _) = gui.searchtime {
          *time = new_time.as_millis();
        }
      }
      let (bestmove, score) = player.poll(&board, gui.searchtime);
      if let Some(score) = score {
        gui.eval = Some(score);
      }
      if let Some(bestmove) = bestmove {
        if let Some(position) = board.move_if_legal(bestmove) {
          #[cfg(feature = "sound")]
          let capture = board.get_piece(bestmove.end()) != 0;
          #[cfg(feature = "sound")]
          if let Some(engine) = &mut gui.audio_engine {
            let effect = update_sound(&position, capture);
            engine.play(&effect);
            #[cfg(feature = "music")]
            {
              let dramatic = get_dramatic(&position) + if capture { 0.5 } else { 0.0 };
              engine.set_dramatic(dramatic);
            }
          }
          #[cfg(feature = "clock")]
          if let Some(clock) = &mut gui.clock {
            clock.update_status(&position);
          }
          gui.screen = Screen::Game(Box::new(position));
          // It needs 1 more frame to update for some reason
          ctx.request_repaint();
        } else {
          gui.message = Some(format!("Engine made illegal move {}", bestmove.to_string()));
        }
      }
    }
    match player {
      PlayerData::Multiplayer(interface) => {
        let mut clear_player = false;
        loop {
          match interface.connection.try_recv() {
            Ok(message) => match message {
              ConnectionMessage::Uci(message) => match message {
                Message::UpdatePosition(new_board) => {
                  let new_board = new_board.load_from_thread();
                  *side = board.to_move();
                  #[cfg(feature = "sound")]
                  if let Some(ref mut engine) = gui.audio_engine {
                    let effect = update_sound(&new_board, false);
                    engine.play(&effect);
                  }
                  board = new_board.clone();
                  gui.screen = Screen::Game(Box::new(new_board));
                  gui.selected = None;
                  gui.drag = None;
                }
                #[cfg(feature = "clock")]
                Message::Go(settings) => {
                  *side = !board.to_move();
                  if gui.config.get_opponentflip() {
                    gui.flipped = *side;
                  }
                  match settings.time {
                    SearchTime::Increment(time, inc) => {
                      let mut clock = Clock::new_symmetric(
                        Duration::from_millis(time as u64),
                        Duration::from_millis(inc as u64),
                        board.to_move(),
                      );
                      clock.toggle_pause();
                      gui.clock = Some(clock);
                    }
                    SearchTime::Asymmetric(wtime, winc, btime, binc) => {
                      let mut clock = Clock::new(
                        [
                          Duration::from_millis(wtime as u64),
                          Duration::from_millis(btime as u64),
                          Duration::from_millis(winc as u64),
                          Duration::from_millis(binc as u64),
                        ],
                        board.to_move(),
                      );
                      clock.toggle_pause();
                      gui.clock = Some(clock);
                    }
                    _ => (),
                  }
                }
                #[cfg(not(feature = "clock"))]
                Message::Go(_) => {
                  *side = board.to_move();
                  if gui.config.get_opponentflip() {
                    gui.flipped = *side;
                  }
                }
                Message::UpdateOption(..)
                | Message::SetDebug(_)
                | Message::Stop
                | Message::Eval
                | Message::Bench(_)
                | Message::NewGame
                | Message::Perft(_) => (),
              },
              ConnectionMessage::Connected(_) | ConnectionMessage::Timeout => (),
            },
            Err(TryRecvError::Disconnected) => {
              clear_player = true;
              gui.message = Some("Disconnected".to_owned());
              break;
            }
            Err(TryRecvError::Empty) => break,
          }
        }
        if clear_player {
          gui.player = None;
        }
      }
      PlayerData::Uci(uci) => {
        if uci.state == UciState::Crashed {
          gui.message = Some("Engine has crashed".to_owned());
        }
      }
      _ => (),
    }
  }
  Area::new("Board")
    .anchor(Align2::CENTER_CENTER, Vec2::ZERO)
    .show(ctx, |ui| {
      draw_board(gui, ctx, ui, board, clickable, gui.flipped);
    });
}

pub(crate) fn draw_board(
  gui: &mut LibertyChessGUI,
  ctx: &Context,
  ui: &mut Ui,
  mut gamestate: Board,
  clickable: bool,
  flipped: bool,
) {
  if gui.safety_mode {
    gamestate.friendly_fire = false;
  }
  let rows = gamestate.height();
  let cols = gamestate.width();
  let (size, board_size) = get_size(ctx, rows as f32, cols as f32);
  let sense = if clickable {
    Sense::click_and_drag()
  } else {
    gui.drag = None;
    Sense::hover()
  };
  let (response, painter) = ui.allocate_painter(board_size, sense);
  let board_rect = response.rect;
  painter.rect_filled(board_rect, Rounding::ZERO, Colours::WhiteSquare.value());
  if let Some(location) = response.interact_pointer_pos() {
    let hover = get_hovered(board_rect, location, size as usize, flipped, &gamestate);
    register_response(gui, &gamestate, &response, hover);
  }
  let (dragged, offset) = unwrap_tuple(gui.drag);
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
        if flipped { i } else { cols + i + 1 } % 2 == 0,
        Align2::RIGHT_TOP,
      ));
    }
    for j in 0..cols {
      let coords = (i, j);
      let black_square = (i + j) % 2 == 0;
      let min_x = if flipped {
        ((j + 1) as f32).mul_add(-size, board_rect.max.x)
      } else {
        (j as f32).mul_add(size, board_rect.min.x)
      };
      let max_x = if flipped {
        (j as f32).mul_add(-size, board_rect.max.x)
      } else {
        ((j + 1) as f32).mul_add(size, board_rect.min.x)
      };
      let rect = Rect {
        min: pos2(min_x, min_y),
        max: pos2(max_x, max_y),
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
        painter.rect_filled(rect, Rounding::ZERO, colour.value());
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
      let x = if flipped {
        ((i + 1) as f32).mul_add(-size, board_rect.max.x)
      } else {
        (i as f32).mul_add(size, board_rect.min.x)
      };
      text.push((
        pos2(x, board_rect.max.y),
        to_letters(i).iter().collect::<String>(),
        if flipped {
          (rows + i + 1) % 2 == 0
        } else {
          i % 2 == 0
        },
        Align2::LEFT_BOTTOM,
      ));
    }
  }
  for (pos, text, colour, align) in &text {
    painter.text(
      *pos,
      *align,
      text,
      FontId::proportional(size / NUMBER_SCALE),
      if *colour {
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
    let x = if flipped {
      board_rect.max.x - location.x
    } else {
      location.x - board_rect.min.x
    } as usize
      / size;
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
          clock.update_status(&newstate);
        }
      }
      if gui.player.is_none() && gui.config.get_autoflip() {
        gui.flipped = gamestate.to_move();
      }
      #[cfg(feature = "sound")]
      {
        effect = update_sound(&newstate, capture);
      }
      #[cfg(feature = "music")]
      {
        let dramatic = get_dramatic(&newstate) + if capture { 0.5 } else { 0.0 };
        if let Some(ref mut player) = gui.audio_engine {
          player.set_dramatic(dramatic);
        }
      }
      let mut play_move = true;
      if !newstate.promotion_available() {
        if let Some((PlayerData::Multiplayer(ref mut interface), _)) = gui.player {
          play_move = false;
          interface.play_move(Move::new(selected, coords));
        }
      }
      if play_move {
        gui.undo.push(gamestate.clone());
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
