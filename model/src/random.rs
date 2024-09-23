use std::array;

use fastrand::Rng;
use itertools::Itertools;

use crate::{Board, Card, Column, MinorSuit, Suit};

const DESIRED_STACK_HEIGHT: usize =
  Board::DECK_SIZE / (Board::COLUMN_COUNT - 1);

struct BoardGenerator {
  rng: Rng,
  source_stacks: Vec<Vec<Card>>,
  board_columns: [Vec<Card>; Board::COLUMN_COUNT - 1],
}

impl BoardGenerator {
  fn new(seed: Option<u64>) -> Self {
    let mut rng = match seed {
      Some(seed) => Rng::with_seed(seed),
      None => Rng::new(),
    };

    let middle_arcana = rng.u8(0..=Card::MAJOR_ARCANA_MAX);
    let left_arcana = (0..middle_arcana)
      .map(|idx| Card::new(Suit::MajorArcana, idx))
      .collect_vec();
    let right_arcana = (middle_arcana..=Card::MAJOR_ARCANA_MAX)
      .map(|idx| Card::new(Suit::MajorArcana, idx))
      .collect_vec();
    let normals = (0..=3).map(|suit_idx| {
      let mut whole_suit = (Card::MINOR_ARCANA_MIN + 1
        ..=Card::MINOR_ARCANA_MAX)
        .map(|card_idx| {
          Card::new(Suit::Minor(MinorSuit::n(suit_idx).unwrap()), card_idx)
        })
        .collect_vec();
      rng.shuffle(&mut whole_suit);
      whole_suit
    });

    let mut stacks = vec![left_arcana, right_arcana];
    stacks.extend(normals);

    Self {
      rng,
      source_stacks: stacks,
      board_columns: array::from_fn(|_| Vec::new()),
    }
  }

  fn any_source_stacks_left(&self) -> bool {
    self.source_stacks.iter().any(|v| !v.is_empty())
  }

  fn move_once(&mut self) -> bool {
    let maybe_pickup_from_col = 'mpfc: {
      // Check to see if we can pick up from a valid stack in the columns
      let column_idx_lookup = {
        let mut cil = (0..self.board_columns.len()).collect_vec();
        self.rng.shuffle(&mut cil);
        cil
      };
      for idx in 0..self.board_columns.len() {
        let idx = column_idx_lookup[idx];
        let col = &mut self.board_columns[idx];
        let Some(last_card) = col.last() else {
          continue;
        };
        let last_card = last_card.clone();

        let ok_stack = if col.len() == 1 {
          true
        } else {
          last_card.can_stack(&col[col.len() - 2])
        };
        if ok_stack {
          break 'mpfc Some(idx);
        }
      }

      None
    };

    let maybe_pickup_from_sources = 'mpfs: {
      let avail_source_stack_idxes = self
        .source_stacks
        .iter()
        .enumerate()
        .filter_map(|(idx, v)| (!v.is_empty()).then_some(idx))
        .collect_vec();
      if avail_source_stack_idxes.is_empty() {
        break 'mpfs None;
      }

      let stack_idx = avail_source_stack_idxes
        [self.rng.usize(..avail_source_stack_idxes.len())];
      // phew
      Some(stack_idx)
    };

    let which = match (maybe_pickup_from_sources, maybe_pickup_from_col) {
      (Some(mpfs), Some(_mpfc)) if self.rng.f32() < 0.5 => Ok(mpfs),
      (Some(_mpfs), Some(mpfc)) => Err(mpfc),
      (Some(mpfs), None) => Ok(mpfs),
      (None, Some(mpfc)) => Err(mpfc),
      _ => return false,
    };

    match which {
      Ok(stack_idx) => {
        let card = self.source_stacks[stack_idx].pop().unwrap();
        // If possible, find a stack with size under the desired size
        let ok_dest_col_idxs = self
          .board_columns
          .iter()
          .enumerate()
          .filter_map(|(idx, col)| {
            (col.len() < DESIRED_STACK_HEIGHT).then_some(idx)
          })
          .collect_vec();
        let dest_col_idx = if ok_dest_col_idxs.is_empty() {
          self.rng.usize(0..self.board_columns.len())
        } else {
          ok_dest_col_idxs[self.rng.usize(..ok_dest_col_idxs.len())]
        };
        self.board_columns[dest_col_idx].push(card);
      }
      Err(col_idx) => {
        let src_col = &mut self.board_columns[col_idx];
        let card = src_col.pop().unwrap();
        let src_col_len = src_col.len();
        // Find a column shorter than this one
        let ok_dest_col_idxs = self
          .board_columns
          .iter()
          .enumerate()
          .filter_map(|(idx, col)| {
            (idx != col_idx && col.len() < src_col_len).then_some(idx)
          })
          .collect_vec();
        let dest_col_idx = if ok_dest_col_idxs.is_empty() {
          self.rng.usize(0..self.board_columns.len())
        } else {
          ok_dest_col_idxs[self.rng.usize(..ok_dest_col_idxs.len())]
        };
        self.board_columns[dest_col_idx].push(card);
      }
    }

    true
  }

  fn force_equi_height(&mut self) {
    loop {
      let mut too_talls = Vec::new();
      let mut too_shorts = Vec::new();
      for (idx, col) in self.board_columns.iter().enumerate() {
        let len = col.len();
        if len > DESIRED_STACK_HEIGHT {
          too_talls.push(idx);
        } else if len < DESIRED_STACK_HEIGHT {
          too_shorts.push(idx);
        }
      }

      if too_talls.is_empty() && too_shorts.is_empty() {
        break;
      }

      let tall_idx = too_talls[self.rng.usize(..too_talls.len())];
      let short_idx = too_shorts[self.rng.usize(..too_shorts.len())];
      let card = self.board_columns[tall_idx].pop().unwrap();
      self.board_columns[short_idx].push(card);
    }
  }

  fn consume_to_board(self) -> Board {
    assert!(!self.any_source_stacks_left());

    let space_center = Board::COLUMN_COUNT / 2;
    let spaced_columns = (0..Board::COLUMN_COUNT)
      .map(|col_idx| {
        if col_idx < space_center {
          Column::new(self.board_columns[col_idx].clone())
        } else if col_idx == space_center {
          Column { cards: Vec::new() }
        } else {
          Column::new(self.board_columns[col_idx - 1].clone())
        }
      })
      .collect_vec()
      .try_into()
      .unwrap();

    Board {
      columns: spaced_columns,
      minor_foundation_storage: None,
      minor_foundation_maxes: [Some(1); 4],
      major_foundation_left_max: None,
      major_foundation_right_min: None,
    }
  }
}

pub fn random_board(seed: Option<u64>) -> Board {
  let mut generator = BoardGenerator::new(seed);

  while generator.any_source_stacks_left() {
    generator.move_once();
  }
  // Shuffle it a little more
  for _ in 0..50 {
    let any_more = generator.move_once();
    if !any_more {
      break;
    }
  }
  generator.force_equi_height();

  generator.consume_to_board()
}
