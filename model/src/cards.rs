use core::panic;
use std::fmt::{Debug, Display};

use getset::CopyGetters;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, enumn::N)]
#[repr(u8)]
pub enum MinorSuit {
  // today i learned these are not "stars," these are "pentacles".
  // There appears to be no standard order for these things so I'm using the
  // ordering Zach uses
  Pentacles,
  Cups,
  Swords,
  Wands,
}

impl MinorSuit {
  pub fn short_char(&self) -> char {
    match self {
      MinorSuit::Pentacles => 'p',
      MinorSuit::Cups => 'c',
      MinorSuit::Swords => 's',
      MinorSuit::Wands => 'w',
    }
  }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub enum Suit {
  Minor(MinorSuit),
  MajorArcana,
}

impl Suit {
  pub fn is_minor(&self) -> bool {
    match self {
      Suit::Minor(_) => true,
      Suit::MajorArcana => false,
    }
  }
}

/// A tarot card on the board.
/// This deliberately does not implement `Copy`, to encourage move semantics
/// to avoid duplicating cards on accident.
#[derive(Clone, PartialEq, Eq, Hash, CopyGetters)]
#[getset(get_copy = "pub")]
pub struct Card {
  suit: Suit,
  number: u8,
}

impl Card {
  pub const MINOR_ARCANA_MIN: u8 = 1;
  pub const MINOR_ARCANA_MAX: u8 = 13;
  pub const MAJOR_ARCANA_MIN: u8 = 0;
  pub const MAJOR_ARCANA_MAX: u8 = 21;

  pub fn new(suit: Suit, number: u8) -> Self {
    if suit.is_minor() {
      if !(Self::MINOR_ARCANA_MIN..=Self::MINOR_ARCANA_MAX).contains(&number) {
        panic!("minor arcana must be between 1 and 13 but got {}", number)
      }
    } else {
      if !(Self::MAJOR_ARCANA_MIN..=Self::MAJOR_ARCANA_MAX).contains(&number) {
        panic!("major arcana must be between 0 and 21 but got {}", number)
      }
    }

    Self { suit, number }
  }

  /// Return if this card can stack on or under the other card
  pub fn can_stack(&self, other: &Card) -> bool {
    self.suit == other.suit && self.number.abs_diff(other.number) == 1
  }

  /// Get the string representation of the number of one of the minor arcana.
  pub fn minor_number_string(num: u8) -> String {
    match num {
      1 => "A".to_string(),
      2..=10 => num.to_string(),
      11 => "J".to_string(),
      12 => "Q".to_string(),
      13 => "K".to_string(),
      ono => format!("{}!", ono),
    }
  }
}

impl Display for Card {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    if self.suit.is_minor() {
      f.write_str(&Card::minor_number_string(self.number))?;
    } else {
      write!(f, "{}", self.number.to_string())?;
    }
    write!(
      f,
      "{}",
      match self.suit {
        Suit::MajorArcana => 'A',
        Suit::Minor(m) => m.short_char(),
      }
    )?;
    Ok(())
  }
}

impl Debug for Card {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.write_str("Card(")?;
    write!(f, "{}", self)?;
    f.write_str(")")
  }
}
