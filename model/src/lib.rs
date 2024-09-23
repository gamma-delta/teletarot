mod cards;
mod random;

use std::array;

pub use cards::*;

use fastrand::Rng;
use getset::{CopyGetters, Getters, MutGetters};
use itertools::{iproduct, Itertools};

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters, MutGetters)]
pub struct Board {
  #[getset(get = "pub")]
  columns: [Column; Board::COLUMN_COUNT],
  /// If there's a card sideways over the minor foundation, it goes here.
  minor_foundation_storage: Option<Card>,

  /// The maximum card in each of the minor foundation slots, in order of suit.
  #[getset(get = "pub")]
  minor_foundation_maxes: [Option<u8>; 4],
  /// The highest number on the left side of the major foundation.
  ///
  /// The final "center" card is placed only in left_max.
  #[getset(get_copy = "pub")]
  major_foundation_left_max: Option<u8>,
  /// The lowest number on the right side of the minor foundation.
  #[getset(get_copy = "pub")]
  major_foundation_right_min: Option<u8>,
}

impl Board {
  pub const COLUMN_COUNT: usize = 11;
  pub const DECK_SIZE: usize =
    (Card::MINOR_ARCANA_MAX - Card::MINOR_ARCANA_MIN + 1) as usize * 4
      + (Card::MAJOR_ARCANA_MAX - Card::MAJOR_ARCANA_MIN + 1) as usize;

  pub fn new(seed: Option<u64>) -> Self {
    random::random_board(seed)
  }

  pub fn new_solved(final_major_arcana: u8) -> Self {
    let (mfl, mfr) = match final_major_arcana {
      Card::MAJOR_ARCANA_MIN => (None, Some(Card::MAJOR_ARCANA_MIN)),
      Card::MAJOR_ARCANA_MAX => (Some(Card::MAJOR_ARCANA_MAX), None),
      _ => (Some(final_major_arcana), Some(final_major_arcana + 1)),
    };

    Self {
      columns: array::from_fn(|_| Column::empty()),
      minor_foundation_storage: None,
      minor_foundation_maxes: [Some(Card::MINOR_ARCANA_MAX); 4],
      major_foundation_left_max: mfl,
      major_foundation_right_min: mfr,
    }
  }

  pub fn empty() -> Board {
    Board {
      columns: array::from_fn(|_| Column::empty()),
      minor_foundation_storage: None,
      minor_foundation_maxes: [None; 4],
      major_foundation_left_max: None,
      major_foundation_right_min: None,
    }
  }

  pub fn move_card(
    &mut self,
    source_zone: BoardZone,
    dest_zone: BoardZone,
    cascade_column_stacks: bool,
  ) -> Result<(), CardMoveError> {
    if source_zone == dest_zone {
      return Err(CardMoveError::NoopMovement);
    }
    if source_zone.is_write_only() {
      return Err(CardMoveError::WriteOnlySource);
    }

    if let (
      true,
      BoardZone::Column(src_col_idx),
      BoardZone::Column(dst_col_idx),
    ) = (cascade_column_stacks, &source_zone, &dest_zone)
    {
      let src_col = self.get_column(*src_col_idx);

      let src_head = src_col.last().ok_or(CardMoveError::EmptySource)?;
      let dst_head = self.get_column(*dst_col_idx).last();
      let stack_ok = match dst_head {
        Some(it) => src_head.can_stack(it),
        None => true,
      };
      if !stack_ok {
        return Err(CardMoveError::CannotStack);
      }

      let source_take_count = 1
        + src_col
          .iter()
          .tuple_windows()
          .take_while(|(prev, here)| prev.can_stack(here))
          .count();

      let src_col_mut = self.get_column_mut(*src_col_idx);
      let sc_len = src_col_mut.len();
      let mut transfer = src_col_mut.split_off(sc_len - source_take_count);
      transfer.reverse();
      self.get_column_mut(*dst_col_idx).extend(transfer);
    }

    let source_card = match &source_zone {
      BoardZone::Column(i) => self.get_column(*i).last(),
      BoardZone::MinorFoundationStorage => {
        self.minor_foundation_storage.as_ref()
      }
      BoardZone::MinorFoundation | BoardZone::MajorFoundation => {
        unreachable!(
          "{:?} should have been caught by is_write_only",
          &source_zone
        )
      }
    };
    let source_card = source_card.ok_or(CardMoveError::EmptySource)?.clone();

    match &dest_zone {
      BoardZone::Column(idx) => {
        let dst_col = self.get_column_mut(*idx);
        let stack_ok = match dst_col.last() {
          None => true,
          Some(here) => source_card.can_stack(here),
        };
        if !stack_ok {
          return Err(CardMoveError::CannotStack);
        }

        // will remove from the source in just a second!
        dst_col.push(source_card);
      }
      BoardZone::MinorFoundationStorage => {
        if self.minor_foundation_storage.is_some() {
          return Err(CardMoveError::BlockedByFullMinorStorage);
        }
        self.minor_foundation_storage = Some(source_card);
      }
      BoardZone::MinorFoundation => {
        if self.minor_foundation_storage.is_some() {
          return Err(CardMoveError::BlockedByFullMinorStorage);
        }
        let suit = match source_card.suit() {
          Suit::Minor(it) => it,
          Suit::MajorArcana => return Err(CardMoveError::WrongTargetZone),
        };

        let stack_height = &mut self.minor_foundation_maxes[suit as usize];
        let stack_ok = match stack_height {
          None => true,
          Some(height) => source_card.number() == *height + 1,
        };
        if stack_ok {
          *stack_height = Some(stack_height.unwrap_or_default() + 1);
        } else {
          return Err(CardMoveError::CannotStack);
        }
      }
      BoardZone::MajorFoundation => {
        if source_card.suit().is_minor() {
          return Err(CardMoveError::WrongTargetZone);
        }

        let left_ok = match self.major_foundation_left_max {
          None => source_card.number() == Card::MAJOR_ARCANA_MIN,
          Some(l) => source_card.number() == l + 1,
        };
        let right_ok = match self.major_foundation_right_min {
          None => source_card.number() == Card::MAJOR_ARCANA_MAX,
          Some(r) => source_card.number() + 1 == r,
        };
        if !left_ok && !right_ok {
          return Err(CardMoveError::CannotStack);
        }
        if left_ok {
          self.major_foundation_left_max = Some(source_card.number());
        } else if right_ok {
          self.major_foundation_right_min = Some(source_card.number());
        }
      }
    }

    // If control flow reaches here, there's been no error,
    // so we need to remove the source card from its zone
    match &source_zone {
      BoardZone::Column(i) => {
        self.get_column_mut(*i).pop();
      }
      BoardZone::MinorFoundationStorage => {
        self.minor_foundation_storage = None;
      }
      BoardZone::MinorFoundation | BoardZone::MajorFoundation => {
        unreachable!(
          "{:?} should have been caught by is_write_only",
          &source_zone
        )
      }
    };

    Ok(())
  }
  pub fn check_automove_cards(&mut self) {
    'columns: loop {
      let moved_any = (0..Board::COLUMN_COUNT).any(|col_idx| {
        let src_zone = BoardZone::Column(col_idx);
        'fix_top: loop {
          let moved_any =
            [BoardZone::MinorFoundation, BoardZone::MajorFoundation]
              .iter()
              .any(|dst| {
                let res = self.move_card(src_zone.clone(), dst.clone(), true);
                res.is_ok()
              });
          if !moved_any {
            break 'fix_top;
          }
        }
        false
      });
      if !moved_any {
        break 'columns;
      };
    }
  }

  pub fn get_column(&self, idx: usize) -> &Column {
    &self.columns[idx]
  }

  pub fn get_column_mut(&mut self, idx: usize) -> &mut Column {
    &mut self.columns[idx]
  }

  /// Get the cards in the left side of the major foundation.
  /// This is "virtual" because they're not actually stored as the cards.
  pub fn virtual_cards_major_foundation_left(&self) -> Vec<Card> {
    match self.major_foundation_left_max {
      None => Vec::new(),
      Some(high) => (Card::MAJOR_ARCANA_MIN..=high)
        .map(|idx| Card::new(Suit::MajorArcana, idx))
        .collect(),
    }
  }

  /// Get the cards in the right side of the major foundation.
  /// This is "virtual" because they're not actually stored as the cards.
  pub fn virtual_cards_major_foundation_right(&self) -> Vec<Card> {
    match self.major_foundation_right_min {
      None => Vec::new(),
      Some(low) => (low..=Card::MAJOR_ARCANA_MAX)
        .rev()
        .map(|idx| Card::new(Suit::MajorArcana, idx))
        .collect(),
    }
  }

  /// Get the cards in each slot of the minor foundation.
  /// This is "virtual" because they're not actually stored as the cards.
  ///
  /// In Zach's implementation, only the top card of these is displayed.
  pub fn virtual_cards_minor_foundation(&self) -> [Vec<Card>; 4] {
    self
      .minor_foundation_maxes
      .iter()
      .enumerate()
      .map(|(suit_idx, high)| {
        let suit = Suit::Minor(MinorSuit::n(suit_idx as u8).unwrap());
        match high {
          None => Vec::new(),
          Some(high) => (Card::MINOR_ARCANA_MIN..=*high)
            .map(|idx| Card::new(suit, idx))
            .collect(),
        }
      })
      .collect::<Vec<_>>()
      .try_into()
      .unwrap()
  }

  pub fn minor_foundation_storage(&self) -> Option<&Card> {
    self.minor_foundation_storage.as_ref()
  }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, CopyGetters)]
pub struct Column {
  /// These go top-to-bottom, so the only accessible card is the last one.
  cards: Vec<Card>,
}

impl Column {
  fn new(cards: Vec<Card>) -> Self {
    Self { cards }
  }

  fn empty() -> Self {
    Self::new(Vec::new())
  }
}

impl std::ops::DerefMut for Column {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.cards
  }
}

impl std::ops::Deref for Column {
  type Target = Vec<Card>;

  fn deref(&self) -> &Self::Target {
    &self.cards
  }
}

impl Column {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoardZone {
  Column(usize),
  MinorFoundationStorage,
  // Figure out where it goes
  MinorFoundation,
  MajorFoundation,
}

impl BoardZone {
  /// Whether this zone can only have cards placed in it, and not removed from.
  pub fn is_write_only(&self) -> bool {
    match self {
      BoardZone::Column(_) => false,
      BoardZone::MinorFoundation => true,
      BoardZone::MinorFoundationStorage => false,
      BoardZone::MajorFoundation => true,
    }
  }
}

#[derive(Debug, PartialEq, Eq)]
pub enum CardMoveError {
  NoopMovement,
  EmptySource,
  WriteOnlySource,
  CannotStack,
  WrongTargetZone,
  BlockedByFullMinorStorage,
}
