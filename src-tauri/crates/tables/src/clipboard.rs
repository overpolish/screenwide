// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What is copied as other apps take it from the clipboard: HTML, in which
//! spreadsheets, documents and mail read a table as rows and columns, and
//! text with a tab between a table's cells and a line a row, which
//! spreadsheets read as cells and anything else as text.

use crate::{Cell, Source, Table};

impl Table {
  /// The part of the table that `selected` touches, from its first row and
  /// column to its last; none where it touches no cell.
  pub fn within(&self, selected: &[Source]) -> Option<Table> {
    let touched = |cell: &Cell| {
      cell.sources.iter().any(|source| {
        selected.iter().any(|other| {
          other.line == source.line && other.start < source.end && source.start < other.end
        })
      })
    };
    let cells = self.rows.iter().enumerate().flat_map(|(row, cells)| {
      cells
        .iter()
        .filter(|cell| touched(cell))
        .map(move |cell| (row, cell))
    });
    let (mut rows, mut columns) = ((usize::MAX, 0), (usize::MAX, 0));
    for (row, cell) in cells {
      rows = (rows.0.min(row), rows.1.max(row));
      columns = (
        columns.0.min(cell.column),
        columns.1.max(cell.column + cell.span - 1),
      );
    }
    if rows.0 == usize::MAX {
      return None;
    }
    Some(Table {
      columns: columns.1 - columns.0 + 1,
      rows: self.rows[rows.0..=rows.1]
        .iter()
        .map(|cells| {
          cells
            .iter()
            .filter(|cell| cell.column + cell.span > columns.0 && cell.column <= columns.1)
            .map(|cell| {
              let start = cell.column.max(columns.0);
              let end = (cell.column + cell.span - 1).min(columns.1);
              Cell {
                column: start - columns.0,
                span: end - start + 1,
                ..cell.clone()
              }
            })
            .collect()
        })
        .collect(),
    })
  }

  /// A tab between cells and a line a row; a cell over several columns is
  /// in the first of them.
  pub fn tab_separated(&self) -> String {
    self
      .rows
      .iter()
      .map(|cells| {
        let mut texts = vec![""; self.columns];
        for cell in cells {
          texts[cell.column] = cell.text.as_str();
        }
        texts
          .iter()
          .map(|text| text.replace(['\t', '\n', '\r'], " "))
          .collect::<Vec<_>>()
          .join("\t")
      })
      .collect::<Vec<_>>()
      .join("\n")
  }

  /// An HTML table, a cell over several columns spanning them.
  fn html_into(&self, html: &mut String) {
    html.push_str("<table>");
    for cells in &self.rows {
      html.push_str("<tr>");
      let mut column = 0;
      for cell in cells {
        html.push_str(&"<td></td>".repeat(cell.column - column));
        if cell.span > 1 {
          html.push_str(&format!("<td colspan=\"{}\">", cell.span));
        } else {
          html.push_str("<td>");
        }
        escape_into(html, &cell.text);
        html.push_str("</td>");
        column = cell.column + cell.span;
      }
      html.push_str(&"<td></td>".repeat(self.columns.saturating_sub(column)));
      html.push_str("</tr>");
    }
    html.push_str("</table>");
  }
}

/// Part of what is copied, in reading order: lines of text, or a table.
#[derive(Clone, Debug, PartialEq)]
pub enum Block {
  Text(Vec<String>),
  Table(Table),
}

/// `blocks` as HTML: a paragraph of lines, a line break between them, for
/// each run of text, and a table for each table.
pub fn html(blocks: &[Block]) -> String {
  let mut html = String::from("<meta charset=\"utf-8\">");
  for block in blocks {
    match block {
      Block::Text(lines) => {
        html.push_str("<p>");
        for (index, line) in lines.iter().enumerate() {
          if index > 0 {
            html.push_str("<br>");
          }
          escape_into(&mut html, line);
        }
        html.push_str("</p>");
      }
      Block::Table(table) => table.html_into(&mut html),
    }
  }
  html
}

/// `blocks` as text: a line a line, and a table's rows with a tab between
/// their cells.
pub fn text(blocks: &[Block]) -> String {
  blocks
    .iter()
    .map(|block| match block {
      Block::Text(lines) => lines.join("\n"),
      Block::Table(table) => table.tab_separated(),
    })
    .collect::<Vec<_>>()
    .join("\n")
}

fn escape_into(html: &mut String, text: &str) {
  for character in text.chars() {
    match character {
      '&' => html.push_str("&amp;"),
      '<' => html.push_str("&lt;"),
      '>' => html.push_str("&gt;"),
      '"' => html.push_str("&quot;"),
      _ => html.push(character),
    }
  }
}
