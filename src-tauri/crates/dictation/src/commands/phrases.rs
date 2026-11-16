// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The spoken commands of each language dictation takes them in, as said.
//! They follow the dictation's language, not the app's: someone may run the
//! app in English and dictate in French. A language not here takes none,
//! and everything said in it is written.

mod asian;
mod eastern;
mod northern;
mod western;

use serde::Serialize;

use super::Command::{self, AddWord, Break, EndList, Mark, RemoveWord, ScratchThat};

/// A language's commands, each as said, how its words are written, and the
/// numbers that start a list's points.
#[derive(Debug)]
pub struct Phrases {
  pub(super) said: &'static [(&'static str, Command)],
  /// Whether words are written apart: not in Chinese or Japanese.
  pub(super) spaced: bool,
  /// Words a speaker says to replace what they just said ("at 2, actually
  /// 3"), lower case. Paused before, they are set apart with a comma the
  /// speech model leaves out, which tells the cleanup model a correction
  /// follows. Only English has them, the one language corrections run in.
  pub(super) corrections: &'static [&'static str],
  /// One to ten as said, in order and as compared: lower case, without
  /// apostrophes, each form a number takes joined by `|` ("un|une"). Said on
  /// their own to start the points of a list, they are written as digits.
  /// Chinese, Japanese and Korean have none: their numbers are single
  /// syllables that are as often words of their own or parts of others.
  pub(super) numbers: &'static str,
  /// The word said before a word or number to write it as a hashtag
  /// ("closes hashtag 50" as "closes #50"). Only English has it.
  pub(super) hashtag: Option<Hashtag>,
  /// What is said round a quotation to write it in quotation marks. Only
  /// English has them.
  pub(super) quotes: Option<Quotes>,
}

/// What is said round a quotation, lower case, each said inside a sentence.
#[derive(Debug)]
pub(super) struct Quotes {
  /// Opens a quotation that runs to its close: "open quote".
  pub open: &'static [&'static str],
  /// Closes it: "close quote".
  pub close: &'static [&'static str],
  /// Opens a quotation that, unless closed first, ends at the next pause
  /// or the end of its sentence: "and I quote".
  pub introduce: &'static [&'static str],
}

/// "Hashtag" said for the mark, and the words round it that make it the
/// noun instead ("add a hashtag to the post"), lower case.
#[derive(Debug)]
pub(super) struct Hashtag {
  pub said: &'static str,
  /// Said before it: "a hashtag", "the hashtag", "which hashtag".
  pub before_it: &'static [&'static str],
  /// Said after it: "hashtag to", "hashtag is".
  pub after_it: &'static [&'static str],
}

/// What a spoken command does, as Settings lists them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Does {
  ScratchThat,
  NewLine,
  NewParagraph,
  EndList,
  Punctuation,
  AddWord,
  RemoveWord,
}

impl Phrases {
  /// Each command as said, with what it does and the mark it types if it
  /// types one, in the language's order.
  pub fn listed(&self) -> impl Iterator<Item = (&'static str, Does, Option<char>)> {
    self.said.iter().map(|&(said, command)| match command {
      ScratchThat => (said, Does::ScratchThat, None),
      Break(lines) if lines.len() > 1 => (said, Does::NewParagraph, None),
      Break(_) => (said, Does::NewLine, None),
      EndList => (said, Does::EndList, None),
      Mark(mark) => (said, Does::Punctuation, Some(mark)),
      AddWord => (said, Does::AddWord, None),
      RemoveWord => (said, Does::RemoveWord, None),
    })
  }
}

const NEW_LINE: Command = Break("\n");
const NEW_PARAGRAPH: Command = Break("\n\n");
const END_LIST: Command = EndList;

/// No commands, words written apart: a language without commands, or one
/// still to be detected.
const NONE: Phrases = Phrases {
  said: &[],
  spaced: true,
  corrections: &[],
  numbers: "",
  hashtag: None,
  quotes: None,
};

/// The commands said in `language`, a code such as `fr`.
pub fn phrases(language: Option<&str>) -> &'static Phrases {
  match language {
    Some("en") => &ENGLISH,
    Some("fr") => &western::FRENCH,
    Some("de") => &western::GERMAN,
    Some("es") => &western::SPANISH,
    Some("it") => &western::ITALIAN,
    Some("pt") => &western::PORTUGUESE,
    Some("nl") => &western::DUTCH,
    Some("sv") => &northern::SWEDISH,
    Some("da") => &northern::DANISH,
    Some("no" | "nb" | "nn") => &northern::NORWEGIAN,
    Some("fi") => &northern::FINNISH,
    Some("pl") => &eastern::POLISH,
    Some("cs") => &eastern::CZECH,
    Some("ru") => &eastern::RUSSIAN,
    Some("uk") => &eastern::UKRAINIAN,
    Some("tr") => &eastern::TURKISH,
    Some("ja") => &asian::JAPANESE,
    Some("zh") => &asian::CHINESE,
    Some("ko") => &asian::KOREAN,
    _ => &NONE,
  }
}

const ENGLISH: Phrases = Phrases {
  said: &[
    ("scratch that", ScratchThat),
    ("new line", NEW_LINE),
    ("next line", NEW_LINE),
    ("new paragraph", NEW_PARAGRAPH),
    ("end list", END_LIST),
    ("end of list", END_LIST),
    ("comma", Mark(',')),
    ("full stop", Mark('.')),
    ("period", Mark('.')),
    ("dot", Mark('.')),
    ("question mark", Mark('?')),
    ("exclamation mark", Mark('!')),
    ("exclamation point", Mark('!')),
    ("colon", Mark(':')),
    ("semicolon", Mark(';')),
    ("add word", AddWord),
    ("remove word", RemoveWord),
  ],
  spaced: true,
  corrections: &[
    "actually",
    "no",
    "sorry",
    "wait",
    "i mean",
    "or rather",
    "make that",
  ],
  numbers: "one two three four five six seven eight nine ten",
  hashtag: Some(Hashtag {
    said: "hashtag",
    before_it: &[
      "a", "an", "the", "this", "that", "each", "every", "any", "no", "one", "my", "your", "our",
      "their", "his", "her", "its", "which", "what", "new",
    ],
    after_it: &[
      "to", "for", "in", "on", "at", "of", "and", "or", "is", "was", "are", "with", "from", "that",
      "here", "there",
    ],
  }),
  quotes: Some(Quotes {
    open: &["open quote"],
    close: &["close quote", "end quote"],
    introduce: &["and i quote", "in quote marks"],
  }),
};
