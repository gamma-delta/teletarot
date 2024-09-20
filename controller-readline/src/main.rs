use itertools::Itertools;
use teletarot_model::{Board, BoardZone, Card};

fn main() {
  let mut board = Board::new(None);
  let mut rl = rustyline::DefaultEditor::new().unwrap();

  println!("Welcome to teletarot.");
  println!("Type where to take a card from and where to put it.");
  println!("- <number>: the specified column (0-indexed)");
  println!("- a: into the minor arcana foundation");
  println!("- A: into the major arcana foundation");
  println!("- s: the storage on top of the minor arcana");
  println!();

  loop {
    print_board(&board);
    println!();

    let line = match rl.readline("> ") {
      Ok(it) => it,
      Err(_) => break,
    };
    let Some((src_s, dst_s)) = line.split_once(' ') else {
      println!("please write a source and dest separated by a space");
      continue;
    };

    let Some(src) = read_zone(src_s) else {
      println!("source zone was written invalid");
      continue;
    };
    let Some(dst) = read_zone(dst_s) else {
      println!("destination zone was written invalid");
      continue;
    };

    let res = board.move_card(src, dst, true);
    if let Err(ono) = res {
      println!("You can't do that! {:?}", &ono);
    }
  }
}

fn read_zone(s: &str) -> Option<BoardZone> {
  if let Ok(num) = usize::from_str_radix(s, 10) {
    return if num > Board::COLUMN_COUNT {
      None
    } else {
      return Some(BoardZone::Column(num));
    };
  }
  match s {
    "a" => Some(BoardZone::MinorFoundation),
    "A" => Some(BoardZone::MajorFoundation),
    "s" => Some(BoardZone::MinorFoundationStorage),
    _ => None,
  }
}

fn print_board(board: &Board) {
  let maj = (Card::MAJOR_ARCANA_MIN..=Card::MAJOR_ARCANA_MAX)
    .map(|idx| {
      let idx = idx as u8;
      let has_left = match board.major_foundation_left_max() {
        None => false,
        Some(lo) => idx <= lo,
      };
      let has_right = match board.major_foundation_right_min() {
        None => false,
        Some(hi) => idx >= hi,
      };
      if has_left || has_right {
        format!("{:>2}", idx)
      } else {
        "..".to_string()
      }
    })
    .join(" ");

  let minor_maxes = board
    .virtual_cards_minor_foundation()
    .map(|v| {
      let card = v.last();
      match card {
        // note i have to call to_string because you have to wire up all
        // the flags yourself when impling Display/Debug and I can't be assed
        Some(c) => format!("{:>3}", c.to_string()),
        None => "...".to_string(),
      }
    })
    .join(" ");

  let columns = (0..Board::COLUMN_COUNT)
    .map(|col_idx| {
      let row = board
        .get_column(col_idx)
        .iter()
        .map(|card| format!("{:>3}", card.to_string()))
        .join(" ");
      format!("{:>2}. {}", col_idx, row)
    })
    .join("\n");

  println!("~{{{}}}~", maj);
  print!("[{}]", minor_maxes);
  if let Some(store) = board.minor_foundation_storage() {
    print!(" -[{:>3}]-", store);
  }
  println!();

  println!("{}", columns);
}
