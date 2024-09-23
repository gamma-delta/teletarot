use boxes::BoxSide;
use cursive::{
  event::{Event, EventResult, Key},
  theme::{BaseColor, Color, ColorType, PaletteColor, Style, Theme},
  views::{BoxedView, FixedLayout},
  Cursive, CursiveExt, Printer, Rect, Vec2, View, With,
};
use cursive_hjkl::HjklToDirectionWrapperView;
use teletarot_model::{Board, BoardZone, Card, Suit};
use xp::{CardAtlas, CardBorderColor, CARD_HEIGHT, CARD_SIZE, CARD_WIDTH};

mod boxes;
mod colors;
mod xp;

fn main() {
  let mut siv = cursive::default();

  let theme = Theme::terminal_default().with(|t| {
    // t.palette[PaletteColor::Background] = Color::Light(BaseColor::White);
  });
  siv.set_theme(theme);

  let teletarot = TeletarotView::new(None);
  siv.add_layer(HjklToDirectionWrapperView::new(teletarot));
  siv.add_global_callback('q', Cursive::quit);
  siv.run();
}

struct TeletarotView {
  atlas: CardAtlas,
  board: Board,

  /// If 0..=10, selects the columns.
  /// If 11, selects the storage over the minor foundation
  cursor: usize,
  cursor_src: Option<usize>,
}

impl TeletarotView {
  fn new(seed: Option<u64>) -> Self {
    Self {
      atlas: CardAtlas::new(),
      board: Board::new(seed),
      cursor: 0,
      cursor_src: None,
    }
  }

  fn draw_foundations(&self, printer: &Printer) {
    // There will be 22 cards here.
    // Each of them will be tightly packed, but the last card
    // should overlay the other two cards.
    /*
      /---\
    |||   |||
    |||   |||
    |||   |||
    */

    let final_major = match (
      self.board.major_foundation_left_max(),
      self.board.major_foundation_right_min(),
    ) {
      (Some(left), Some(right)) if left + 1 == right => Some(left),
      _ => None,
    };

    // Print right first so that the leftmost card appears on top
    for (idx, card) in self
      .board
      .virtual_cards_major_foundation_right()
      .iter()
      .enumerate()
    {
      self.atlas.print_card(
        card,
        self.atlas.maj_fndn_right() - (CARD_WIDTH + idx + 1, 0),
        printer,
        false,
        CardBorderColor::HilightIfThickBorder,
      );
    }
    for (idx, card) in self
      .board
      .virtual_cards_major_foundation_left()
      .iter()
      .enumerate()
    {
      let is_final = Some(idx as u8) == final_major;
      self.atlas.print_card(
        card,
        self.atlas.maj_fndn_left()
          + (idx + if is_final { CARD_WIDTH / 2 } else { 0 }, 0)
          - (0, if is_final { 1 } else { 0 }),
        printer,
        false,
        CardBorderColor::HilightIfThickBorder,
      );
    }

    // Minor foundation
    let minor_blocked = self.board.minor_foundation_storage().is_some();
    for (suit_idx, stack) in self
      .board
      .virtual_cards_minor_foundation()
      .iter()
      .enumerate()
    {
      let pos = self.atlas.min_fndn_poses()[suit_idx];
      // slot, just in case
      BoxSide::draw_box(
        printer,
        pos,
        (CARD_WIDTH + 2, CARD_HEIGHT + 2),
        fg_color(Color::Light(BaseColor::Black)),
        false,
      );

      let Some(card) = stack.last() else { continue };
      let cbc = if minor_blocked {
        CardBorderColor::Custom(fg_color(Color::Dark(BaseColor::White)))
      } else {
        CardBorderColor::HilightIfThickBorder
      };
      self.atlas.print_card(card, pos, printer, false, cbc);
    }

    // Minor storage
    let normal_column = self.cursor == 11;
    let src_column = self.cursor_src == Some(11);
    let selected = normal_column || src_column;

    if selected {
      BoxSide::draw_box(
        printer,
        self.atlas.min_fndn_storage(),
        CARD_SIZE + Vec2::new(2, 2),
        fg_color(Color::Dark(BaseColor::White)),
        selected,
      );
      printer.with_color(Color::Light(BaseColor::Black).into(), |prn| {
        prn.print_rect(
          Rect::from_size(
            Vec2::from(self.atlas.min_fndn_storage()) + Vec2::new(1, 1),
            CARD_SIZE,
          ),
          " ",
        );
      });
    }

    if let Some(storage) = self.board.minor_foundation_storage() {
      let cbc = if src_column {
        CardBorderColor::Custom(fg_color(Color::Light(BaseColor::White)))
      } else {
        CardBorderColor::HilightIfThickBorder
      };

      self.atlas.print_card(
        storage,
        self.atlas.min_fndn_storage(),
        printer,
        selected,
        cbc,
      );
    }
  }

  fn draw_tableau(&self, printer: &Printer) {
    for (col_idx, col) in self.board.columns().iter().enumerate() {
      let base_pos = self.atlas.column_poses()[col_idx];
      // base slot
      BoxSide::draw_box(
        printer,
        base_pos,
        (CARD_WIDTH + 2, CARD_HEIGHT + 2),
        fg_color(Color::Light(BaseColor::Black)),
        col_idx == self.cursor,
      );

      for (card_idx, card) in col.iter().enumerate() {
        let pos = base_pos + (0, card_idx * 2);

        let normal_column = col_idx == self.cursor;
        let src_column = Some(col_idx) == self.cursor_src;
        let selected =
          card_idx == col.len() - 1 && (normal_column || src_column);

        let is_next_card = match card.suit() {
          Suit::Minor(suit) => {
            let foundation_max =
              self.board.minor_foundation_maxes()[suit as usize];
            card.number() == foundation_max.unwrap_or_default() + 1
          }
          Suit::MajorArcana => {
            let next_hi_min = match self.board.major_foundation_left_max() {
              Some(it) => card.number() == it + 1,
              None => card.number() == 0,
            };
            let next_lo_max = match self.board.major_foundation_right_min() {
              Some(it) => card.number() + 1 == it,
              None => card.number() == Card::MAJOR_ARCANA_MAX,
            };
            next_hi_min || next_lo_max
          }
        };

        let cbc = if selected && src_column {
          CardBorderColor::Custom(fg_color(Color::Light(BaseColor::White)))
        } else if is_next_card {
          CardBorderColor::AlwaysHilight
        } else {
          CardBorderColor::HilightIfThickBorder
        };

        self.atlas.print_card(card, pos, printer, selected, cbc);
      }
    }
  }
}

impl View for TeletarotView {
  fn required_size(&mut self, _constraint: cursive::Vec2) -> cursive::Vec2 {
    self.atlas.board_size()
  }

  fn draw(&self, printer: &cursive::Printer) {
    self.atlas.print_background(printer);
    self.draw_foundations(printer);
    self.draw_tableau(printer);
  }

  fn on_event(&mut self, ev: Event) -> EventResult {
    match ev {
      Event::Key(Key::Left) => {
        self.cursor = (self.cursor + 12 - 1) % 12;
        EventResult::consumed()
      }
      Event::Key(Key::Right) => {
        self.cursor = (self.cursor + 1) % 12;
        EventResult::consumed()
      }
      Event::Char(' ') | Event::Key(Key::Enter) => {
        if let Some(src) = self.cursor_src {
          let src_zone = idx_to_board_zone(src);
          let dst_zone = idx_to_board_zone(self.cursor);
          let _ = self.board.move_card(src_zone, dst_zone, true);

          self.cursor_src = None;
        } else {
          self.cursor_src = Some(self.cursor);
        }
        EventResult::consumed()
      }
      Event::Key(Key::Esc) => {
        self.cursor_src = None;
        EventResult::consumed()
      }
      Event::Char('z') => {
        self.board.check_automove_cards();
        EventResult::consumed()
      }
      _ => EventResult::Ignored,
    }
  }
}

fn fg_color(color: Color) -> Style {
  let mut sty = Style::terminal_default();
  sty.color.front = ColorType::Color(color);
  sty
}

fn idx_to_board_zone(idx: usize) -> BoardZone {
  if idx < Board::COLUMN_COUNT {
    BoardZone::Column(idx)
  } else {
    BoardZone::MinorFoundationStorage
  }
}
