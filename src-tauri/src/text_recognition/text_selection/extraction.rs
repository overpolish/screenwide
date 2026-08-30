// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::{TextRange, TextRecognitionResult, TextRect};

pub(crate) fn paragraph(text: &str) -> String {
  text
    .split('\n')
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>()
    .join(" ")
}

pub(super) fn line_length(text: &str) -> usize {
  text.encode_utf16().count()
}

fn line_offsets(range: TextRange, line: usize, length: usize) -> Option<(usize, usize)> {
  if line < range.start.line || line > range.end.line {
    return None;
  }
  Some((
    if line == range.start.line {
      range.start.offset
    } else {
      0
    },
    if line == range.end.line {
      range.end.offset
    } else {
      length
    },
  ))
}

pub(super) fn selection_rects(
  result: &TextRecognitionResult,
  ranges: &[TextRange],
) -> Vec<TextRect> {
  let mut rects = Vec::new();
  for (line_index, line) in result.lines.iter().enumerate() {
    for range in ranges {
      let Some((start, end)) = line_offsets(*range, line_index, line_length(&line.text)) else {
        continue;
      };
      if start == end {
        continue;
      }
      let characters = line
        .characters
        .iter()
        .filter(|character| character.end > start && character.start < end)
        .collect::<Vec<_>>();
      if characters.is_empty() {
        let length = line_length(&line.text).max(1) as f64;
        rects.push(TextRect {
          x: line.bounds.x + line.bounds.width * start as f64 / length,
          y: line.bounds.y,
          width: line.bounds.width * (end - start) as f64 / length,
          height: line.bounds.height,
        });
      } else {
        let left = characters
          .iter()
          .map(|c| c.bounds.x)
          .fold(f64::INFINITY, f64::min);
        let top = characters
          .iter()
          .map(|c| c.bounds.y)
          .fold(f64::INFINITY, f64::min);
        let right = characters
          .iter()
          .map(|c| c.bounds.x + c.bounds.width)
          .fold(f64::NEG_INFINITY, f64::max);
        let bottom = characters
          .iter()
          .map(|c| c.bounds.y + c.bounds.height)
          .fold(f64::NEG_INFINITY, f64::max);
        rects.push(TextRect {
          x: left,
          y: top,
          width: right - left,
          height: bottom - top,
        });
      }
    }
  }
  rects
}

pub(super) fn selected_text(result: &TextRecognitionResult, ranges: &[TextRange]) -> String {
  let mut selected = Vec::new();
  for (line_index, line) in result.lines.iter().enumerate() {
    let mut offsets = ranges
      .iter()
      .filter_map(|range| line_offsets(*range, line_index, line_length(&line.text)))
      .collect::<Vec<_>>();
    offsets.sort_unstable_by_key(|offset| offset.0);
    let mut merged: Vec<(usize, usize)> = Vec::new();
    for (start, end) in offsets {
      if let Some(previous) = merged.last_mut().filter(|previous| start <= previous.1) {
        previous.1 = previous.1.max(end);
      } else {
        merged.push((start, end));
      }
    }
    let utf16 = line.text.encode_utf16().collect::<Vec<_>>();
    selected.extend(merged.into_iter().map(|(start, end)| {
      String::from_utf16_lossy(&utf16[start.min(utf16.len())..end.min(utf16.len())])
    }));
  }
  selected.join("\n")
}
