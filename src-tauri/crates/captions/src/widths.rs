// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How wide a run of a caption's words is set, and which of its words the
//! accent sets apart: chosen by hand where any word of the caption is,
//! otherwise its longest word not kept plain by hand. Each word is measured
//! once in each face, and the gap between two words once, so a line is
//! never measured afresh.

use std::ops::Range;

use crate::segment::Shown;
use crate::settings::CaptionEmphasisKind;
use crate::style::{CaptionAccent, CaptionFace};

/// Which words of a caption the accent sets apart.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Emphasis {
  None,
  /// Those chosen by hand.
  ByHand,
  /// The one at this index, picked by the rule.
  Picked(usize),
}

pub(crate) struct Widths {
  /// The words' widths in the main face, summed up to each index.
  main: Vec<f64>,
  /// The words' widths in the accent, summed likewise; none where the
  /// style sets no word in it.
  accent: Vec<f64>,
  /// The gap between two words.
  gap: f64,
  accent_mode: CaptionAccent,
  /// What each word was chosen as by hand, if anything.
  chosen: Vec<Option<CaptionEmphasisKind>>,
  /// How many characters each word has, which the rule picks by.
  characters: Vec<usize>,
  /// How many words were set apart by hand up to each index.
  emphasized_before: Vec<usize>,
  /// How much wider in the accent than in the main face the words set apart
  /// by hand are, summed up to each index.
  emphasized_wider_before: Vec<f64>,
}

/// `values` summed up to each index, from none.
fn running(values: impl Iterator<Item = f64>) -> Vec<f64> {
  let mut sums = vec![0.0];
  for value in values {
    sums.push(sums[sums.len() - 1] + value);
  }
  sums
}

impl Widths {
  /// `measure` gives a text's width set in a face, at that face's size.
  pub fn new(
    words: &[Shown],
    gap: &str,
    accent_mode: CaptionAccent,
    measure: &mut dyn FnMut(&str, CaptionFace) -> f64,
  ) -> Self {
    let main = running(
      words
        .iter()
        .map(|word| measure(&word.text, CaptionFace::Main)),
    );
    let accent = if accent_mode == CaptionAccent::None {
      Vec::new()
    } else {
      running(
        words
          .iter()
          .map(|word| measure(&word.text, CaptionFace::Accent)),
      )
    };
    let gap = if gap.is_empty() {
      0.0
    } else {
      measure(gap, CaptionFace::Main)
    };
    let chosen: Vec<_> = words.iter().map(|word| word.emphasis).collect();
    let by_hand = |index: usize| chosen[index] == Some(CaptionEmphasisKind::Emphasized);
    let mut emphasized_before = vec![0];
    for index in 0..words.len() {
      emphasized_before.push(emphasized_before[index] + usize::from(by_hand(index)));
    }
    let mut widths = Self {
      main,
      accent,
      gap,
      accent_mode,
      chosen,
      characters: words.iter().map(|word| word.text.chars().count()).collect(),
      emphasized_before,
      emphasized_wider_before: Vec::new(),
    };
    let wider_by_hand: Vec<f64> = (0..words.len())
      .map(|index| {
        if widths.chosen[index] == Some(CaptionEmphasisKind::Emphasized) {
          widths.wider(index)
        } else {
          0.0
        }
      })
      .collect();
    widths.emphasized_wider_before = running(wider_by_hand.into_iter());
    widths
  }

  fn width(sums: &[f64], words: &Range<usize>) -> f64 {
    sums[words.end] - sums[words.start]
  }

  /// How much wider word `index` is set in the accent than in the main face.
  fn wider(&self, index: usize) -> f64 {
    if self.accent.is_empty() {
      return 0.0;
    }
    Self::width(&self.accent, &(index..index + 1)) - Self::width(&self.main, &(index..index + 1))
  }

  /// Which words of the caption `words` the accent sets apart.
  pub fn emphasis(&self, words: &Range<usize>) -> Emphasis {
    if self.accent_mode != CaptionAccent::Word || words.is_empty() {
      return Emphasis::None;
    }
    if self.emphasized_before[words.end] > self.emphasized_before[words.start] {
      return Emphasis::ByHand;
    }
    // The longest word by its characters, the first of those as long.
    words
      .clone()
      .filter(|&index| self.chosen[index] != Some(CaptionEmphasisKind::Plain))
      .fold(None, |best: Option<usize>, index| match best {
        Some(best) if self.characters[best] >= self.characters[index] => Some(best),
        _ => Some(index),
      })
      .map_or(Emphasis::None, Emphasis::Picked)
  }

  /// Whether `emphasis` sets word `index` apart.
  pub fn is_emphasized(&self, emphasis: Emphasis, index: usize) -> bool {
    match emphasis {
      Emphasis::None => false,
      Emphasis::ByHand => self.chosen[index] == Some(CaptionEmphasisKind::Emphasized),
      Emphasis::Picked(picked) => picked == index,
    }
  }

  /// How wide the line `words` is set, with the words `emphasis` sets apart
  /// in the accent.
  pub fn of(&self, words: &Range<usize>, emphasis: Emphasis) -> f64 {
    if words.is_empty() {
      return 0.0;
    }
    let wider = match emphasis {
      Emphasis::None => 0.0,
      Emphasis::ByHand => Self::width(&self.emphasized_wider_before, words),
      Emphasis::Picked(index) if words.contains(&index) => self.wider(index),
      Emphasis::Picked(_) => 0.0,
    };
    Self::width(&self.main, words) + wider + self.gap * (words.len() - 1) as f64
  }
}
