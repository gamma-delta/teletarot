use cursive::{theme::Style, utils::markup::StyledString, Printer, Vec2};
use itertools::{iproduct, Itertools};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoxSide {
  NWCorner,
  Horz,
  NECorner,
  Vert,
  SWCorner,
  SECorner,
}

impl BoxSide {
  pub fn from_delta_in_box(delta: Vec2, size: Vec2) -> Self {
    let kinds = delta.zip_map(size, |dn, sz| {
      if dn == 0 {
        0
      } else if dn == sz - 1 {
        2
      } else {
        1
      }
    });
    match (kinds.x, kinds.y) {
      (0, 0) => Self::NWCorner,
      (1, 0 | 2) => Self::Horz,
      (2, 0) => Self::NECorner,
      (0 | 2, 1) => Self::Vert,
      (0, 2) => Self::SWCorner,
      (2, 2) => Self::SECorner,
      ono => unreachable!("{:?}", ono),
    }
  }

  pub fn box_drawing_char(&self, thick: bool) -> char {
    match (thick, self) {
      (true, BoxSide::NWCorner) => '╔',
      (true, BoxSide::Horz) => '═',
      (true, BoxSide::NECorner) => '╗',
      (true, BoxSide::Vert) => '║',
      (true, BoxSide::SWCorner) => '╚',
      (true, BoxSide::SECorner) => '╝',
      (false, BoxSide::NWCorner) => '┌',
      (false, BoxSide::Horz) => '─',
      (false, BoxSide::NECorner) => '┐',
      (false, BoxSide::Vert) => '│',
      (false, BoxSide::SWCorner) => '└',
      (false, BoxSide::SECorner) => '┘',
    }
  }

  pub fn draw_box(
    printer: &Printer,
    pos: impl Into<Vec2>,
    size: impl Into<Vec2>,
    style: Style,
    thick: bool,
  ) {
    let pos = pos.into();
    let size = size.into();
    let edges = (0..size.y).flat_map(|y| {
      if y == 0 || y == size.y - 1 {
        (0..size.x).map(|x| (x, y)).collect_vec()
      } else {
        vec![(0, y), (size.x - 1, y)]
      }
    });

    for (dx, dy) in edges {
      let here = pos + (dx, dy);

      let ch = BoxSide::from_delta_in_box((dx, dy).into(), size)
        .box_drawing_char(thick);
      printer.print_styled(here, &StyledString::styled(ch, style));
    }
  }
}
