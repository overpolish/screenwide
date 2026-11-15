// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Spoken commands, carried out on what was heard: "scratch that" takes the
//! last sentence away, "new line" and "new paragraph" break the text, a
//! punctuation mark said by name becomes the mark, and "add word" or "remove
//! word" followed by a word, spelled or said, changes the word list. A
//! command is one only when said on its own, with a pause before and after
//! it; inside a sentence its words are just words.

mod render;

use screenwide_speech::runner::dictation::{DictatedText, DictatedWord};

/// The silence before and after a command that sets it apart from the words
/// round it, in milliseconds. Word times are where the model wrote each
/// word, which in fluent speech can leave gaps of a quarter of a second.
const PAUSE_MS: u64 = 500;
/// How long a word spelled out letter by letter may pause between letters.
const SPELLING_PAUSE_MS: u64 = 1_200;
/// How far behind what was heard the model's words are, at most: silence
/// after the last word counts only once heard beyond this.
const LAG_MS: u64 = 700;

/// What a dictation comes to once its commands are carried out.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Shaped {
  /// The text that will no longer change, then the text after it that may.
  pub settled: String,
  pub unsettled: String,
  /// Changes to the word list said in commands that will no longer change.
  pub changes: Vec<WordChange>,
}

impl Shaped {
  pub fn text(&self) -> String {
    format!("{}{}", self.settled, self.unsettled)
  }
}

/// A change to the word list, said while dictating.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum WordChange {
  Add(String),
  Remove(String),
}

/// A command, by what it does.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Command {
  ScratchThat,
  Break(&'static str),
  Mark(char),
  AddWord,
  RemoveWord,
}

/// Each command, by its letters as said: the model writes a command's words
/// apart or run together ("add word", "AdWord"), so they are compared
/// without the spaces.
const COMMANDS: &[(&str, Command)] = &[
  ("scratchthat", Command::ScratchThat),
  ("newline", Command::Break("\n")),
  ("nextline", Command::Break("\n")),
  ("newparagraph", Command::Break("\n\n")),
  ("comma", Command::Mark(',')),
  ("fullstop", Command::Mark('.')),
  ("period", Command::Mark('.')),
  ("dot", Command::Mark('.')),
  ("questionmark", Command::Mark('?')),
  ("exclamationmark", Command::Mark('!')),
  ("exclamationpoint", Command::Mark('!')),
  ("colon", Command::Mark(':')),
  ("semicolon", Command::Mark(';')),
  ("addword", Command::AddWord),
  // "Add" said before a pause is often written "ad".
  ("adword", Command::AddWord),
  ("removeword", Command::RemoveWord),
];

/// The most words a command is said in.
const LONGEST_COMMAND: usize = 2;

/// `heard` with its commands carried out.
pub fn shape(heard: &DictatedText) -> Shaped {
  if heard.words.is_empty() {
    return Shaped {
      settled: heard.settled.trim().to_owned(),
      unsettled: heard.unsettled.clone(),
      changes: Vec::new(),
    };
  }
  let settled_words = heard.words.iter().take_while(|word| word.settled).count();
  let steps = steps(heard);
  let all = render::render(&heard.words, &steps, heard.words.len());
  let settled = render::render(&heard.words, &steps, settled_words);
  let common = common_prefix(&settled.text, &all.text);
  Shaped {
    settled: all.text[..common].to_owned(),
    unsettled: all.text[common..].to_owned(),
    changes: settled.changes,
  }
}

/// What each stretch of words comes to: words as they are, or a command
/// with what it covers.
#[derive(Clone, Debug, Eq, PartialEq)]
enum Step {
  /// The word at this place.
  Word(usize),
  /// A command over words `from..to`, with the word an add or remove names.
  Command {
    command: Command,
    from: usize,
    to: usize,
    word: Option<String>,
  },
}

fn steps(heard: &DictatedText) -> Vec<Step> {
  let words = &heard.words;
  let plain: Vec<String> = words.iter().map(|word| plain(&word.text)).collect();
  let mut steps = Vec::new();
  let mut at = 0;
  while at < words.len() {
    let command = (at == 0 || gap(&words[at - 1], &words[at]) >= PAUSE_MS)
      .then(|| matched(&plain[at..]))
      .flatten();
    let Some((command, length)) = command else {
      steps.push(Step::Word(at));
      at += 1;
      continue;
    };
    let after = at + length;
    let (to, word) = match command {
      Command::AddWord | Command::RemoveWord => {
        // The word runs on to the next pause, a letter spelled out being
        // allowed a longer one.
        let end = (after..words.len())
          .find(|&k| {
            let pause = gap(&words[k - 1], &words[k]);
            k > after && pause >= PAUSE_MS && !(is_letter(&plain[k]) && pause < SPELLING_PAUSE_MS)
          })
          .unwrap_or(words.len());
        (end, named(&words[after..end]))
      }
      _ => (after, None),
    };
    let alone = to < words.len() && gap(&words[to - 1], &words[to]) >= PAUSE_MS
      || to == words.len() && silent_after(heard, &words[to - 1]);
    let names_a_word = !matches!(command, Command::AddWord | Command::RemoveWord) || word.is_some();
    if alone && names_a_word {
      steps.push(Step::Command {
        command,
        from: at,
        to,
        word,
      });
      at = to;
    } else {
      steps.push(Step::Word(at));
      at += 1;
    }
  }
  steps
}

/// The command `words` start with, and how many words it takes.
fn matched(words: &[String]) -> Option<(Command, usize)> {
  let mut letters = String::new();
  for (taken, word) in words.iter().take(LONGEST_COMMAND).enumerate() {
    letters.push_str(word);
    if let Some((_, command)) = COMMANDS.iter().find(|(said, _)| *said == letters) {
      return Some((*command, taken + 1));
    }
  }
  None
}

fn gap(before: &DictatedWord, after: &DictatedWord) -> u64 {
  after.start_ms.saturating_sub(before.end_ms)
}

/// Whether silence has been heard after `last`, the last word so far.
fn silent_after(heard: &DictatedText, last: &DictatedWord) -> bool {
  heard.finished || heard.heard_ms.saturating_sub(last.end_ms) >= PAUSE_MS + LAG_MS
}

/// A word as compared with a command's: lower case, without punctuation.
fn plain(word: &str) -> String {
  word
    .chars()
    .filter(|letter| letter.is_alphanumeric())
    .flat_map(char::to_lowercase)
    .collect()
}

/// Whether a word, as compared, is a letter spelled out.
fn is_letter(word: &str) -> bool {
  word.chars().count() == 1
}

/// The word `words` name after "add word" or "remove word": spelled letter by
/// letter, the letters joined and capitalised as a name; said whole, as
/// written, without the punctuation round it.
fn named(words: &[DictatedWord]) -> Option<String> {
  let pieces: Vec<String> = words
    .iter()
    .flat_map(|word| word.text.split('-'))
    .map(|piece| {
      piece
        .trim_matches(|letter: char| !letter.is_alphanumeric())
        .to_owned()
    })
    .filter(|piece| !piece.is_empty())
    .collect();
  if pieces.is_empty() {
    return None;
  }
  if pieces.iter().all(|piece| piece.chars().count() == 1) {
    let letters: String = pieces.concat().to_lowercase();
    let mut chars = letters.chars();
    return chars
      .next()
      .map(|first| first.to_uppercase().chain(chars).collect());
  }
  Some(pieces.join(" "))
}

fn common_prefix(a: &str, b: &str) -> usize {
  a.char_indices()
    .zip(b.chars())
    .find(|((_, x), y)| x != y)
    .map_or_else(|| a.len().min(b.len()), |((at, _), _)| at)
}

#[cfg(test)]
mod tests;
