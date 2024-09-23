use cursive::theme::BaseColor;
use teletarot_model::{MinorSuit, Suit};

pub fn suit_color(suit: Suit) -> BaseColor {
  match suit {
    Suit::Minor(MinorSuit::Pentacles) => BaseColor::Yellow,
    Suit::Minor(MinorSuit::Cups) => BaseColor::Red,
    Suit::Minor(MinorSuit::Wands) => BaseColor::Green,
    Suit::Minor(MinorSuit::Swords) => BaseColor::Cyan,
    Suit::MajorArcana => BaseColor::Magenta,
  }
}
