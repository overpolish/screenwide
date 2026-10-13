// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use std::collections::{HashMap, HashSet};

fn word(text: &str, start_ms: f64) -> Unit<'_> {
  Unit {
    text,
    start_ms,
    end_ms: start_ms + 300.0,
    removed: false,
  }
}

fn finding(folded: &str, first: usize, last: usize) -> Finding {
  Finding {
    folded: folded.to_owned(),
    first,
    last,
  }
}

fn lexicon() -> ProfanityLexicon {
  ProfanityLexicon {
    words: ["damn", "bitch"].map(str::to_owned).into(),
    phrases: HashMap::from([(
      "son".to_owned(),
      vec![["son", "of", "a", "bitch"].map(str::to_owned).to_vec()],
    )]),
    unspaced: vec!["笨蛋".to_owned()],
  }
}

#[test]
fn finds_a_filler_however_it_is_written_out() {
  let units = [
    word("Well,", 0.0),
    word("Ummm,", 500.0),
    word("uh", 1_000.0),
  ];
  assert_eq!(
    find_fillers(&units, Some("en")),
    [finding("um", 1, 1), finding("uh", 2, 2)]
  );
}

#[test]
fn reads_a_phrase_said_for_its_meaning_as_no_filler() {
  let units = [
    word("You", 0.0),
    word("know,", 350.0),
    word("I", 1_000.0),
    word("mean", 1_350.0),
  ];
  assert!(find_fillers(&units, Some("en")).is_empty());
}

#[test]
fn finds_the_languages_own_sounds_as_well_as_englishs() {
  let units = [word("Äh,", 0.0), word("um", 500.0)];
  let folded = |language| -> Vec<String> {
    find_fillers(&units, Some(language))
      .into_iter()
      .map(|filler| filler.folded)
      .collect()
  };
  assert_eq!(folded("de"), ["äh", "um"]);
  assert_eq!(folded("en"), ["um"]);
}

#[test]
fn finds_a_word_started_over_only_when_said_again_quickly() {
  let quick = [word("the", 0.0), word("the", 400.0), word("tool", 800.0)];
  assert_eq!(find_fillers(&quick, Some("en")), [finding("the", 0, 0)]);
  let later = [word("the", 0.0), word("the", 1_500.0)];
  assert!(find_fillers(&later, Some("en")).is_empty());
}

#[test]
fn never_finds_a_filler_already_cut() {
  let mut cut = word("um", 0.0);
  cut.removed = true;
  assert_eq!(
    find_fillers(&[cut, word("uh", 1_400.0)], Some("en")),
    [finding("uh", 1, 1)]
  );
}

#[test]
fn finds_profanity_however_it_is_cased_or_punctuated() {
  assert_eq!(
    find_profanity(
      &[word("Oh", 0.0), word("DAMN!", 0.0)],
      &lexicon(),
      &HashSet::new()
    ),
    [finding("damn", 1, 1)]
  );
}

#[test]
fn finds_a_phrase_once_across_the_words_it_spans() {
  let units = ["You", "son", "of", "a", "bitch"].map(|text| word(text, 0.0));
  assert_eq!(
    find_profanity(&units, &lexicon(), &HashSet::new()),
    [finding("son of a bitch", 1, 4)]
  );
}

#[test]
fn finds_unspaced_profanity_across_the_words_it_runs_over() {
  let units = ["你", "笨", "蛋"].map(|text| word(text, 0.0));
  assert_eq!(
    find_profanity(&units, &lexicon(), &HashSet::new()),
    [finding("笨蛋", 1, 2)]
  );
}

#[test]
fn leaves_out_what_was_allowed_and_what_was_cut() {
  let mut cut = word("bitch", 0.0);
  cut.removed = true;
  let allowed = HashSet::from(["damn".to_owned()]);
  assert!(find_profanity(&[word("damn", 0.0), cut], &lexicon(), &allowed).is_empty());
}

#[test]
fn reads_english_for_every_language_with_the_languages_own() {
  let german = ProfanityLexicon::for_language(Some("de"));
  assert!(german.words.contains("damn"));
  assert!(german.words.contains("arschloch"));
  let english = ProfanityLexicon::for_language(Some("en"));
  assert!(!english.words.contains("arschloch"));
}

#[test]
fn finds_a_list_for_every_language_that_has_one_and_reads_none_of_its_notes() {
  let size = |lexicon: &ProfanityLexicon| {
    lexicon.words.len() + lexicon.phrases.len() + lexicon.unspaced.len()
  };
  let english = size(&ProfanityLexicon::for_language(Some("en")));
  for language in ["ar", "ja", "ms", "nn", "th", "yue", "zh"] {
    let lexicon = ProfanityLexicon::for_language(Some(language));
    assert!(size(&lexicon) > english, "{language}");
    assert!(!lexicon.words.contains("spdxlicenseidentifier"));
  }
  // A language without a list finds English swearing alone.
  assert_eq!(size(&ProfanityLexicon::for_language(Some("sw"))), english);
}
