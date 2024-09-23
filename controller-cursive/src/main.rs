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
  const VIEW_SIZE: Vec2 = Vec2::new(
    Board::COLUMN_COUNT * (CARD_WIDTH + 2) + 2,
    CARD_HEIGHT + 3 + Self::TABLEAU_HEIGHT,
  );
  const TABLEAU_HEIGHT: usize = CARD_HEIGHT * 3;

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
    let majfound_width = 22 + CARD_WIDTH + 2;

    for (idx, card) in self
      .board
      .virtual_cards_major_foundation_left()
      .iter()
      .enumerate()
    {
      self.atlas.print_card(
        card,
        (1 + idx, 1),
        printer,
        false,
        CardBorderColor::HilightIfThickBorder,
      );
    }
    for (idx, card) in self
      .board
      .virtual_cards_major_foundation_right()
      .iter()
      .enumerate()
    {
      self.atlas.print_card(
        card,
        (majfound_width - idx, 1),
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
      let xpos = Self::VIEW_SIZE.x - (4 - suit_idx) * (CARD_WIDTH + 2) - 1;
      // slot, just in case
      BoxSide::draw_box(
        printer,
        (xpos, 1),
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
      self.atlas.print_card(card, (xpos, 1), printer, false, cbc);
    }

    // Minor storage
    let mfs_pos = (
      Self::VIEW_SIZE.x - 2 * (CARD_WIDTH + 2) - CARD_WIDTH / 2 - 2,
      1,
    );
    let normal_column = self.cursor == 11;
    let src_column = self.cursor_src == Some(11);
    let selected = normal_column || src_column;

    if selected {
      BoxSide::draw_box(
        printer,
        mfs_pos,
        CARD_SIZE + Vec2::new(2, 2),
        fg_color(Color::Dark(BaseColor::White)),
        selected,
      );
      printer.with_color(Color::Light(BaseColor::Black).into(), |prn| {
        prn.print_rect(
          Rect::from_size(Vec2::from(mfs_pos) + Vec2::new(1, 1), CARD_SIZE),
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

      self
        .atlas
        .print_card(storage, mfs_pos, printer, selected, cbc);
    }
  }

  fn draw_tableau(&self, printer: &Printer) {
    let tableau_y = CARD_HEIGHT + 3;
    BoxSide::draw_box(
      printer,
      (0, tableau_y),
      (Self::VIEW_SIZE.x, Self::TABLEAU_HEIGHT),
      fg_color(Color::Dark(BaseColor::White)),
      false,
    );

    let tableau_card_y = tableau_y + 1;

    for (col_idx, col) in self.board.columns().iter().enumerate() {
      let col_x = 1 + (col_idx * (CARD_WIDTH + 2));
      // base slot
      BoxSide::draw_box(
        printer,
        (col_x, tableau_card_y),
        (CARD_WIDTH + 2, CARD_HEIGHT + 2),
        fg_color(Color::Light(BaseColor::Black)),
        col_idx == self.cursor,
      );

      for (card_idx, card) in col.iter().enumerate() {
        let pos = (col_x, tableau_card_y + card_idx * 2);

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
    Self::VIEW_SIZE
  }

  fn draw(&self, printer: &cursive::Printer) {
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
