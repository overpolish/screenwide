// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Quotations said into a sentence: "he said open quote hello close quote"
//! as `he said "hello"`. "Open quote" runs to "close quote"; "and I quote"
//! runs to its close too, or else to the next pause or the end of its
//! sentence, which is where a quotation said that way ends. What is said for
//! the marks is left out, and a mark said after the close ("close quote.")
//! stays after it. A quotation still open when the words run out is closed
//! there, so the marks always come in pairs.

use screenwide_speech::runner::dictation::DictatedWord;

use super::super::{gap, plain, Phrases, Step, PAUSE_MS};

/// Ends the sentence a quotation said with "and I quote" closes at.
const SENTENCE_ENDS: [char; 3] = ['.', '?', '!'];

/// The quotations among the first `upto` of `words`.
#[derive(Default)]
pub(super) struct Quoted {
  /// The places of the words said for the marks, which are left out.
  pub said: Vec<usize>,
  /// The places of the words a quotation opens before.
  pub opens: Vec<usize>,
  /// The places of the words a quotation closes after, each with the marks
  /// said after its close.
  pub closes: Vec<(usize, String)>,
}

/// An open quotation: whether it ends at a pause or a sentence's end, and
/// its last word so far.
struct Open {
  introduced: bool,
  last: Option<usize>,
}

pub(super) fn quotes(
  words: &[DictatedWord],
  steps: &[Step],
  (upto, phrases): (usize, &Phrases),
) -> Quoted {
  let mut quoted = Quoted::default();
  let Some(quotes) = &phrases.quotes else {
    return quoted;
  };
  let said = |place: usize, list: &[&str]| said_at(words, &steps[place..], upto, list);
  let mut open: Option<Open> = None;
  let mut place = 0;
  while place < steps.len() {
    match steps[place] {
      Step::Word(at) if at < upto => {
        if let Some(quotation) = open.as_mut() {
          if let Some(length) = said(place, quotes.close) {
            let said_words = || word_places(&steps[place..place + length]);
            if let (Some(last), Some(close)) = (quotation.last, said_words().last()) {
              // "Tool. Close quote." ends the quotation with the full stop
              // it already holds, not a second one after it.
              let ended = !words[last].text.ends_with(char::is_alphanumeric);
              let after = if ended {
                ""
              } else {
                after_word(&words[close].text)
              };
              quoted.closes.push((last, after.to_owned()));
            }
            quoted.said.extend(said_words());
            open = None;
            place += length;
            continue;
          }
          quotation.last = Some(at);
          let next = match steps.get(place + 1) {
            Some(&Step::Word(next)) if next < upto => Some(next),
            _ => None,
          };
          let ends = words[at].text.ends_with(SENTENCE_ENDS)
            || next.is_some_and(|next| gap(&words[at], &words[next]) >= PAUSE_MS);
          if quotation.introduced && ends {
            quoted.closes.push((at, String::new()));
            open = None;
          }
          place += 1;
          continue;
        }
        let opened = [(quotes.open, false), (quotes.introduce, true)]
          .into_iter()
          .find_map(|(list, introduced)| Some((said(place, list)?, introduced)));
        let first = opened.and_then(|(length, _)| match steps.get(place + length) {
          Some(&Step::Word(first)) if first < upto => Some(first),
          _ => None,
        });
        if let (Some((length, introduced)), Some(first)) = (opened, first) {
          quoted
            .said
            .extend(word_places(&steps[place..place + length]));
          quoted.opens.push(first);
          open = Some(Open {
            introduced,
            last: None,
          });
          place += length;
        } else {
          place += 1;
        }
      }
      // A command between ends the quotation before it.
      Step::Command { to, .. } if to <= upto => {
        if let Some(last) = open.take().and_then(|quotation| quotation.last) {
          quoted.closes.push((last, String::new()));
        }
        place += 1;
      }
      // Steps run in the order of their words, so none after this one is
      // within `upto` either.
      _ => break,
    }
  }
  if let Some(last) = open.and_then(|quotation| quotation.last) {
    quoted.closes.push((last, String::new()));
  }
  quoted
}

/// How many steps from the first of `steps` say one of `list`, each a word
/// within `upto`.
fn said_at(words: &[DictatedWord], steps: &[Step], upto: usize, list: &[&str]) -> Option<usize> {
  list.iter().find_map(|phrase| {
    let length = phrase.split(' ').count();
    let matches = phrase.split(' ').zip(steps).all(
      |(part, step)| matches!(*step, Step::Word(at) if at < upto && plain(&words[at].text) == part),
    );
    (matches && steps.len() >= length).then_some(length)
  })
}

/// The places of the words among `steps`.
fn word_places(steps: &[Step]) -> impl Iterator<Item = usize> + '_ {
  steps.iter().filter_map(|step| match *step {
    Step::Word(at) => Some(at),
    Step::Command { .. } => None,
  })
}

/// The marks written after a word's last letter ("quote." leaves ".").
fn after_word(text: &str) -> &str {
  &text[text
    .trim_end_matches(|letter: char| !letter.is_alphanumeric())
    .len()..]
}
