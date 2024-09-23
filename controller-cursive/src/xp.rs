use std::io;

use codepage_437::CP437_WINGDINGS;
use cursive::{
  theme::{BaseColor, Color, ColorPair, Style},
  utils::markup::{StyledStr, StyledString},
  vec::Vec2i,
  Printer, Vec2, With,
};
use getset::{CopyGetters, Getters};
use itertools::iproduct;
use rexpaint::{XpCell, XpColor, XpFile, XpLayer};
use teletarot_model::{Board, Card, Suit};

use crate::{boxes::BoxSide, colors, fg_color};

const CARD_XP_INCLUDE: &'static [u8] = include_bytes!("include/cards.xp");
const LAYOUT_XP_INCLUDE: &'static [u8] = include_bytes!("include/board.xp");

pub const CARD_WIDTH: usize = 11;
pub const CARD_HEIGHT: usize = 10;
pub const CARD_PADDING: usize = 1;
pub const CARD_SIZE: Vec2 = Vec2::new(CARD_WIDTH, CARD_HEIGHT);

#[derive(Getters, CopyGetters)]
pub struct CardAtlas {
  cards_xp: XpFile,
  layout_display: XpLayer,

  #[getset(get_copy = "pub")]
  maj_fndn_left: Vec2,
  #[getset(get_copy = "pub")]
  maj_fndn_right: Vec2,
  #[getset(get_copy = "pub")]
  min_fndn_storage: Vec2,
  #[getset(get = "pub")]
  min_fndn_poses: [Vec2; 4],
  #[getset(get = "pub")]
  column_poses: [Vec2; Board::COLUMN_COUNT],
  #[getset(get_copy = "pub")]
  board_size: Vec2,
}

impl CardAtlas {
  pub fn new() -> Self {
    let cards_xp = XpFile::read(&mut io::Cursor::new(CARD_XP_INCLUDE)).unwrap();

    let layout_xp =
      XpFile::read(&mut io::Cursor::new(LAYOUT_XP_INCLUDE)).unwrap();

    let mut maj_fndn_left = None;
    let mut maj_fndn_right = None;
    let mut min_fndn_storage = None;
    let mut min_fndn_poses = [None; 4];
    let mut column_poses = [None; Board::COLUMN_COUNT];
    let mut board_size = None;

    let layout_layer = &layout_xp.layers[1];
    for (idx, cell) in layout_layer.cells.iter().enumerate() {
      // pos is backwards from idx. huuahrhg
      let pos = Vec2::new(idx / layout_layer.height, idx % layout_layer.height);

      match cell.ch as u8 {
        b'L' => {
          maj_fndn_left = Some(pos);
        }
        b'R' => {
          maj_fndn_right = Some(pos);
        }
        0x0F => {
          min_fndn_poses[0] = Some(pos);
        }
        0x9D => {
          min_fndn_poses[1] = Some(pos);
        }
        0x18 => {
          min_fndn_poses[2] = Some(pos);
        }
        b'%' => {
          min_fndn_poses[3] = Some(pos);
        }
        b'S' => {
          min_fndn_storage = Some(pos);
        }
        it @ b'a'..=b'k' => {
          column_poses[(it - b'a') as usize] = Some(pos);
        }
        b'X' => {
          board_size = Some(pos + (1, 1));
        }
        _ => {}
      }
    }

    Self {
      cards_xp,
      layout_display: layout_xp.layers[0].clone(),
      maj_fndn_left: maj_fndn_left.unwrap(),
      maj_fndn_right: maj_fndn_right.unwrap(),
      min_fndn_storage: min_fndn_storage.unwrap(),
      min_fndn_poses: min_fndn_poses.map(Option::unwrap),
      column_poses: column_poses.map(Option::unwrap),
      board_size: board_size.unwrap(),
    }
  }

  /// Draw a card, including the border, at the given position.
  pub fn print_card(
    &self,
    card: &Card,
    card_corner: impl Into<Vec2>,
    printer: &Printer,
    thick_border: bool,
    border_color: CardBorderColor,
  ) {
    let card_corner = card_corner.into();

    let suit_color = colors::suit_color(card.suit());
    let border_style = match border_color {
      CardBorderColor::HilightIfThickBorder => fg_color(if thick_border {
        Color::Light
      } else {
        Color::Dark
      }(suit_color)),
      CardBorderColor::AlwaysHilight => fg_color(Color::Light(suit_color)),
      CardBorderColor::NeverHilight => fg_color(Color::Dark(suit_color)),
      CardBorderColor::Custom(it) => it,
    };

    BoxSide::draw_box(
      printer,
      card_corner,
      CARD_SIZE + Vec2::new(2, 2),
      border_style,
      thick_border,
    );

    let atlas_idxes = match card.suit() {
      Suit::Minor(suit) => Vec2::new(card.number() as usize - 1, suit as _),
      Suit::MajorArcana => {
        Vec2::new(0, 4) + (card.number() % 13, card.number() / 13)
      }
    };
    for (dy, dx) in iproduct!(0..CARD_HEIGHT, 0..CARD_WIDTH) {
      let src_pos = (atlas_idxes * (CARD_SIZE + (1, 1))) + (1, 1) + (dx, dy);
      let cell = get_xp_thru_layers(&self.cards_xp, src_pos.x, src_pos.y);
      draw_xp_cell(cell, card_corner + (dx + 1, dy + 1), printer);
    }
  }

  pub fn print_background(&self, printer: &Printer) {
    for (idx, cell) in self.layout_display.cells.iter().enumerate() {
      let x = idx / self.layout_display.height;
      let y = idx % self.layout_display.height;
      draw_xp_cell(cell.clone(), (x, y), printer);
    }
  }
}

fn color_xp_to_siv(color: XpColor) -> Color {
  use cursive::style::{BaseColor::*, Color::*};

  match (color.r, color.g, color.b) {
    (0, 0, 0) => Dark(Black),
    (170, 0, 0) => Dark(Red),
    (0, 170, 0) => Dark(Green),
    (170, 85, 0) => Dark(Yellow),
    (0, 0, 170) => Dark(Blue),
    (170, 0, 170) => Dark(Magenta),
    (0, 170, 170) => Dark(Cyan),
    (170, 170, 170) => Dark(White),
    //
    (85, 85, 85) => Light(Black),
    (255, 85, 85) => Light(Red),
    (85, 255, 85) => Light(Green),
    (255, 255, 85) => Light(Yellow),
    (85, 85, 255) => Light(Blue),
    (255, 85, 255) => Light(Magenta),
    (85, 255, 255) => Light(Cyan),
    (255, 255, 255) => Light(White),
    ono => unreachable!("forgot to account for xp color {:?}", ono),
  }
}

pub enum CardBorderColor {
  HilightIfThickBorder,
  AlwaysHilight,
  NeverHilight,
  Custom(Style),
}

/// Draw an XP cell from the given position from the xp file.
pub fn draw_xp_cell(cell: XpCell, dest: impl Into<Vec2>, printer: &Printer) {
  let siv_col = color_xp_to_siv(cell.fg);
  let ch = CP437_WINGDINGS.decode(cell.ch as u8);
  printer.print_styled(
    dest.into(),
    &StyledString::styled(
      ch.to_string(),
      ColorPair::terminal_default().with(|x| x.front = siv_col),
    ),
  );
}

fn get_xp_thru_layers(xp: &XpFile, x: usize, y: usize) -> XpCell {
  xp.layers
    .iter()
    .rev()
    .find_map(|layer| {
      let cell = layer.get(x, y)?;
      if cell.bg.is_transparent() || cell.ch == ' ' as u32 {
        None
      } else {
        Some(*cell)
      }
    })
    .unwrap_or(XpCell {
      ch: '█' as u32,
      fg: XpColor::BLACK,
      bg: XpColor::BLACK,
    })
}
