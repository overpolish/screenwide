// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Turning the recognition model's answer into text. For every step across
//! the line the model gives a chance for each of its classes: a blank, then
//! the characters of its dictionary, then a space. A character is read where
//! the surest class changes to it, and runs for as long as it stays.

/// The characters the recognition model reads, in the order of its classes.
pub(super) struct Dictionary {
  entries: Vec<String>,
}

impl Dictionary {
  /// The characters listed under `character_dict` in the recognition model's
  /// configuration (`inference.yml`). It lists each as a plain scalar, or in
  /// single quotes with a quote doubled, so no YAML reader is needed.
  pub(super) fn from_config(config: &str) -> Result<Self, String> {
    let mut lines = config
      .lines()
      .skip_while(|line| line.trim_end() != "  character_dict:");
    if lines.next().is_none() {
      return Err("The text recognition configuration lists no characters".to_owned());
    }
    let entries = lines
      .map_while(|line| line.strip_prefix("  - "))
      .map(scalar)
      .collect::<Vec<_>>();
    if entries.is_empty() {
      return Err("The text recognition configuration lists no characters".to_owned());
    }
    Ok(Self { entries })
  }

  /// How many classes the model answers with: the blank, every character
  /// and the space.
  pub(super) fn classes(&self) -> usize {
    self.entries.len() + 2
  }

  fn symbol(&self, class: usize) -> &str {
    match class {
      0 => "",
      class if class <= self.entries.len() => &self.entries[class - 1],
      _ => " ",
    }
  }
}

fn scalar(raw: &str) -> String {
  let raw = raw.trim_end();
  match raw
    .strip_prefix('\'')
    .and_then(|quoted| quoted.strip_suffix('\''))
  {
    Some(quoted) => quoted.replace("''", "'"),
    None => raw.to_owned(),
  }
}

/// One thing read: its text, usually a single character, and the steps
/// across the line it was read over, the first and one past the last.
#[derive(Debug, PartialEq)]
pub(super) struct Symbol {
  pub text: String,
  pub steps: (usize, usize),
}

/// What a line reads as, and how sure the reading is: the mean chance of
/// the class each symbol was read from where it began.
#[derive(Debug, PartialEq)]
pub(super) struct Reading {
  pub symbols: Vec<Symbol>,
  pub confidence: f32,
}

/// Reads `chances`, a row of [`Dictionary::classes`] chances for each step.
pub(super) fn read(chances: &[f32], dictionary: &Dictionary) -> Reading {
  let classes = dictionary.classes();
  let mut symbols: Vec<Symbol> = Vec::new();
  let mut sureness = 0.0;
  let mut previous = 0;
  for (step, row) in chances.chunks_exact(classes).enumerate() {
    let (class, chance) =
      row
        .iter()
        .copied()
        .enumerate()
        .fold((0, f32::MIN), |best, (class, chance)| {
          if chance > best.1 {
            (class, chance)
          } else {
            best
          }
        });
    if class != 0 && class == previous {
      if let Some(symbol) = symbols.last_mut() {
        symbol.steps.1 = step + 1;
      }
    } else if class != 0 {
      symbols.push(Symbol {
        text: dictionary.symbol(class).to_owned(),
        steps: (step, step + 1),
      });
      sureness += chance;
    }
    previous = class;
  }
  let confidence = if symbols.is_empty() {
    0.0
  } else {
    sureness / symbols.len() as f32
  };
  Reading {
    symbols,
    confidence,
  }
}
