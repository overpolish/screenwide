// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use sha2::{Digest, Sha256};

use super::catalogue::MODELS;
use super::download::hex;
use super::language::is_valid;

#[test]
fn a_checksum_is_compared_as_the_catalogue_writes_it() {
  // The digest of "abc", from FIPS 180-2.
  assert_eq!(
    hex(&Sha256::digest(b"abc")),
    "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
  );
  for model in MODELS {
    assert_eq!(model.sha256.len(), 64, "{}", model.id);
    assert_eq!(model.sha256, model.sha256.to_lowercase(), "{}", model.id);
  }
}

#[test]
fn only_known_language_settings_are_kept() {
  assert!(is_valid("system"));
  assert!(is_valid("auto"));
  assert!(is_valid("en"));
  assert!(is_valid("haw"));
  assert!(!is_valid("EN"));
  assert!(!is_valid("en-GB"));
  assert!(!is_valid(""));
  assert!(!is_valid("../x"));
}
