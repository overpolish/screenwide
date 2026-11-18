// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

fn said(text: &str, spans: Vec<SensitiveSpan>) -> Vec<(String, SensitiveKind)> {
  let units: Vec<u16> = text.encode_utf16().collect();
  spans
    .into_iter()
    .map(|span| {
      (
        String::from_utf16_lossy(&units[span.start..span.end]),
        span.kind,
      )
    })
    .collect()
}

fn kinds(text: &str) -> Vec<(String, SensitiveKind)> {
  said(text, patterns(text))
}

#[test]
fn finds_what_is_written_in_a_private_form() {
  assert_eq!(
    kinds("Mail dana@fbi.gov or see https://acme.io/review. Call 07700 900123."),
    [
      ("dana@fbi.gov".to_owned(), SensitiveKind::Email),
      ("https://acme.io/review".to_owned(), SensitiveKind::Link),
      ("07700 900123".to_owned(), SensitiveKind::Phone),
    ]
  );
  assert_eq!(
    kinds("Card 4242 4242 4242 4242, sort code 20-00-00, IBAN GB82 WEST 1234 5698 7654 32."),
    [
      ("4242 4242 4242 4242".to_owned(), SensitiveKind::Card),
      ("20-00-00".to_owned(), SensitiveKind::Account),
      (
        "GB82 WEST 1234 5698 7654 32".to_owned(),
        SensitiveKind::Account
      ),
    ]
  );
}

#[test]
fn leaves_ordinary_numbers_alone() {
  assert!(patterns("It costs 1,999 pounds and ships on 12 May 2026, version 3.2.1.").is_empty());
}

#[test]
fn finds_special_kinds_by_their_words() {
  let text = "She's pregnant and off sick, so the union rep and her church know.";
  assert_eq!(
    said(text, special_words(text)),
    [
      ("pregnant".to_owned(), SensitiveKind::Health),
      ("off sick".to_owned(), SensitiveKind::Health),
      ("union rep".to_owned(), SensitiveKind::Union),
      ("church".to_owned(), SensitiveKind::Religion),
    ]
  );
}

#[test]
fn leaves_words_used_another_way_alone() {
  let text = "The button is disabled, the race condition is a headache, and the update is coming out in Asian markets.";
  assert!(special_words(text).is_empty());
}

#[test]
fn forms_win_over_the_model_and_the_model_over_special_words() {
  let span = |start, end, kind| SensitiveSpan { start, end, kind };
  let model = vec![
    span(0, 9, SensitiveKind::Name),
    span(12, 18, SensitiveKind::Account),
  ];
  let forms = vec![span(11, 20, SensitiveKind::Phone)];
  let special = vec![
    span(0, 9, SensitiveKind::Religion),
    span(22, 30, SensitiveKind::Health),
  ];
  assert_eq!(
    merged(model, forms, special),
    [
      span(0, 9, SensitiveKind::Name),
      span(11, 20, SensitiveKind::Phone),
      span(22, 30, SensitiveKind::Health),
    ]
  );
}

#[test]
fn offsets_count_utf16_units() {
  let found = patterns("😀 ana@x.io");
  assert_eq!((found[0].start, found[0].end), (3, 11));
}
