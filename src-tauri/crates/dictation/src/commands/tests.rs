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
fn numbers_starting_the_points_of_a_list_are_written_as_digits() {
  assert_eq!(
    text("One. The transcript goes into a file. | Three. It is read. | 4. It is checked."),
    "1. The transcript goes into a file.\n3. It is read.\n4. It is checked."
  );
  assert_eq!(
    text("Two steps. One, the transcript goes into a file. Two, it is read."),
    "Two steps.\n1. The transcript goes into a file.\n2. It is read."
  );
  assert_eq!(
    text("Shopping | new line. | One, eggs. | New line. | Two, flour."),
    "Shopping\n1. Eggs.\n2. Flour."
  );
  let german = shape(
    &finished("Eins, Eier kaufen. Zwei, Mehl holen."),
    phrases(Some("de")),
  );
  assert_eq!(german.text(), "1. Eier kaufen.\n2. Mehl holen.");
}

#[test]
fn end_list_closes_the_last_point_so_what_follows_starts_a_paragraph() {
  assert_eq!(
    text("The agenda. One, review the build. Two, ship it. | End list. | He said hi."),
    "The agenda.\n1. Review the build.\n2. Ship it.\n\nHe said hi."
  );
  assert_eq!(
    text(
      "The agenda. One, review the build. Two, test dictation. Three, ship it. | End list. | He \
       said open quote see you at noon close quote and left."
    ),
    "The agenda.\n1. Review the build.\n2. Test dictation.\n3. Ship it.\n\nHe said \"see you at \
     noon\" and left."
  );
  assert_eq!(
    text("Ship it. | New paragraph. | He said open quote see you at noon close quote and left."),
    "Ship it.\n\nHe said \"see you at noon\" and left."
  );
  assert_eq!(
    text("Ship it. | End list. | He said, open quote, see you at noon, close quote, and left."),
    "Ship it.\n\nHe said, \"see you at noon,\" and left."
  );
  let french = shape(
    &finished("Un, acheter du pain. Deux, appeler Sam. | Fin de liste. | Merci."),
    phrases(Some("fr")),
  );
  assert_eq!(
    french.text(),
    "1. Acheter du pain.\n2. Appeler Sam.\n\nMerci."
  );
}

#[test]
fn numbers_that_start_no_list_are_written_as_heard() {
  for said in [
    "One of them came. Three days later, two more.",
    "One, two, three, go. One, two, three, stop.",
    "How many do we need? Two. That should do.",
    "We started in 2023. 2024, we grew. 2025, we hired.",
  ] {
    assert_eq!(text(said), said);
  }
  let chinese = shape(&finished("一. 你好。 二. 谢谢。"), phrases(Some("zh")));
  assert_eq!(chinese.text(), "一.你好。二.谢谢。");
}

#[test]
fn hashtag_said_before_a_word_is_written_as_the_mark() {
  assert_eq!(text("Closes hashtag 50."), "Closes #50.");
  assert_eq!(text("Hashtag launch week is on."), "#launch week is on.");
  for said in [
    "Add a hashtag to the post.",
    "Which hashtag do we use?",
    "The hashtag is trending.",
    "Nobody said hashtag.",
  ] {
    assert_eq!(text(said), said);
  }
  let french = shape(&finished("Ferme hashtag 50."), phrases(Some("fr")));
  assert_eq!(french.text(), "Ferme hashtag 50.");
}

#[test]
fn a_quotation_said_is_written_in_quotation_marks() {
  assert_eq!(
    text("He said open quote see you at noon close quote and left."),
    "He said \"see you at noon\" and left."
  );
  assert_eq!(
    text("Let's try. | Open quote, this is the best. | I love it. | Close quote."),
    "Let's try. \"This is the best. I love it.\""
  );
  assert_eq!(
    text("Type open quote hello close quote."),
    "Type \"hello\"."
  );
  assert_eq!(
    text("Instead of saying, and I quote, it's on your clipboard. Just say moved."),
    "Instead of saying, \"it's on your clipboard.\" Just say moved."
  );
  assert_eq!(
    text("Label it in quote marks draft | for now."),
    "Label it \"draft\" for now."
  );
  // A quotation never closed ends with the words.
  assert_eq!(text("He wrote open quote not yet"), "He wrote \"not yet\"");
}

#[test]
fn words_for_quotations_with_nothing_to_quote_are_written_as_heard() {
  for said in [
    "Put it in quote marks.",
    "Close quote.",
    "The quote was open quote.",
  ] {
    assert_eq!(text(said), said);
  }
  let french = shape(
    &finished("Il a dit open quote bonjour"),
    phrases(Some("fr")),
  );
  assert_eq!(french.text(), "Il a dit open quote bonjour");
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
