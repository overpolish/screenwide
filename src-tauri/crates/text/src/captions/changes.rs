// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What typing into a caption on the picture changed: the words it spelled
//! another way, as corrections of the transcript over the stretches those
//! words were said in, and the words it took out whole, which the captions
//! then hide while the recording keeps them. Words left as they were change
//! nothing, so their timings stay exactly as heard.

use std::ops::Range;

use super::{Caption, Correction};
use crate::emphasis::TextSpan;

/// The changes typing into a caption made: corrections of the transcript,
/// and the stretches of the words to hide, all in source milliseconds.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct CaptionChanges {
  pub corrections: Vec<Correction>,
  pub hidden: Vec<[f64; 2]>,
}

/// One word of the caption as typing leaves it: the stretch it was said in,
/// what the transcript spelled it, and what it reads now, where it is still
/// there. Words merged into one by typing share the first's unit.
struct Unit {
  span: [f64; 2],
  said: String,
  typed: Option<String>,
}

/// Which of `before`'s words and `after`'s pair up unchanged: the longest
/// run in common, in order, as pairs of their places.
fn common(before: &[&str], after: &[&str]) -> Vec<(usize, usize)> {
  let mut longest = vec![vec![0usize; after.len() + 1]; before.len() + 1];
  for i in (0..before.len()).rev() {
    for j in (0..after.len()).rev() {
      longest[i][j] = if before[i] == after[j] {
        longest[i + 1][j + 1] + 1
      } else {
        longest[i + 1][j].max(longest[i][j + 1])
      };
    }
  }
  let (mut i, mut j, mut pairs) = (0, 0, Vec::new());
  while i < before.len() && j < after.len() {
    if before[i] == after[j] {
      pairs.push((i, j));
      i += 1;
      j += 1;
    } else if longest[i + 1][j] >= longest[i][j + 1] {
      i += 1;
    } else {
      j += 1;
    }
  }
  pairs
}

/// One word of a caption's typed text: its bytes in the text, and the
/// stretch of the source the words it stands for were said in. A word typed
/// where nothing was said stands for the word it joins.
#[derive(Clone, Debug, PartialEq)]
pub struct TypedCaptionWord {
  pub bytes: Range<usize>,
  pub span: [f64; 2],
}

/// The bytes of `part`, a slice of `text`, within it.
fn bytes_in(text: &str, part: &str) -> Range<usize> {
  let start = part.as_ptr() as usize - text.as_ptr() as usize;
  start..start + part.len()
}

/// `caption`, which held its words as the transcript spells them, read
/// again as `typed`: its units as typing leaves them, and each word of
/// `typed`, by its bytes, with the unit it went to. A caption whose words
/// run on without spaces is read a line at a time, each line one word.
fn read(caption: &Caption, typed: &str) -> (Vec<Unit>, Vec<(Range<usize>, usize)>) {
  let mut units: Vec<Unit> = caption
    .lines
    .iter()
    .flatten()
    .map(|word| Unit {
      span: [word.source_start_ms, word.source_end_ms],
      said: word.said.clone(),
      typed: Some(word.said.clone()),
    })
    .collect();
  if caption.spaced {
    let words: Vec<&str> = typed.split_whitespace().collect();
    let owners = respell(&mut units, &words, " ");
    let read = words
      .iter()
      .zip(owners)
      .map(|(word, owner)| (bytes_in(typed, word), owner))
      .collect();
    return (units, read);
  }
  // Without spaces the words cannot be told apart again, so each line is
  // one word, as long as the lines are as they were.
  let mut lines: Vec<Unit> = Vec::new();
  let mut at = 0;
  for line in &caption.lines {
    let words = &units[at..at + line.len()];
    at += line.len();
    if let (Some(first), Some(last)) = (words.first(), words.last()) {
      let said: String = words.iter().map(|word| word.said.as_str()).collect();
      lines.push(Unit {
        span: [first.span[0], last.span[1]],
        typed: Some(said.clone()),
        said,
      });
    }
  }
  let typed_lines: Vec<&str> = typed.lines().map(str::trim).collect();
  let mut read = Vec::new();
  if typed_lines.len() == lines.len() {
    for (index, (line, typed_line)) in lines.iter_mut().zip(&typed_lines).enumerate() {
      line.typed = (!typed_line.is_empty()).then(|| (*typed_line).to_owned());
      if !typed_line.is_empty() {
        read.push((bytes_in(typed, typed_line), index));
      }
    }
  } else {
    let joined: String = typed.split_whitespace().collect();
    let owners = respell(&mut lines, &[&joined], "");
    let whole = typed.trim();
    if let (Some(owner), false) = (owners.first(), whole.is_empty()) {
      read.push((bytes_in(typed, whole), *owner));
    }
  }
  (lines, read)
}

/// What typing `typed` into `caption`, which held its words as the
/// transcript spells them, changed. A word spelled another way is corrected
/// over its own stretch; several typed where fewer were said are one
/// correction over those words' stretches; a word typed between two kept as
/// they were joins the one before it, or the one after at the start; a word
/// taken out whole is hidden. A caption whose words run on without spaces
/// is compared a line at a time.
pub fn caption_changes(caption: &Caption, typed: &str) -> CaptionChanges {
  let (units, _) = read(caption, typed);
  let mut changes = CaptionChanges::default();
  for unit in units {
    match unit.typed {
      None => changes.hidden.push(unit.span),
      Some(typed) if typed != unit.said => changes.corrections.push(Correction {
        start_ms: unit.span[0],
        end_ms: unit.span[1],
        text: typed,
      }),
      Some(_) => {}
    }
  }
  changes
}

/// Each word of `typed`, typed into `caption`, with the stretch of the
/// source it stands for, read as [`caption_changes`] reads them.
pub fn typed_caption_words(caption: &Caption, typed: &str) -> Vec<TypedCaptionWord> {
  let (units, read) = read(caption, typed);
  read
    .into_iter()
    .map(|(bytes, owner)| TypedCaptionWord {
      bytes,
      span: units[owner].span,
    })
    .collect()
}

/// The key words of `typed`, typed into `caption`, as `captions`, made again
/// from the transcript meanwhile, set their words apart: a typed word is one
/// where a word set apart was said within the stretch it stands for.
pub fn typed_emphasis(caption: &Caption, typed: &str, captions: &[Caption]) -> Vec<TextSpan> {
  let apart: Vec<f64> = captions
    .iter()
    .flat_map(|caption| caption.lines.iter().flatten())
    .filter(|word| word.emphasis)
    .map(|word| (word.source_start_ms + word.source_end_ms) / 2.0)
    .collect();
  typed_caption_words(caption, typed)
    .into_iter()
    .filter(|word| {
      apart
        .iter()
        .any(|middle| word.span[0] <= *middle && *middle <= word.span[1])
    })
    .map(|word| TextSpan {
      start: word.bytes.start,
      end: word.bytes.end,
    })
    .collect()
}

/// `units` read again as `typed`, a word of it each, `gap` between words
/// that come to share one unit. Answers the unit each typed word went to,
/// counted among the units left.
fn respell(units: &mut Vec<Unit>, typed: &[&str], gap: &str) -> Vec<usize> {
  let said: Vec<String> = units.iter().map(|unit| unit.said.clone()).collect();
  let said: Vec<&str> = said.iter().map(String::as_str).collect();
  let kept = common(&said, typed);
  let mut owners = vec![0; typed.len()];
  // The runs between the words kept, and after the last.
  let mut runs = Vec::new();
  let (mut from_said, mut from_typed) = (0, 0);
  for &(at_said, at_typed) in kept.iter().chain([&(said.len(), typed.len())]) {
    runs.push((from_said..at_said, from_typed..at_typed));
    if let Some(owner) = owners.get_mut(at_typed) {
      *owner = at_said;
    }
    (from_said, from_typed) = (at_said + 1, at_typed + 1);
  }
  let mut merged = Vec::new();
  for (said_run, typed_run) in runs {
    let words = &typed[typed_run.clone()];
    if said_run.is_empty() {
      if words.is_empty() {
        continue;
      }
      // Typed where nothing was said: it joins the word kept before it, or
      // the one after where it starts the caption.
      let joined = words.join(gap);
      if let Some(before) = said_run.start.checked_sub(1) {
        owners[typed_run].fill(before);
        let unit = &mut units[before];
        if let Some(text) = unit.typed.as_mut() {
          *text = format!("{text}{gap}{joined}");
        }
      } else if let Some(text) = units
        .get_mut(said_run.end)
        .and_then(|unit| unit.typed.as_mut())
      {
        owners[typed_run].fill(said_run.end);
        *text = format!("{joined}{gap}{text}");
      }
    } else if words.is_empty() {
      for unit in &mut units[said_run] {
        unit.typed = None;
      }
    } else if words.len() == said_run.len() {
      for (offset, (unit, word)) in units[said_run.clone()].iter_mut().zip(words).enumerate() {
        unit.typed = Some((*word).to_owned());
        owners[typed_run.start + offset] = said_run.start + offset;
      }
    } else {
      // Respelled as a different number of words: one correction over them
      // all, the first unit taking the others' stretch.
      owners[typed_run].fill(said_run.start);
      let last = units[said_run.end - 1].span[1];
      let first = &mut units[said_run.start];
      first.span[1] = last;
      first.said = said[said_run.clone()].join(gap);
      first.typed = Some(words.join(gap));
      merged.extend(said_run.start + 1..said_run.end);
    }
  }
  let mut index = 0;
  units.retain(|_| {
    index += 1;
    !merged.contains(&(index - 1))
  });
  // Counted again among the units left once the merged ones are gone.
  for owner in &mut owners {
    *owner -= merged.iter().filter(|gone| **gone < *owner).count();
  }
  owners
}

#[cfg(test)]
mod tests;
