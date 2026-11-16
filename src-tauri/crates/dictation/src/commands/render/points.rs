// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Numbers said on their own to start the points of a list ("One, the
//! transcript goes into a file. Two, …"), which the speech model writes one
//! way in one point and another in the next ("One.", "4."). Each is written
//! as a digit and a full stop starting a line, so a list is numbered and
//! laid out one way. Two points at least make a list: a number said alone
//! once is more likely an answer ("How many? Two.") and stays as heard.

use screenwide_speech::runner::dictation::DictatedWord;

use super::super::{plain, Command, Phrases, Step};

/// Ends the sentence a point's number follows.
const SENTENCE_ENDS: [char; 3] = ['.', '?', '!'];
/// The marks the speech model sets a point's number apart from the point
/// with. A number with none runs on into its sentence ("Three days later").
const SET_APART: [char; 3] = ['.', ',', ':'];
/// The highest number written in digits that starts a point, so a year
/// starting a sentence ("2024, we grew") does not.
const HIGHEST: u8 = 20;
const LEAST_POINTS: usize = 2;

/// Each list point among the first `upto` of `words`: the place of its
/// number and the number. A number starts a point when it starts a
/// sentence or line, is set apart from what follows, and what follows is
/// not another number, as in a count ("One, two, three").
pub(super) fn points(
  words: &[DictatedWord],
  steps: &[Step],
  (upto, phrases): (usize, &Phrases),
) -> Vec<(usize, u8)> {
  let mut points = Vec::new();
  let mut starts_sentence = true;
  for (place, step) in steps.iter().enumerate() {
    match *step {
      Step::Word(at) if at < upto => {
        let text = &words[at].text;
        let next = match steps.get(place + 1) {
          Some(&Step::Word(next)) if next < upto => Some(&words[next].text),
          _ => None,
        };
        if let (true, Some(number), Some(next)) = (starts_sentence, point(text, phrases), next) {
          if said_number(next, phrases).is_none() {
            points.push((at, number));
          }
        }
        starts_sentence = text.ends_with(SENTENCE_ENDS);
      }
      Step::Command { command, to, .. } if to <= upto => match command {
        Command::ScratchThat | Command::Break(_) | Command::EndList => starts_sentence = true,
        Command::Mark(mark) => starts_sentence = SENTENCE_ENDS.contains(&mark),
        Command::AddWord | Command::RemoveWord => {}
      },
      // Steps run in the order of their words, so none after this one is
      // within `upto` either.
      _ => break,
    }
  }
  if points.len() < LEAST_POINTS {
    points.clear();
  }
  points
}

/// The number a word starting a point says: a number alone, followed by
/// one mark setting it apart.
fn point(text: &str, phrases: &Phrases) -> Option<u8> {
  let number = text.strip_suffix(SET_APART)?;
  number
    .chars()
    .all(|letter| letter.is_alphanumeric() || letter == '\'' || letter == '’')
    .then(|| said_number(number, phrases))
    .flatten()
}

/// The number `word` says, in digits or in `phrases`'s words.
fn said_number(word: &str, phrases: &Phrases) -> Option<u8> {
  if phrases.numbers.is_empty() {
    return None;
  }
  let said = plain(word);
  if !said.is_empty() && said.chars().all(|letter| letter.is_ascii_digit()) {
    return said
      .parse()
      .ok()
      .filter(|number| (1..=HIGHEST).contains(number));
  }
  let position = phrases
    .numbers
    .split(' ')
    .position(|forms| forms.split('|').any(|form| form == said))?;
  u8::try_from(position + 1).ok()
}
