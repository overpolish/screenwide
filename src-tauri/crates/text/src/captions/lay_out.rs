// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A caption made of the words it shows: broken into lines as its style
//! arranges them, each word marked where the accent sets it apart.

use std::ops::Range;

use crate::captions::segment::{self, Shown};
use crate::captions::widths::Widths;
use crate::captions::{max_line_width, type_size_px, Caption, CaptionFrame, CaptionWord, Measure};
use crate::style::TextStyle;

/// The caption of `shown[words]`, laid out in lines no wider than
/// `max_width`, as `widths` measures them; `gap` parts its words.
pub(crate) fn caption(
  shown: &[Shown],
  words: Range<usize>,
  style: &TextStyle,
  max_width: f64,
  widths: &Widths,
  gap: &str,
) -> Caption {
  let emphasis = widths.emphasis(&words);
  let lines: Vec<Vec<CaptionWord>> = segment::lines(words.clone(), style, max_width, widths)
    .into_iter()
    .map(|line| {
      line
        .map(|index| {
          let word = &shown[index];
          CaptionWord {
            text: word.text.clone(),
            said: word.said.clone(),
            start_ms: word.output_start_ms,
            end_ms: word.output_end_ms,
            source_start_ms: word.source_start_ms,
            source_end_ms: word.source_end_ms,
            emphasis: widths.is_emphasized(emphasis, index),
          }
        })
        .collect()
    })
    .collect();
  let text = lines
    .iter()
    .map(|line| {
      line
        .iter()
        .map(|word| word.text.as_str())
        .collect::<Vec<_>>()
        .join(gap)
    })
    .collect::<Vec<_>>()
    .join("\n");
  Caption {
    id: shown[words.start].source_start_ms.round() as u64,
    start_ms: shown[words.start].output_start_ms,
    end_ms: shown[words.end - 1].output_end_ms,
    text,
    lines,
    spaced: !gap.is_empty(),
  }
}

/// One caption of `words`, said when each says, laid out as `style` lays
/// a caption out in `frame`: what a style's preview shows, without a
/// transcript to make captions of. Words are parted by spaces.
pub fn sample_caption(
  words: &[CaptionWord],
  style: &TextStyle,
  frame: CaptionFrame,
  measure: &mut Measure,
) -> Caption {
  let style = style.normalized();
  let shown: Vec<Shown> = words
    .iter()
    .map(|word| Shown {
      text: if style.all_caps {
        word.text.to_uppercase()
      } else {
        word.text.clone()
      },
      said: word.text.clone(),
      source_start_ms: word.source_start_ms,
      source_end_ms: word.source_end_ms,
      output_start_ms: word.start_ms,
      output_end_ms: word.end_ms,
      follows: true,
      emphasis: None,
    })
    .collect();
  let size = type_size_px(&style, frame);
  let widths = Widths::new(&shown, " ", style.accent, &mut |text, face| {
    measure(text, face, size * style.scale_of(face))
  });
  let mut caption = caption(
    &shown,
    0..shown.len(),
    &style,
    max_line_width(frame, size),
    &widths,
    " ",
  );
  caption.id = words
    .first()
    .map_or(0, |word| word.source_start_ms.round() as u64);
  caption
}
