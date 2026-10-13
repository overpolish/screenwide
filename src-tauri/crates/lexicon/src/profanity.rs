// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The profanity lists in `profanity/`, one file a language. They are written
//! for speech: swearing, insults and slurs, without the adult-content terms,
//! everyday words and filter-dodging spellings general blocklists carry.

use crate::fold::fold_profanity_text;
use crate::{Finding, Unit};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::sync::{Arc, LazyLock, Mutex};

const LISTS: &[(&str, &str)] = &[
  ("ar", include_str!("profanity/ar.txt")),
  ("cs", include_str!("profanity/cs.txt")),
  ("da", include_str!("profanity/da.txt")),
  ("de", include_str!("profanity/de.txt")),
  ("el", include_str!("profanity/el.txt")),
  ("en", include_str!("profanity/en.txt")),
  ("es", include_str!("profanity/es.txt")),
  ("fi", include_str!("profanity/fi.txt")),
  ("fr", include_str!("profanity/fr.txt")),
  ("he", include_str!("profanity/he.txt")),
  ("hi", include_str!("profanity/hi.txt")),
  ("hu", include_str!("profanity/hu.txt")),
  ("id", include_str!("profanity/id.txt")),
  ("it", include_str!("profanity/it.txt")),
  ("ja", include_str!("profanity/ja.txt")),
  ("ko", include_str!("profanity/ko.txt")),
  ("nl", include_str!("profanity/nl.txt")),
  ("no", include_str!("profanity/no.txt")),
  ("pl", include_str!("profanity/pl.txt")),
  ("pt", include_str!("profanity/pt.txt")),
  ("ro", include_str!("profanity/ro.txt")),
  ("ru", include_str!("profanity/ru.txt")),
  ("sv", include_str!("profanity/sv.txt")),
  ("th", include_str!("profanity/th.txt")),
  ("tr", include_str!("profanity/tr.txt")),
  ("uk", include_str!("profanity/uk.txt")),
  ("vi", include_str!("profanity/vi.txt")),
  ("zh", include_str!("profanity/zh.txt")),
];

/// Languages the transcriber reports under another code than a list's:
/// Malay reads Indonesian's, Nynorsk Norwegian's, and Cantonese the Chinese
/// characters written for it.
fn list_code(language: &str) -> &str {
  match language {
    "ms" => "id",
    "nn" => "no",
    "yue" => "zh",
    other => other,
  }
}

/// Scripts written without spaces between words, whose entries are found
/// inside the text rather than word by word.
static UNSPACED: LazyLock<Regex> = LazyLock::new(|| {
  Regex::new(r"[\p{Han}\p{Hiragana}\p{Katakana}\p{Thai}]").expect("a valid pattern")
});

#[derive(Debug, Default)]
pub struct ProfanityLexicon {
  /// Single words, folded.
  pub words: HashSet<String>,
  /// Expressions of several words, by their first word, each as its words
  /// folded.
  pub phrases: HashMap<String, Vec<Vec<String>>>,
  /// Expressions in a script written without spaces, folded.
  pub unspaced: Vec<String>,
}

impl ProfanityLexicon {
  fn read(lists: &[&str]) -> Self {
    let mut lexicon = Self::default();
    let mut unspaced = HashSet::new();
    for list in lists {
      for line in list.lines() {
        let entry = line.trim();
        if entry.is_empty() || entry.starts_with('#') {
          continue;
        }
        if UNSPACED.is_match(entry) {
          unspaced.insert(fold_profanity_text(entry));
          continue;
        }
        let parts: Vec<String> = entry
          .split_whitespace()
          .map(fold_profanity_text)
          .filter(|part| !part.is_empty())
          .collect();
        match parts.len() {
          0 => {}
          1 => {
            lexicon.words.extend(parts);
          }
          _ => lexicon
            .phrases
            .entry(parts[0].clone())
            .or_default()
            .push(parts),
        }
      }
    }
    lexicon.unspaced = unspaced.into_iter().collect();
    lexicon.unspaced.sort();
    lexicon
  }

  /// The profanity to look for in a transcript in `language`: English's list
  /// always, since English swearing turns up in every language, with the
  /// language's own where it has one. Read once each.
  pub fn for_language(language: Option<&str>) -> Arc<Self> {
    static LOADED: LazyLock<Mutex<HashMap<String, Arc<ProfanityLexicon>>>> =
      LazyLock::new(|| Mutex::new(HashMap::new()));
    let own = language.map(list_code).filter(|code| *code != "en");
    let key = own.unwrap_or("en").to_owned();
    let mut loaded = LOADED
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner());
    loaded
      .entry(key)
      .or_insert_with(|| {
        let lists: Vec<&str> = LISTS
          .iter()
          .filter(|(code, _)| *code == "en" || Some(*code) == own)
          .map(|(_, list)| *list)
          .collect();
        Arc::new(Self::read(&lists))
      })
      .clone()
  }
}

struct Token {
  text: String,
  unit: usize,
}

/// Every word the transcript still keeps, folded, each with the unit it
/// belongs to: a corrected unit can hold several, and a word corrected into
/// profanity is profanity.
fn kept_tokens(units: &[Unit]) -> Vec<Token> {
  units
    .iter()
    .enumerate()
    .filter(|(_, unit)| !unit.removed)
    .flat_map(|(index, unit)| {
      unit.text.split_whitespace().filter_map(move |word| {
        let text = fold_profanity_text(word);
        (!text.is_empty()).then_some(Token { text, unit: index })
      })
    })
    .collect()
}

struct Match {
  first: usize,
  last: usize,
  text: String,
}

/// Where each entry written without spaces occurs in the kept words run
/// together, as the token runs it covers.
fn unspaced_matches(tokens: &[Token], entries: &[String]) -> Vec<Match> {
  if entries.is_empty() {
    return Vec::new();
  }
  let mut joined = String::new();
  // The token each byte of `joined` came from.
  let mut owner = Vec::new();
  for (position, token) in tokens.iter().enumerate() {
    joined.push_str(&token.text);
    owner.extend(std::iter::repeat_n(position, token.text.len()));
  }
  let mut matches = Vec::new();
  for entry in entries {
    for (at, _) in joined.match_indices(entry.as_str()) {
      matches.push(Match {
        first: owner[at],
        last: owner[at + entry.len() - 1],
        text: entry.clone(),
      });
    }
  }
  matches
}

/// Every stretch of profanity `units` still keep, by the lists in `lexicon`,
/// leaving out what was marked "Not profanity" (`allowed`, as folded text). A
/// phrase is found across the units it spans, and a match in a script
/// written without spaces wherever it occurs in the words run together. The
/// findings are in transcript order without overlapping.
pub fn find_profanity(
  units: &[Unit],
  lexicon: &ProfanityLexicon,
  allowed: &HashSet<String>,
) -> Vec<Finding> {
  let tokens = kept_tokens(units);
  let mut found = Vec::new();
  for (position, token) in tokens.iter().enumerate() {
    if lexicon.words.contains(&token.text) {
      found.push(Match {
        first: position,
        last: position,
        text: token.text.clone(),
      });
    }
    for phrase in lexicon.phrases.get(&token.text).into_iter().flatten() {
      let end = position + phrase.len() - 1;
      if end < tokens.len()
        && phrase
          .iter()
          .enumerate()
          .all(|(offset, part)| tokens[position + offset].text == *part)
      {
        found.push(Match {
          first: position,
          last: end,
          text: phrase.join(" "),
        });
      }
    }
  }
  found.extend(unspaced_matches(&tokens, &lexicon.unspaced));
  // The longest match at each start wins, and a match inside one already
  // taken is dropped, so "son of a bitch" is one finding, not two.
  found.sort_by(|a, b| a.first.cmp(&b.first).then(b.last.cmp(&a.last)));
  let mut findings = Vec::new();
  let mut reached: Option<usize> = None;
  for found in found {
    if reached.is_some_and(|reached| found.first <= reached) || allowed.contains(&found.text) {
      continue;
    }
    reached = Some(found.last);
    findings.push(Finding {
      first: tokens[found.first].unit,
      last: tokens[found.last].unit,
      folded: found.text,
    });
  }
  findings
}
