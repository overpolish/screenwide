// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_speech::runner::dictation::{DictatedText, DictatedWord};

use super::{shape, WordChange};

/// A finished dictation of `said`: words a quarter of a second apart, as
/// fluent speech comes out, with `|` for a pause of a second.
fn finished(said: &str) -> DictatedText {
  let mut at = 0;
  let mut words = Vec::new();
  for word in said.split(' ') {
    if word == "|" {
      at += 1_000;
      continue;
    }
    words.push(DictatedWord {
      text: word.to_owned(),
      start_ms: at,
      end_ms: at + 80,
      settled: true,
    });
    at += 250;
  }
  DictatedText {
    heard_ms: at,
    finished: true,
    words,
    ..DictatedText::default()
  }
}

fn text(said: &str) -> String {
  shape(&finished(said)).text()
}

#[test]
fn a_command_said_on_its_own_is_carried_out() {
  assert_eq!(
    text("Let's meet Thursday. | Scratch that. | Let's meet Friday."),
    "Let's meet Friday."
  );
  assert_eq!(
    text("Thanks for the update. | New paragraph. | I'll review it."),
    "Thanks for the update.\n\nI'll review it."
  );
  assert_eq!(
    text("Hi Sarah | comma. | How are you?"),
    "Hi Sarah, How are you?"
  );
  assert_eq!(
    text("Dear Sam | next line. | Thanks for coming | dot."),
    "Dear Sam\nThanks for coming."
  );
}

#[test]
fn a_command_inside_a_sentence_is_just_words() {
  assert_eq!(
    text("I had to scratch that itch."),
    "I had to scratch that itch."
  );
  assert_eq!(
    text("Put a comma there. | Thanks."),
    "Put a comma there. Thanks."
  );
}

#[test]
fn scratch_that_keeps_the_sentences_before() {
  assert_eq!(text("One. Two three. | Scratch that."), "One.");
}

#[test]
fn words_spelled_or_said_change_the_list() {
  let spelled = shape(&finished("Hello. | Add word S C R E E N W I D E. | Bye."));
  assert_eq!(spelled.text(), "Hello. Bye.");
  assert_eq!(spelled.changes, [WordChange::Add("Screenwide".to_owned())]);

  let hyphenated = shape(&finished("| Remove word O-V-E-R-P-O-L-I-S-H. |"));
  assert_eq!(
    hyphenated.changes,
    [WordChange::Remove("Overpolish".to_owned())]
  );

  let said = shape(&finished("| Add word Screenwide. |"));
  assert_eq!(said.changes, [WordChange::Add("Screenwide".to_owned())]);
}

#[test]
fn a_trailing_command_waits_for_silence_after_it() {
  let mut heard = finished("Let's meet Thursday. | Scratch that.");
  heard.finished = false;
  let last_end = heard.words.last().map_or(0, |word| word.end_ms);
  heard.heard_ms = last_end + 200;
  assert_eq!(shape(&heard).text(), "Let's meet Thursday. Scratch that.");
  heard.heard_ms = last_end + 1_500;
  assert_eq!(shape(&heard).text(), "");
}

#[test]
fn words_still_being_heard_are_unsettled() {
  let mut heard = finished("Hello there. | New line. | Second");
  heard.finished = false;
  if let Some(word) = heard.words.last_mut() {
    word.settled = false;
  }
  let shaped = shape(&heard);
  assert_eq!(shaped.settled, "Hello there.\n");
  assert_eq!(shaped.unsettled, "Second");
}
