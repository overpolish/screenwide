// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A dictation's text as corrected so far: the part that is settled, as
//! heard and as corrected, and the open part after it with its last
//! correction. Once a correction comes back, every sentence of it but the
//! last is settled, as a correction reaches back only into the sentence it
//! is made in; the last stays open and is corrected again with whatever is
//! said after it.

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
  /// Heard text whose correction will not change, and that correction.
  settled_heard: String,
  settled_text: String,
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

  /// The part of `text` after what is settled, letting the settled text go
  /// when `text` no longer begins with it.
  pub fn open<'t>(&mut self, text: &'t str) -> &'t str {
    if !text.starts_with(&self.settled_heard) {
      self.settled_heard.clear();
      self.settled_text.clear();
      self.answered = None;
      self.round += 1;
    }
    &text[self.settled_heard.len()..]
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
    let Some(open) = text.strip_prefix(self.settled_heard.as_str()) else {
      return (text.to_owned(), false);
    };
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
        (format!("{}{answer}{rest}", self.settled_text), whole)
      }
      _ => (
        format!("{}{open}", self.settled_text),
        open.trim().is_empty(),
      ),
    }
  }

  /// Takes `answer`, asked for in `round`, as the correction of `open`,
  /// settling all but its last sentence. The same open text can be asked
  /// for again while its first answer is on its way; once that answer has
  /// settled sentences, the second is stale and would settle them twice.
  pub fn answer(&mut self, round: u64, open: &str, answer: String) {
    if round != self.round {
      return;
    }
    let heard_starts = sentence_starts(open);
    let answer_starts = sentence_starts(&answer);
    if let (Some(&heard), Some(&written), true) = (
      heard_starts.last(),
      answer_starts.last(),
      heard_starts.len() == answer_starts.len(),
    ) {
      self.settled_heard.push_str(&open[..heard]);
      self.settled_text.push_str(&answer[..written]);
      self.answered = Some((open[heard..].to_owned(), answer[written..].to_owned()));
      self.round += 1;
    } else if open.split_whitespace().count() > MOST_OPEN_WORDS {
      self.settled_heard.push_str(open);
      self.settled_text.push_str(&answer);
      self.answered = None;
      self.round += 1;
    } else {
      self.answered = Some((open.to_owned(), answer));
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
    let open = draft.open("At 2, actually 3. Then lunch, no").to_owned();
    draft.answer(draft.round(), &open, "At 3. Then lunch, no".to_owned());
    assert_eq!(draft.settled_text, "At 3. ");
    assert_eq!(
      draft
        .corrected("At 2, actually 3. Then lunch, no, dinner")
        .0,
      "At 3. Then lunch, no, dinner"
    );
    assert_eq!(
      draft.open("At 2, actually 3. Then lunch, no, dinner"),
      "Then lunch, no, dinner"
    );
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
    let open = draft.open(text).to_owned();
    draft.answer(round, &open, "At 3. Then lunch".to_owned());
    draft.answer(round, &open, "At 3. Then lunch".to_owned());
    assert_eq!(draft.corrected(text).0, "At 3. Then lunch");
    assert_eq!(draft.open(text), "Then lunch");
  }

  #[test]
  fn sentences_start_after_their_ends() {
    assert_eq!(sentence_starts("One. Two? Three\nFour"), vec![5, 10, 16]);
    assert_eq!(sentence_starts("Version 2.5 is out."), Vec::<usize>::new());
  }
}
