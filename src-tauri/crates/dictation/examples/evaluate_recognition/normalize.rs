// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Text as it is scored: lower case, without punctuation, and every number
//! spelled out, so "3:30", "3.30" and "three thirty" score alike and a
//! model is not marked down for writing digits where another writes words.

const ONES: [&str; 20] = [
  "zero",
  "one",
  "two",
  "three",
  "four",
  "five",
  "six",
  "seven",
  "eight",
  "nine",
  "ten",
  "eleven",
  "twelve",
  "thirteen",
  "fourteen",
  "fifteen",
  "sixteen",
  "seventeen",
  "eighteen",
  "nineteen",
];
const TENS: [&str; 10] = [
  "", "", "twenty", "thirty", "forty", "fifty", "sixty", "seventy", "eighty", "ninety",
];

/// `text`'s words as scored.
pub fn words(text: &str) -> Vec<String> {
  let mut out = Vec::new();
  for token in text
    .to_lowercase()
    .replace(['-', '…', '/'], " ")
    .split_whitespace()
  {
    let token = token.trim_matches(|c: char| !c.is_alphanumeric() && !"$%'".contains(c));
    let token = token.trim_matches('\'');
    if let Some(amount) = token.strip_prefix('$') {
      if number(amount, &mut out) {
        out.push("dollars".to_owned());
        continue;
      }
    }
    if let Some(amount) = token.strip_suffix('%') {
      if number(amount, &mut out) {
        out.push("percent".to_owned());
        continue;
      }
    }
    if number(token, &mut out) {
      continue;
    }
    let word: String = token
      .chars()
      .filter(|c| c.is_alphanumeric() || *c == '\'')
      .collect();
    if !word.is_empty() {
      out.push(word);
    }
  }
  without_number_ands(out)
}

/// Spells `token` into `out` if it is a number, a time ("3:30", "6.30") or
/// an ordinal ("13th"). False, and nothing written, otherwise.
fn number(token: &str, out: &mut Vec<String>) -> bool {
  if let Some((hours, minutes)) = token.split_once([':', '.']) {
    if let (Ok(hours), Ok(minutes)) = (hours.parse::<u64>(), minutes.parse::<u64>()) {
      spell(hours, out);
      if minutes > 0 {
        spell(minutes, out);
      }
      return true;
    }
    return false;
  }
  let digits = token.replace(',', "");
  for suffix in ["st", "nd", "rd", "th"] {
    if let Some(Ok(value)) = digits.strip_suffix(suffix).map(str::parse::<u64>) {
      spell(value, out);
      if let Some(last) = out.last_mut() {
        *last = ordinal(last);
      }
      return true;
    }
  }
  match digits.parse::<u64>() {
    Ok(value) if !digits.is_empty() => {
      spell(value, out);
      true
    }
    _ => false,
  }
}

fn spell(value: u64, out: &mut Vec<String>) {
  match value {
    0..20 => out.push(ONES[value as usize].to_owned()),
    20..100 => {
      out.push(TENS[(value / 10) as usize].to_owned());
      if !value.is_multiple_of(10) {
        out.push(ONES[(value % 10) as usize].to_owned());
      }
    }
    100..1_000 => {
      spell(value / 100, out);
      out.push("hundred".to_owned());
      if !value.is_multiple_of(100) {
        spell(value % 100, out);
      }
    }
    _ => {
      spell(value / 1_000, out);
      out.push("thousand".to_owned());
      if !value.is_multiple_of(1_000) {
        spell(value % 1_000, out);
      }
    }
  }
}

fn ordinal(word: &str) -> String {
  match word {
    "one" => "first".to_owned(),
    "two" => "second".to_owned(),
    "three" => "third".to_owned(),
    "five" => "fifth".to_owned(),
    "eight" => "eighth".to_owned(),
    "nine" => "ninth".to_owned(),
    "twelve" => "twelfth".to_owned(),
    _ if word.ends_with('y') => format!("{}ieth", &word[..word.len() - 1]),
    _ => format!("{word}th"),
  }
}

fn is_number_word(word: &str) -> bool {
  ONES.contains(&word) || TENS.contains(&word) || word == "hundred" || word == "thousand"
}

/// Without the "and" said inside a number ("two hundred and forty").
fn without_number_ands(words: Vec<String>) -> Vec<String> {
  let mut out: Vec<String> = Vec::with_capacity(words.len());
  for (at, word) in words.iter().enumerate() {
    let inside = word == "and"
      && out
        .last()
        .is_some_and(|before| before == "hundred" || before == "thousand")
      && words.get(at + 1).is_some_and(|after| is_number_word(after));
    if !inside {
      out.push(word.clone());
    }
  }
  out
}

/// Word errors: the fewest words substituted, inserted or deleted to turn
/// `heard` into `said`.
pub fn errors(said: &[String], heard: &[String]) -> usize {
  let mut row: Vec<usize> = (0..=heard.len()).collect();
  for (i, said_word) in said.iter().enumerate() {
    let mut diagonal = row[0];
    row[0] = i + 1;
    for (j, heard_word) in heard.iter().enumerate() {
      let above = row[j + 1];
      row[j + 1] = (diagonal + usize::from(said_word != heard_word))
        .min(above + 1)
        .min(row[j] + 1);
      diagonal = above;
    }
  }
  row[heard.len()]
}

#[cfg(test)]
mod tests {
  use super::words;

  #[test]
  fn numbers_score_alike_however_written() {
    assert_eq!(words("Call me at 3:30."), words("call me at three thirty"));
    assert_eq!(
      words("Take me up at 6.30"),
      words("take me up at six thirty")
    );
    assert_eq!(words("on the 13th"), words("on the thirteenth"));
    assert_eq!(words("$5,000"), words("five thousand dollars"));
    assert_eq!(words("240 pounds"), words("two hundred and forty pounds"));
    assert_eq!(words("forty-five, 12%"), words("forty five twelve percent"));
  }
}
