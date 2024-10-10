use ahash::{AHashMap, AHashSet};
use backtrack::problem::Scope;
use itertools::Itertools;

use crate::{Board, BoardZone};

/// The further "into the future" this function can check,
/// the more of the search tree we can prune!
fn board_is_dead(board: &Board) -> bool {
  let availables = {
    let mut av = Vec::new();
    match board.minor_foundation_storage() {
      Some(card) => av.push(card.clone()),
      // Game can't be blocked if there's a free space to stick things in
      None => return false,
    }

    for col in &board.columns {
      match col.last() {
        Some(it) => av.push(it.clone()),
        None => return false,
      };
    }
    av
  };

  // this major and minor has nothing to do with major and minor arcana
  let mut triangle_idxes =
    (0..availables.len()).flat_map(|maj| (0..maj).map(move |min| (maj, min)));
  triangle_idxes.all(|(l, r)| availables[l].can_stack(&availables[r]))
}

struct Solver {
  seen_states: AHashSet<Board>,
  steps: Vec<Step>,
}

impl Solver {
  fn take_step(&mut self) -> bool {
    let depth = self.steps.len();

    let step = self.steps.last_mut().unwrap();
    let mut board2 = step.state.clone();

    let move_res =
      board2.move_card(step.trying_zones.src, step.trying_zones.dst, true);
    let next_step = match move_res {
      Ok(()) => {
        // hey! this was a valid move
        board2.check_automove_cards();
        if board_is_dead(&board2) {
          println!("#{}: board is dead", depth);
          None
        } else if !self.seen_states.insert(board2.clone()) {
          // println!("#{}: this board has been seen before", depth);
          None
        } else {
          if board2.is_solved() {
            return true;
          }
          self.seen_states.insert(board2.clone());
          // println!(
          //   "#{}: seeking from {:?} -> {:?}",
          //   depth, &step.trying_src, &step.trying_dst
          // );
          Some(Step::new(board2))
        }
      }
      Err(_) => {
        // println!("#{}: why did we even try that move", depth);
        None
      }
    };

    if let Some(next_step) = next_step {
      self.steps.push(next_step);
    } else {
      // Try to advance this step
      if step.trying_zones.next().is_none() {
        // This line of inquiry is thru with
        println!("#{}: tried all moves from this position", depth);
        self.steps.pop();
      }
    }
    false
  }
}

struct Step {
  state: Board,
  trying_zones: ZonesIter,
}

impl Step {
  fn new(board: Board) -> Self {
    Self {
      state: board.clone(),
      trying_zones: ZonesIter::new(),
    }
  }
}

struct ZonesIter {
  src: BoardZone,
  dst: BoardZone,
  finished: bool,
}

impl ZonesIter {
  fn new() -> Self {
    Self {
      src: BoardZone::Column(1),
      dst: BoardZone::Column(0),
      finished: false,
    }
  }

  fn next_board_zone(zone: &BoardZone) -> Option<BoardZone> {
    Some(match zone {
      &BoardZone::Column(c) if c < Board::COLUMN_COUNT - 1 => {
        BoardZone::Column(c + 1)
      }
      BoardZone::Column(_) => BoardZone::MinorFoundationStorage,
      BoardZone::MinorFoundationStorage => return None,
      _ => unreachable!(),
    })
  }
}

impl Iterator for ZonesIter {
  type Item = (BoardZone, BoardZone);

  fn next(&mut self) -> Option<Self::Item> {
    if self.finished {
      return None;
    }

    // Avoid trying to move a card into itself to save time
    // Rust doesn't have do-while! But this is actually a time to use it.
    // You can hack a do-while like this, by using a block as the *condition*
    // for the while loop, and have an empty body.
    // Jank as all get-out but works.
    while {
      if let Some(next) = Self::next_board_zone(&self.src) {
        self.src = next;
      } else {
        self.src = BoardZone::Column(0);
        if let Some(next) = Self::next_board_zone(&self.dst) {
          self.dst = next;
        } else {
          self.finished = true;
          return None;
        }
      }
      self.src == self.dst
    } {}

    Some((self.src, self.dst))
  }
}

pub fn try_solve(board: &Board) -> Option<Vec<(BoardZone, BoardZone)>> {
  let mut board2 = board.clone();
  board2.check_automove_cards();
  let mut solver = Solver {
    seen_states: AHashSet::new(),
    steps: vec![Step::new(board2)],
  };

  while !solver.steps.is_empty() {
    let success = solver.take_step();
    if success {
      // woooooohoooo!
      let solution = solver
        .steps
        .into_iter()
        .map(|step| (step.trying_zones.src, step.trying_zones.dst))
        .collect_vec();
      return Some(solution);
    }
  }

  None
}
