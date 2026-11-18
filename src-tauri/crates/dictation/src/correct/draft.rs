// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A dictation's text as corrected so far: the parts that are settled, as
//! heard and as corrected, and the open part after them with its last
//! correction. Once a correction comes back, every sentence of it but the
//! last is settled, as a correction reaches back only into the sentence it
//! is made in; the last stays open and is corrected again with whatever is
//! said after it.
//!
//! Only what the recogniser has settled is settled here, since words it may
//! still change would let the settled text go. Even so, carrying out a
//! command can change what was said before it (a list's points numbered
//! once there are two), so the settled text is kept in the pieces it was
//! settled in: a change lets go of the piece it falls in and those after
//! it, not of the whole dictation, which would then be corrected in one
//! request at every pause, until it no longer fits the model.

/// Open text longer than this many words is settled whole once corrected,
/// so a dictation without sentence ends is not corrected from its start at
/// every pause.
const MOST_OPEN_WORDS: usize = 60;
/// Marks that end a sentence.
const SENTENCE_ENDS: [char; 3] = ['.', '!', '?'];
/// Marks the stream may write after a word.
const PUNCTUATION: [char; 6] = ['.', '!', '?', ',', ';', ':'];

#[derive(Default)]
pub(super) struct Draft {
  /// Heard text whose correction will not change, in the pieces it was
  /// settled in, each with its correction.
  settled: Vec<(String, String)>,
  /// How much of the heard text, in bytes from its start, the recogniser
  /// will no longer change.
  recognised: usize,
  /// Counts the times the settled text changed: let go when what was heard
  /// no longer begins with it ("scratch that"), or grown by an answer. An
  /// answer asked for in an earlier round was about open text that has
  /// since moved, and is not taken.
  round: u64,
  /// The open text last corrected, and its correction.
  answered: Option<(String, String)>,
}

impl Draft {
  pub fn round(&self) -> u64 {
    self.round
  }

  /// The part of `text` after what is settled, the first `recognised` bytes
  /// of it no longer changing. Settled pieces `text` no longer begins with
  /// are let go, and the last correction is settled as far as the
  /// recogniser has.
  pub fn open<'t>(&mut self, text: &'t str, recognised: usize) -> &'t str {
    let mut at = 0;
    let kept = self
      .settled
      .iter()
      .take_while(|(heard, _)| {
        let begins = text[at..].starts_with(heard.as_str());
        if begins {
          at += heard.len();
        }
        begins
      })
      .count();
    if kept < self.settled.len() {
      self.settled.truncate(kept);
      self.answered = None;
      self.round += 1;
    }
    self.recognised = recognised;
    self.settle();
    &text[self.settled_len()..]
  }

  /// Whether `open` is the open text last corrected.
  pub fn answers(&self, open: &str) -> bool {
    self
      .answered
      .as_ref()
      .is_some_and(|(heard, _)| heard == open)
  }

  /// `text` with the corrections made so far, and whether all of it is
  /// corrected.
  pub fn corrected(&self, text: &str) -> (String, bool) {
    let mut written = String::with_capacity(text.len());
    let mut open = text;
    for (heard, piece) in &self.settled {
      let Some(rest) = open.strip_prefix(heard.as_str()) else {
        return (text.to_owned(), false);
      };
      written.push_str(piece);
      open = rest;
    }
    match &self.answered {
      Some((heard, answer)) if open.starts_with(heard.as_str()) => {
        let rest = &open[heard.len()..];
        // The stream punctuates the end of what was said once the pause
        // after it passes, where the correction made at that pause put a
        // mark of its own: the stream's is kept.
        let answer = if rest.starts_with(PUNCTUATION) {
          answer.trim_end().trim_end_matches(SENTENCE_ENDS)
        } else {
          answer.as_str()
        };
        let whole = rest.trim_start_matches(PUNCTUATION).trim().is_empty();
        written.push_str(answer);
        written.push_str(rest);
        (written, whole)
      }
      _ => {
        written.push_str(open);
        (written, open.trim().is_empty())
      }
    }
  }

  /// Takes `answer`, asked for in `round`, as the correction of `open`,
  /// settling all but its last sentence as far as the recogniser has. The
  /// same open text can be asked for again while its first answer is on its
  /// way; once that answer has settled sentences, the second is stale and
  /// would settle them twice.
  pub fn answer(&mut self, round: u64, open: &str, answer: String) {
    if round != self.round {
      return;
    }
    self.answered = Some((open.to_owned(), answer));
    self.settle();
  }

  fn settled_len(&self) -> usize {
    self.settled.iter().map(|(heard, _)| heard.len()).sum()
  }

  /// Settles the sentences of the last correction that the recogniser has
  /// settled, all but its last, or the whole of a long one without sentence
  /// ends once the recogniser has settled all of it.
  fn settle(&mut self) {
    let Some((heard, answer)) = self.answered.take() else {
      return;
    };
    let recognised = self.recognised.saturating_sub(self.settled_len());
    let heard_starts = sentence_starts(&heard);
    let answer_starts = sentence_starts(&answer);
    let ready = heard_starts
      .iter()
      .take_while(|&&start| start <= recognised)
      .count();
    if heard_starts.len() == answer_starts.len() && ready > 0 {
      let (said, written) = (heard_starts[ready - 1], answer_starts[ready - 1]);
      self
        .settled
        .push((heard[..said].to_owned(), answer[..written].to_owned()));
      self.answered = Some((heard[said..].to_owned(), answer[written..].to_owned()));
      self.round += 1;
    } else if heard_starts.len() != answer_starts.len()
      && heard.split_whitespace().count() > MOST_OPEN_WORDS
      && heard.trim_end().len() <= recognised
    {
      self.settled.push((heard, answer));
      self.round += 1;
    } else {
      self.answered = Some((heard, answer));
    }
  }
}

/// Where each sentence of `text` after its first begins: after a full
/// stop, question or exclamation mark followed by a space, or a line break.
fn sentence_starts(text: &str) -> Vec<usize> {
  let mut starts = Vec::new();
  let mut last: Option<char> = None;
  let (mut spaced, mut broken) = (false, false);
  for (at, letter) in text.char_indices() {
    if letter.is_whitespace() {
      spaced = true;
      broken |= letter == '\n';
      continue;
    }
    let ended = last.is_some_and(|last| SENTENCE_ENDS.contains(&last));
    if last.is_some() && (broken || spaced && ended) {
      starts.push(at);
    }
    last = Some(letter);
    (spaced, broken) = (false, false);
  }
  starts
}

#[cfg(test)]
mod tests {
  use super::{sentence_starts, Draft};

  #[test]
  fn all_but_the_last_sentence_settles() {
    let mut draft = Draft::default();
    let text = "At 2, actually 3. Then lunch, no";
    let open = draft.open(text, text.len()).to_owned();
    draft.answer(draft.round(), &open, "At 3. Then lunch, no".to_owned());
    assert_eq!(draft.corrected(text).0, "At 3. Then lunch, no");
    let text = "At 2, actually 3. Then lunch, no, dinner";
    assert_eq!(draft.corrected(text).0, "At 3. Then lunch, no, dinner");
    assert_eq!(draft.open(text, text.len()), "Then lunch, no, dinner");
  }

  #[test]
  fn the_stream_punctuates_after_a_correction() {
    let mut draft = Draft::default();
    draft.answer(draft.round(), "At 2, actually 3", "At 3.".to_owned());
    assert_eq!(
      draft.corrected("At 2, actually 3, then lunch").0,
      "At 3, then lunch"
    );
    assert_eq!(
      draft.corrected("At 2, actually 3."),
      ("At 3.".to_owned(), true)
    );
  }

  #[test]
  fn an_answer_asked_again_while_the_first_was_coming_is_not_taken() {
    let mut draft = Draft::default();
    let text = "At 2, actually 3. Then lunch";
    let round = draft.round();
    let open = draft.open(text, text.len()).to_owned();
    draft.answer(round, &open, "At 3. Then lunch".to_owned());
    draft.answer(round, &open, "At 3. Then lunch".to_owned());
    assert_eq!(draft.corrected(text).0, "At 3. Then lunch");
    assert_eq!(draft.open(text, text.len()), "Then lunch");
  }

  #[test]
  fn what_the_recogniser_may_still_change_stays_open() {
    let mut draft = Draft::default();
    let text = "At 2, actually 3. Then lunch";
    let open = draft.open(text, 0).to_owned();
    draft.answer(draft.round(), &open, "At 3. Then lunch".to_owned());
    assert_eq!(draft.open(text, 0), text);
    // Recognised later, it settles from the answer already in hand.
    assert_eq!(draft.open(text, text.len()), "Then lunch");
  }

  #[test]
  fn a_change_lets_go_only_of_the_pieces_from_where_it_falls() {
    let mut draft = Draft::default();
    let first = "One. Two. Three";
    let open = draft.open(first, first.len()).to_owned();
    draft.answer(draft.round(), &open, open.clone());
    let second = "One. Two. Three. Four";
    let open = draft.open(second, second.len()).to_owned();
    assert_eq!(open, "Three. Four");
    draft.answer(draft.round(), &open, open.clone());
    // What was said after "Two. " was heard again otherwise.
    let changed = "One. Two. 3. Four. Five";
    assert_eq!(draft.open(changed, changed.len()), "3. Four. Five");
  }

  #[test]
  fn sentences_start_after_their_ends() {
    assert_eq!(sentence_starts("One. Two? Three\nFour"), vec![5, 10, 16]);
    assert_eq!(sentence_starts("Version 2.5 is out."), Vec::<usize>::new());
  }
}
