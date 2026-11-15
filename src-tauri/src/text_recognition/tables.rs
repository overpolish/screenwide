// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The tables among what was read (`screenwide_tables`), and copying what
//! was read so its tables paste as rows and columns.

use screenwide_imaging::CapturedImage;
use screenwide_tables::{Block, Glyph, Line, Picture, Rect, Source, Table};

use super::{text_selection::source_text, RecognizedLine, TextRecognitionResult, TextRect};

fn rect(bounds: TextRect) -> Rect {
  Rect {
    x: bounds.x,
    y: bounds.y,
    width: bounds.width,
    height: bounds.height,
  }
}

/// The tables among `lines`, read off `image`, the area they were read in.
pub(super) fn find(lines: &[RecognizedLine], image: &CapturedImage) -> Vec<Table> {
  let glyphs: Vec<Vec<Glyph>> = lines
    .iter()
    .map(|line| {
      line
        .characters
        .iter()
        .map(|character| Glyph {
          start: character.start,
          end: character.end,
          bounds: rect(character.bounds),
        })
        .collect()
    })
    .collect();
  let lines: Vec<Line> = lines
    .iter()
    .zip(&glyphs)
    .map(|(line, glyphs)| Line {
      text: &line.text,
      bounds: rect(line.bounds),
      glyphs,
    })
    .collect();
  screenwide_tables::find(
    &lines,
    Picture {
      rgba: &image.rgba,
      width: image.width,
      height: image.height,
    },
  )
}

/// What copying puts on the clipboard: the text, and HTML where a table is
/// among what is copied.
#[derive(Debug, PartialEq)]
pub(super) struct Copied {
  pub text: String,
  pub html: Option<String>,
}

/// What copying `selected` copies of `result`, or all of it with `None`, in
/// reading order: the lines outside the tables as text, and each table it
/// touches, at its first line, as the cells from the first it touches to
/// the last. A line any cell is read from is the table's, so a row the
/// reader gave as one line, and again cell by cell, is copied once.
pub(super) fn copied(result: &TextRecognitionResult, selected: Option<&[Source]>) -> Copied {
  let owner: Vec<Option<usize>> = (0..result.lines.len())
    .map(|line| {
      result.tables.iter().position(|table| {
        table
          .rows
          .iter()
          .flatten()
          .any(|cell| cell.sources.iter().any(|source| source.line == line))
      })
    })
    .collect();
  let whole = |line: usize| Source {
    line,
    start: 0,
    end: result.lines[line].text.encode_utf16().count(),
  };
  let mut blocks: Vec<Block> = Vec::new();
  let mut placed = vec![false; result.tables.len()];
  for (line, owner) in owner.into_iter().enumerate() {
    if let Some(table) = owner {
      if !std::mem::replace(&mut placed[table], true) {
        let copied = match selected {
          None => Some(result.tables[table].clone()),
          Some(selected) => result.tables[table].within(selected),
        };
        blocks.extend(copied.map(Block::Table));
      }
      continue;
    }
    let texts: Vec<String> = match selected {
      None => vec![source_text(result, whole(line))],
      Some(selected) => selected
        .iter()
        .filter(|source| source.line == line)
        .map(|source| source_text(result, *source))
        .collect(),
    };
    if texts.is_empty() {
      continue;
    }
    match blocks.last_mut() {
      Some(Block::Text(lines)) => lines.extend(texts),
      _ => blocks.push(Block::Text(texts)),
    }
  }
  let has_table = blocks.iter().any(|block| matches!(block, Block::Table(_)));
  Copied {
    text: screenwide_tables::text(&blocks),
    html: has_table.then(|| screenwide_tables::html(&blocks)),
  }
}

#[cfg(test)]
mod tests {
  use screenwide_tables::Cell;

  use super::*;
  use crate::text_recognition::RecognizedLine;

  fn line(text: &str) -> RecognizedLine {
    RecognizedLine {
      text: text.to_owned(),
      confidence: 1.0,
      bounds: TextRect {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 0.1,
      },
      characters: Vec::new(),
    }
  }

  /// A cell read from each of `read`: a line and where in it the text starts.
  fn cell(text: &str, column: usize, read: &[(usize, usize)]) -> Cell {
    Cell {
      text: text.to_owned(),
      column,
      span: 1,
      sources: read
        .iter()
        .map(|&(line, start)| Source {
          line,
          start,
          end: start + text.len(),
        })
        .collect(),
      bounds: Rect::default(),
    }
  }

  /// A heading, a table whose head the reader gave as one line and again
  /// cell by cell, and a closing line.
  fn result() -> TextRecognitionResult {
    let texts = ["Results", "Run Time", "Run", "Time", "1 4.2 s", "Done"];
    TextRecognitionResult {
      lines: texts.iter().map(|text| line(text)).collect(),
      qr_codes: Vec::new(),
      text: texts.join("\n"),
      tables: vec![Table {
        columns: 2,
        rows: vec![
          vec![
            cell("Run", 0, &[(1, 0), (2, 0)]),
            cell("Time", 1, &[(1, 4), (3, 0)]),
          ],
          vec![cell("1", 0, &[(4, 0)]), cell("4.2 s", 1, &[(4, 2)])],
        ],
      }],
    }
  }

  #[test]
  fn copying_all_keeps_the_text_around_a_table_and_reads_its_rows_once() {
    let copied = copied(&result(), None);
    assert_eq!(copied.text, "Results\nRun\tTime\n1\t4.2 s\nDone");
    assert_eq!(
      copied.html.as_deref(),
      Some(
        "<meta charset=\"utf-8\"><p>Results</p>\
         <table><tr><td>Run</td><td>Time</td></tr><tr><td>1</td><td>4.2 s</td></tr></table>\
         <p>Done</p>"
      )
    );
  }

  #[test]
  fn a_selection_copies_its_text_and_the_cells_it_touches() {
    let selected = [
      Source {
        line: 0,
        start: 3,
        end: 7,
      },
      Source {
        line: 4,
        start: 2,
        end: 7,
      },
    ];
    let copied = copied(&result(), Some(&selected));
    assert_eq!(copied.text, "ults\n4.2 s");
  }

  #[test]
  fn text_without_a_table_is_copied_as_text_alone() {
    let selected = [Source {
      line: 5,
      start: 0,
      end: 4,
    }];
    assert_eq!(
      copied(&result(), Some(&selected)),
      Copied {
        text: "Done".to_owned(),
        html: None,
      }
    );
  }
}
