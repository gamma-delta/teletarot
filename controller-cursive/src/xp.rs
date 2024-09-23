use std::io;

use codepage_437::CP437_WINGDINGS;
use cursive::{
  theme::{BaseColor, Color, ColorPair, Style},
  utils::markup::{StyledStr, StyledString},
  vec::Vec2i,
  Printer, Vec2, With,
};
use itertools::iproduct;
use rexpaint::{XpCell, XpColor, XpFile};
use teletarot_model::{Card, Suit};

use crate::{boxes::BoxSide, colors, fg_color};

const CARD_XP_INCLUDE: &'static [u8] = include_bytes!("include/cards.xp");

pub const CARD_WIDTH: usize = 11;
pub const CARD_HEIGHT: usize = 10;
pub const CARD_PADDING: usize = 1;
pub const CARD_SIZE: Vec2 = Vec2::new(CARD_WIDTH, CARD_HEIGHT);

pub struct CardAtlas {
  xp: XpFile,
}

impl CardAtlas {
  pub fn new() -> Self {
    let xp = XpFile::read(&mut io::Cursor::new(CARD_XP_INCLUDE)).unwrap();
    Self { xp }
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
      self.draw_xp_cell(
        src_pos.x,
        src_pos.y,
        card_corner + (dx + 1, dy + 1),
        printer,
      );
    }
  }

  fn get_xp_thru_layers(&self, x: usize, y: usize) -> XpCell {
    self
      .xp
      .layers
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

  /// Draw an XP cell from the given position from the xp file.
  fn draw_xp_cell(&self, x: usize, y: usize, dest: Vec2, printer: &Printer) {
    let cell = self.get_xp_thru_layers(x, y);
    let siv_col = color_xp_to_siv(cell.fg);
    let ch = CP437_WINGDINGS.decode(cell.ch as u8);
    printer.print_styled(
      dest,
      &StyledString::styled(
        ch.to_string(),
        ColorPair::terminal_default().with(|x| x.front = siv_col),
      ),
    );
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
