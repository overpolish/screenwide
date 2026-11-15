// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each fixture is a page as a browser drew it, in grey, and the lines
//! Vision read off it as the app reads them (`fixtures/`).

use serde::Deserialize;

use crate::{find, Block, Cell, Glyph, Line, Picture, Rect, Source, Table};

#[derive(Deserialize)]
struct FixtureLine {
  text: String,
  bounds: Rect,
}

impl<'de> Deserialize<'de> for Rect {
  fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    #[derive(Deserialize)]
    struct Fields {
      x: f64,
      y: f64,
      width: f64,
      height: f64,
    }
    let Fields {
      x,
      y,
      width,
      height,
    } = Fields::deserialize(deserializer)?;
    Ok(Rect {
      x,
      y,
      width,
      height,
    })
  }
}

fn tables(name: &str) -> Vec<Table> {
  let folder = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("fixtures");
  let lines: Vec<FixtureLine> =
    serde_json::from_str(&std::fs::read_to_string(folder.join(format!("{name}.json"))).unwrap())
      .unwrap();
  let grey = image::open(folder.join(format!("{name}.png")))
    .unwrap()
    .into_rgba8();
  let lines: Vec<Line> = lines
    .iter()
    .map(|line| Line {
      text: &line.text,
      bounds: line.bounds,
      glyphs: &[],
    })
    .collect();
  find(
    &lines,
    Picture {
      rgba: grey.as_raw(),
      width: grey.width(),
      height: grey.height(),
    },
  )
}

fn tab_separated(name: &str) -> Vec<String> {
  tables(name).iter().map(Table::tab_separated).collect()
}

#[test]
fn a_ruled_table_is_read_cell_by_cell() {
  assert_eq!(
    tab_separated("ruled"),
    ["Region\tQ1\tQ2\tQ3\tOwner\n\
      North\t12,400\t13,050\t11,900\tAna Silva\n\
      South\t9,800\t10,120\t10,640\tBen Okafor\n\
      East\t15,210\t14,980\t16,300\tChen Wei\n\
      West\t7,450\t\t8,010\tDana Kim\n\
      Total\t44,860\t38,150\t46,850\t"]
  );
}

#[test]
fn rows_read_as_one_line_are_parted_where_the_picture_shows_the_gaps() {
  // Vision reads most of these rows as one line, with each character an
  // even share of it; and "23" and "_windowserver" as one word, which no
  // gap can part.
  assert_eq!(
    tab_separated("unruled"),
    ["Process Name\t% CPU\tCPU Time\tThreads\tUser\n\
      WindowServer\t18.2\t4:12:09.33\t23_windowserver\t\n\
      Safari\t6.4\t1:02:44.10\t41\tdom\n\
      kernel_task\t5.9\t3:55:20.02\t512\troot\n\
      Screenwide\t2.1\t12:08.77\t37\tdom\n\
      Finder\t0.3\t8:41.25\t9\tdom\n\
      mds_stores\t0.2\t44:17.90\t6\troot"]
  );
}

#[test]
fn a_tight_table_is_parted_at_the_gutters_its_other_rows_show() {
  // Size and Kind are a few pixels apart, no further than the words of a
  // date: most rows part them, and the rest are cut where those rows do.
  assert_eq!(
    tab_separated("finder"),
    ["Name\tDate Modified\tSize\tKind\n\
      Invoice March.pdf\tToday at 09:14\t182 KB\tPDF document\n\
      screenshot 2026-03-11.png\tYesterday at 17:02\t2.4 MB\tPNG image\n\
      notes.md\t9 March 2026 at 11:30\t4 KB\tMarkdown\n\
      Archive\t2 March 2026 at 08:45\t--\tFolder\n\
      budget.numbers\t27 February 2026 at 16:20\t311 KB\tNumbers Spreadsheet"]
  );
}

#[test]
fn a_table_among_text_and_a_list_is_the_table_alone() {
  assert_eq!(
    tab_separated("mixed"),
    ["Run\tTime\tMemory\tResult\n\
      1\t4.2 s\t512 MB\tPassed\n\
      2\t1.9 s\t530 MB\tPassed\n\
      3\t2.1 s\t498 MB\tFailed"]
  );
}

#[test]
fn a_cell_the_reader_missed_leaves_its_row_whole() {
  assert_eq!(
    tab_separated("dark"),
    ["NAME\tREADY\tSTATUS\tRESTARTS\tAGE\n\
      api-7d9f8b6c5-x2kqp\t1/1\tRunning\t\t3d2h\n\
      worker-5c7b9-mm4tz\t1/1\tRunning\t2\t3d2h\n\
      redis-0\t1/1\tRunning\t\t12d\n\
      cron-28911340-zl8sv\t0/1\tCompleted\t\t41m"]
  );
}

#[test]
fn numbers_set_to_the_end_of_a_column_headed_at_its_start_stay_in_it() {
  let tables = tab_separated("sheet");
  assert_eq!(tables.len(), 1);
  let rows: Vec<&str> = tables[0].lines().collect();
  assert_eq!(rows[1], "1\tDate\tItem\tQty\tUnit\tTotal\tPaid");
  assert_eq!(rows[3], "\t2026-03-02\tToner black\t2\t61.20\t122.40\tNo");
}

#[test]
fn a_cells_wrapped_lines_are_one_cell() {
  let tables = tab_separated("wrapped");
  assert_eq!(tables.len(), 1);
  assert!(tables[0]
    .lines()
    .any(|row| row
      == "Team\t$29\t$290\tUp to ten seats with shared projects and priority email support"));
}

#[test]
fn a_cell_of_several_lines_beside_one_line_cells_is_one_row() {
  // A list of two bullets in the first cell, the other cells level with
  // the middle of the pair.
  assert_eq!(
    tab_separated("bullets"),
    ["Tables\tAre\tCool\n\
      col 3 is\tright-aligned\t$1600\n\
      col 2 is\tcentered\t$12\n\
      zebra stripes\tare neat\t$1\n\
      • item1 • item2\tSee the list\tfrom the first collum"]
  );
}

#[test]
fn tables_under_headings_on_one_page_are_each_whole() {
  let rows = "Tables\tAre\tCool\n\
              col 3 is\tright-aligned\t$1600\n\
              col 2 is\tcentered\t$12\n\
              zebra stripes\tare neat\t$1\n";
  // The browser's own button covers the second list's bullets.
  assert_eq!(
    tab_separated("two-tables"),
    [
      format!("{rows}• item1 • item2\tSee the list\tfrom the first collum"),
      format!("{rows}item1 item2\tSee the list\tfrom the first collum"),
    ]
  );
}

#[test]
fn text_read_on_a_dark_table_with_faint_marks_lands_in_its_cells() {
  // Light text on a dark page, each name and prefix in faint brackets the
  // reader catches some of; it misses the second column's [[O]].
  assert_eq!(
    tab_separated("dark-faint"),
    ["Seeds\tPrefix\tDescription\n\
      [| Sparks*]]\t［*］\tTagged anywhere\n\
      [|Free\t[[F]]\tFreewriting passages\n\
      Originators\t\tIdea/Setup/Audience mixes\n\
      Dumps\t[[D]]\tText dumps on topics\n\
      [Stubs\t[[S]]\tNew note placeholders"]
  );
}

#[test]
fn names_and_values_side_by_side_are_a_table() {
  assert_eq!(
    tab_separated("keyvalue"),
    ["Chip\tApple M3 Pro\nMemory\t36 GB\nStartup disk\tMacintosh HD\nSerial number\tC02XK1ABCDEF\nmacOS\tTahoe 26.1"]
  );
}

#[test]
fn text_in_columns_code_and_lists_are_not_tables() {
  for name in ["prose", "code", "python", "numbered"] {
    assert_eq!(tables(name), [], "{name}");
  }
}

/// A line of words each a glyph's width apart, from `x`, on a blank page.
fn glyphs(words: &[&str], x: f64, y: f64, gap: f64) -> (String, Vec<Glyph>) {
  let mut glyphs = Vec::new();
  let (mut offset, mut at) = (0, x);
  for word in words {
    for _ in word.encode_utf16() {
      glyphs.push(Glyph {
        start: offset,
        end: offset + 1,
        bounds: Rect {
          x: at,
          y,
          width: 0.01,
          height: 0.03,
        },
      });
      offset += 1;
      at += 0.01;
    }
    offset += 1;
    at += gap;
  }
  (words.join(" "), glyphs)
}

#[test]
fn without_ink_to_go_by_the_readers_boxes_part_the_cells() {
  let rows = [["Name", "Size"], ["a.txt", "4 KB"], ["b.png", "12 KB"]];
  let lines: Vec<(String, Vec<Glyph>)> = rows
    .iter()
    .enumerate()
    .map(|(row, cells)| glyphs(cells, 0.1, 0.1 + 0.05 * row as f64, 0.1))
    .collect();
  let lines: Vec<Line> = lines
    .iter()
    .map(|(text, glyphs)| Line {
      text,
      bounds: Rect {
        x: 0.1,
        y: glyphs[0].bounds.y,
        width: 0.3,
        height: 0.03,
      },
      glyphs,
    })
    .collect();
  let blank = vec![255_u8; 100 * 100 * 4];
  let found = find(
    &lines,
    Picture {
      rgba: &blank,
      width: 100,
      height: 100,
    },
  );
  assert_eq!(found.len(), 1);
  assert_eq!(
    found[0].tab_separated(),
    "Name\tSize\na.txt\t4 KB\nb.png\t12 KB"
  );
}

/// A cell whose text is line `line`, a line to a cell.
fn cell(text: &str, column: usize, span: usize, line: usize) -> Cell {
  Cell {
    text: text.to_owned(),
    column,
    span,
    sources: vec![selected(line)],
    bounds: Rect::default(),
  }
}

fn selected(line: usize) -> Source {
  Source {
    line,
    start: 0,
    end: 1,
  }
}

#[test]
fn copied_text_and_tables_keep_their_order_and_a_table_its_cells() {
  let table = Table {
    columns: 3,
    rows: vec![
      vec![cell("Price", 1, 2, 0)],
      vec![cell("A & B", 0, 1, 1), cell("<1>", 2, 1, 1)],
    ],
  };
  let blocks = [
    Block::Text(vec!["Results:".to_owned(), "a < b".to_owned()]),
    Block::Table(table),
    Block::Text(vec!["Done.".to_owned()]),
  ];
  assert_eq!(
    crate::html(&blocks),
    "<meta charset=\"utf-8\"><p>Results:<br>a &lt; b</p>\
     <table><tr><td></td><td colspan=\"2\">Price</td></tr>\
     <tr><td>A &amp; B</td><td></td><td>&lt;1&gt;</td></tr></table>\
     <p>Done.</p>"
  );
  assert_eq!(
    crate::text(&blocks),
    "Results:\na < b\n\tPrice\t\nA & B\t\t<1>\nDone."
  );
}

#[test]
fn a_selection_copies_the_cells_from_its_first_to_its_last() {
  let table = Table {
    columns: 3,
    rows: vec![
      vec![cell("a", 0, 1, 0), cell("b", 1, 1, 1), cell("c", 2, 1, 2)],
      vec![cell("d", 0, 1, 3), cell("e", 1, 2, 4)],
      vec![cell("g", 0, 1, 6), cell("h", 1, 1, 7), cell("i", 2, 1, 8)],
    ],
  };
  let within = table.within(&[selected(0), selected(3)]).unwrap();
  assert_eq!(within.tab_separated(), "a\nd");
  let within = table.within(&[selected(1), selected(8)]).unwrap();
  assert_eq!(within.tab_separated(), "b\tc\ne\t\nh\ti");
  assert_eq!(table.within(&[selected(9)]), None);
}
