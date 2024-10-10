use fastrand::Rng;
use itertools::Itertools;

use crate::{Board, Card, MinorSuit, Suit};

pub fn shuffled_random(seed: Option<u64>) -> Board {
  let mut rng = match seed {
    Some(seed) => Rng::with_seed(seed),
    None => Rng::new(),
  };

  let mut all_the_cards = {
    // Skip aces
    let minors = ((Card::MINOR_ARCANA_MIN + 1)..=Card::MINOR_ARCANA_MAX)
      .flat_map(|minor_idx| {
        (0..=3).map(move |suit| {
          Card::new(Suit::Minor(MinorSuit::n(suit).unwrap()), minor_idx)
        })
      });
    let majors = (Card::MAJOR_ARCANA_MIN..=Card::MAJOR_ARCANA_MAX)
      .map(|major_idx| Card::new(Suit::MajorArcana, major_idx));
    minors.chain(majors)
  }
  .collect_vec();
  rng.shuffle(&mut all_the_cards);

  let mut board = Board::empty();
  board.minor_foundation_maxes = [Some(1); 4];
  for (idx, chunk) in all_the_cards
    .chunks_exact(Board::DESIRED_STACK_HEIGHT)
    .enumerate()
  {
    // skip the middle column
    let middle = Board::COLUMN_COUNT / 2;
    let col_idx = if idx < middle { idx } else { idx + 1 };
    board.get_column_mut(col_idx).extend_from_slice(chunk);
  }

  board
}
