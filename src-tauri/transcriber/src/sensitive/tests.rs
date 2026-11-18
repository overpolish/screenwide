// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::path::PathBuf;

use screenwide_transcriber::SensitiveKind;

use super::{flat, utf16_offset, words, SensitiveFinder};

#[test]
fn words_are_cut_as_gliner_cuts_them() {
  let text = "Mail dana.scully@fbi-records.com, now!";
  let cut: Vec<&str> = words(text).iter().map(|&(a, b)| &text[a..b]).collect();
  assert_eq!(
    cut,
    [
      "Mail",
      "dana",
      ".",
      "scully",
      "@",
      "fbi-records",
      ".",
      "com",
      ",",
      "now",
      "!"
    ]
  );
}

#[test]
fn overlapping_spans_keep_the_surest() {
  let kept = flat(vec![(0, 1, 0, 0.6), (1, 2, 1, 0.9), (3, 3, 2, 0.7)]);
  assert_eq!(kept, vec![(1, 2, 1), (3, 3, 2)]);
}

#[test]
fn offsets_count_utf16_units() {
  let text = "😀 Zoë";
  assert_eq!(utf16_offset(text, text.find('Z').unwrap_or(0)), 3);
}

/// Meeting-notes paragraphs as the notes write them, with what should be
/// found in each: the set the model was chosen on.
const CASES: &[(&str, &[(&str, SensitiveKind)])] = &[
  (
    "Thanks. The build is ready and it passed every test overnight. We sent it to the store.",
    &[],
  ),
  (
    "Thanks for joining. Let's go through the launch plan for the beta.",
    &[],
  ),
  (
    "I spoke to Priya Shah at Northwind Traders yesterday and she's happy with the quote.",
    &[
      ("Priya Shah", SensitiveKind::Name),
      ("Northwind Traders", SensitiveKind::Company),
    ],
  ),
  (
    "Can you send the contract to dana.scully@fbi-records.com before Friday?",
    &[("dana.scully@fbi-records.com", SensitiveKind::Email)],
  ),
  (
    "My number is 07700 900123 if anything comes up over the weekend.",
    &[("07700 900123", SensitiveKind::Phone)],
  ),
  (
    "The invoice goes to 221B Baker Street, London NW1 6XE, attention of accounts.",
    &[("221B Baker Street, London NW1 6XE", SensitiveKind::Address)],
  ),
  (
    "We paid with the company card, the one ending 4242 4242 4242 4242.",
    &[("4242 4242 4242 4242", SensitiveKind::Card)],
  ),
  (
    "Tom from Globex said the renewal is worth about forty thousand pounds a year.",
    &[
      ("Tom", SensitiveKind::Name),
      ("Globex", SensitiveKind::Company),
    ],
  ),
  (
    "Marketing needs the final screenshots by Thursday, and the release notes a day before.",
    &[],
  ),
  (
    "You can reach me at john at gmail dot com, that's probably easiest.",
    &[("john at gmail dot com", SensitiveKind::Email)],
  ),
  (
    "So Initech is the client, and Bill Lumbergh signs off on every change.",
    &[
      ("Initech", SensitiveKind::Company),
      ("Bill Lumbergh", SensitiveKind::Name),
    ],
  ),
  (
    "The crash on older Windows machines is fixed, but I'd like another day of testing.",
    &[],
  ),
  (
    "Sarah, can you loop in Mark from legal and the team at Hooli before we sign?",
    &[
      ("Sarah", SensitiveKind::Name),
      ("Mark", SensitiveKind::Name),
      ("Hooli", SensitiveKind::Company),
    ],
  ),
  (
    "We'll keep Wednesday as the fallback and tell the beta group on Friday.",
    &[],
  ),
  (
    "Send the NDA to legal at contoso dot co dot uk and copy in Mei Lin.",
    &[
      ("legal at contoso dot co dot uk", SensitiveKind::Email),
      ("Mei Lin", SensitiveKind::Name),
    ],
  ),
];

/// The model and its tokenizer, from the folder `SCREENWIDE_MODELS` names:
/// the app's models folder, where the meeting notes download puts them.
fn model_files() -> Option<(PathBuf, PathBuf)> {
  let folder = PathBuf::from(std::env::var_os("SCREENWIDE_MODELS")?);
  let model = folder.join("gliner-pii-base-v1.0-q8.onnx");
  let tokenizer = folder.join("gliner-pii-base-v1.0-tokenizer.json");
  (model.is_file() && tokenizer.is_file()).then_some((model, tokenizer))
}

#[test]
fn finds_what_is_private_in_meeting_notes() {
  let Some((model, tokenizer)) = model_files() else {
    eprintln!("Skipped: SCREENWIDE_MODELS does not hold the sensitive information model");
    return;
  };
  let mut finder = SensitiveFinder::load(&model, &tokenizer).expect("the model loads");
  let (mut expected, mut found, mut false_finds) = (0, 0, Vec::new());
  let mut missed = Vec::new();
  for &(text, truth) in CASES {
    let spans = finder.find(text).expect("the model runs");
    let units: Vec<u16> = text.encode_utf16().collect();
    let truth: Vec<(usize, usize, SensitiveKind)> = truth
      .iter()
      .map(|&(words, kind)| {
        let start = utf16_offset(text, text.find(words).expect("the case's words"));
        (start, start + words.encode_utf16().count(), kind)
      })
      .collect();
    expected += truth.len();
    for &(a, b, kind) in &truth {
      let hit = spans.iter().find(|s| s.start < b && a < s.end);
      match hit {
        Some(span) if span.kind == kind => found += 1,
        Some(span) => missed.push(format!(
          "{} as {:?} for {kind:?}",
          String::from_utf16_lossy(&units[span.start..span.end]),
          span.kind
        )),
        None => missed.push(String::from_utf16_lossy(&units[a..b])),
      }
    }
    for span in &spans {
      if !truth
        .iter()
        .any(|&(a, b, _)| span.start < b && a < span.end)
      {
        false_finds.push(format!(
          "{} as {:?}",
          String::from_utf16_lossy(&units[span.start..span.end]),
          span.kind
        ));
      }
    }
  }
  eprintln!("found {found} of {expected}; missed: {missed:?}; false finds: {false_finds:?}");
  assert!(found + 3 >= expected, "found only {found} of {expected}");
  assert!(false_finds.len() <= 2, "false finds: {false_finds:?}");
}
