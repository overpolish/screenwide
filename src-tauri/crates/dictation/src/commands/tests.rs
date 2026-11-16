// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use screenwide_speech::runner::dictation::{DictatedText, DictatedWord};

use super::{phrases, shape, Shaped, WordChange};

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

fn in_english(heard: &DictatedText) -> Shaped {
  shape(heard, phrases(Some("en")))
}

fn text(said: &str) -> String {
  in_english(&finished(said)).text()
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
  let spelled = in_english(&finished("Hello. | Add word S C R E E N W I D E. | Bye."));
  assert_eq!(spelled.text(), "Hello. Bye.");
  assert_eq!(spelled.changes, [WordChange::Add("Screenwide".to_owned())]);

  let hyphenated = in_english(&finished("| Remove word O-V-E-R-P-O-L-I-S-H. |"));
  assert_eq!(
    hyphenated.changes,
    [WordChange::Remove("Overpolish".to_owned())]
  );

  let said = in_english(&finished("| Add word Screenwide. |"));
  assert_eq!(said.changes, [WordChange::Add("Screenwide".to_owned())]);
}

#[test]
fn a_trailing_command_waits_for_silence_after_it() {
  let mut heard = finished("Let's meet Thursday. | Scratch that.");
  heard.finished = false;
  let last_end = heard.words.last().map_or(0, |word| word.end_ms);
  heard.heard_ms = last_end + 200;
  assert_eq!(
    in_english(&heard).text(),
    "Let's meet Thursday. Scratch that."
  );
  heard.heard_ms = last_end + 1_500;
  assert_eq!(in_english(&heard).text(), "");
}

#[test]
fn words_still_being_heard_are_unsettled() {
  let mut heard = finished("Hello there. | New line. | Second");
  heard.finished = false;
  if let Some(word) = heard.words.last_mut() {
    word.settled = false;
  }
  let shaped = in_english(&heard);
  assert_eq!(shaped.settled, "Hello there.\n");
  assert_eq!(shaped.unsettled, "Second");
}

#[test]
fn a_pause_before_a_correction_takes_a_comma() {
  assert_eq!(
    text("Coffee at free | actually at two"),
    "Coffee at free, actually at two"
  );
  assert_eq!(
    text("Coffee at three | Actually at two"),
    "Coffee at three, actually at two"
  );
  assert_eq!(text("Tell Mark | I mean Mike"), "Tell Mark, I mean Mike");
  // Said without a pause, or already punctuated, it is left as heard.
  assert_eq!(text("I actually enjoyed it"), "I actually enjoyed it");
  assert_eq!(
    text("Coffee at three, | actually at two"),
    "Coffee at three, actually at two"
  );
  // A pause before a word that only starts one is no correction.
  assert_eq!(text("I said | I would"), "I said I would");
  let french = shape(&finished("Café à trois | actually"), phrases(Some("fr")));
  assert_eq!(french.text(), "Café à trois actually");
}

#[test]
fn each_language_takes_its_own_commands() {
  let french = shape(
    &finished("Bonjour Claire | À la ligne. | Tu viens | point d'interrogation"),
    phrases(Some("fr")),
  );
  assert_eq!(french.text(), "Bonjour Claire\nTu viens?");
  // English commands are words in French.
  let english_in_french = shape(&finished("Bonjour | new line | merci"), phrases(Some("fr")));
  assert_eq!(english_in_french.text(), "Bonjour new line merci");
  let undetected = shape(&finished("Bonjour | à la ligne | merci"), phrases(None));
  assert_eq!(undetected.text(), "Bonjour à la ligne merci");
}

#[test]
fn chinese_and_japanese_run_their_words_together() {
  let japanese = shape(
    &finished("今日は晴れです | 読点 | 明日は雨です | 句点"),
    phrases(Some("ja")),
  );
  assert_eq!(japanese.text(), "今日は晴れです、明日は雨です。");
  let chinese = shape(&finished("你好。 | 谢谢 | 撤销"), phrases(Some("zh")));
  assert_eq!(chinese.text(), "你好。");
}
