// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The recognisers scored, and written out as `recognition.md`.

use std::fmt::Write as _;

use super::normalize::{errors, words};
use super::Recording;

/// One recording as a recogniser heard it.
pub struct Heard {
  pub text: String,
  /// From the speaker stopping to the last of the text.
  pub finish_ms: u128,
  /// A person's own recording, corrected, when a cleanup model was given.
  pub corrected: Option<String>,
}

pub struct Recognised {
  pub name: String,
  pub heard: Vec<Heard>,
}

#[derive(Default)]
struct Tally {
  words: usize,
  errors: usize,
  exact: usize,
  recordings: usize,
  as_expected: usize,
  corrected: usize,
  finish: Vec<u128>,
}

fn tally(recordings: &[Recording], heard: &[Heard], own: bool) -> Tally {
  let mut tally = Tally::default();
  for (recording, heard) in recordings.iter().zip(heard) {
    if recording.own != own {
      continue;
    }
    let said = words(&recording.said);
    let wrote = words(&heard.text);
    let wrong = errors(&said, &wrote);
    tally.words += said.len();
    tally.errors += wrong;
    tally.exact += usize::from(wrong == 0);
    tally.recordings += 1;
    if let (Some(expect), Some(corrected)) = (&recording.expect, &heard.corrected) {
      tally.corrected += 1;
      tally.as_expected += usize::from(words(expect) == words(corrected));
    }
    tally.finish.push(heard.finish_ms);
  }
  tally.finish.sort_unstable();
  tally
}

fn median(values: &[u128]) -> u128 {
  values.get(values.len() / 2).copied().unwrap_or_default()
}

/// The scores of every recogniser on each set of recordings, then each
/// recording as each heard it.
pub fn report(recordings: &[Recording], results: &[Recognised]) -> String {
  let mut out = String::from(
    "<!-- SPDX-FileCopyrightText: 2026 overpolish -->\n<!-- SPDX-License-Identifier: GPL-3.0-or-later -->\n\n# Recognising English dictation\n\nWritten by `cargo run --example evaluate_recognition`. Word errors are the words substituted, inserted or left out against what was said, case, punctuation and the way numbers are written aside. Exact counts the recordings heard without a word wrong. Finish is from the speaker stopping to the last of the text. As expected counts a person's own recordings that came out as they should once corrected.\n",
  );
  for (own, title) in [(false, "macOS voices"), (true, "Your recordings")] {
    let tallies: Vec<(&str, Tally)> = results
      .iter()
      .map(|result| (result.name.as_str(), tally(recordings, &result.heard, own)))
      .filter(|(_, tally)| tally.recordings > 0)
      .collect();
    if tallies.is_empty() {
      continue;
    }
    let _ = write!(
      out,
      "\n## {title}\n\n| Recogniser | Word errors | Exact | Median finish ms | Slowest finish ms | As expected |\n| --- | --- | --- | --- | --- | --- |\n"
    );
    for (name, tally) in tallies {
      let rate = 100.0 * tally.errors as f64 / tally.words.max(1) as f64;
      let expected = if tally.corrected > 0 {
        format!("{}/{}", tally.as_expected, tally.corrected)
      } else {
        "".to_owned()
      };
      let _ = writeln!(
        out,
        "| {name} | {rate:.1}% ({}/{}) | {}/{} | {} | {} | {expected} |",
        tally.errors,
        tally.words,
        tally.exact,
        tally.recordings,
        median(&tally.finish),
        tally.finish.last().copied().unwrap_or_default(),
      );
    }
  }
  out.push_str("\n## Each recording\n");
  for (index, recording) in recordings.iter().enumerate() {
    let _ = write!(
      out,
      "\n### {}\n\n- Said: `{}`\n",
      recording.id, recording.said
    );
    if let Some(expect) = &recording.expect {
      let _ = writeln!(out, "- Expected: `{expect}`");
    }
    for result in results {
      let heard = &result.heard[index];
      let _ = write!(
        out,
        "- {}: `{}`",
        result.name,
        heard.text.replace('\n', "⏎")
      );
      if let Some(corrected) = &heard.corrected {
        let _ = write!(out, " → `{corrected}`");
      }
      out.push('\n');
    }
  }
  out
}
