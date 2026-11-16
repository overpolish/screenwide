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

use super::Command::{self, AddWord, Break, Mark, RemoveWord, ScratchThat};

/// A language's commands, each as said, and how its words are written.
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
}

/// What a spoken command does, as Settings lists them.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Does {
  ScratchThat,
  NewLine,
  NewParagraph,
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
      Mark(mark) => (said, Does::Punctuation, Some(mark)),
      AddWord => (said, Does::AddWord, None),
      RemoveWord => (said, Does::RemoveWord, None),
    })
  }
}

const NEW_LINE: Command = Break("\n");
const NEW_PARAGRAPH: Command = Break("\n\n");

/// No commands, words written apart: a language without commands, or one
/// still to be detected.
const NONE: Phrases = Phrases {
  said: &[],
  spaced: true,
  corrections: &[],
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
};
