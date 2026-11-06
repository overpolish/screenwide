// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::ecdsa::{signature::Signer, Signature, SigningKey, VerifyingKey};

use crate::record::key_hash;
use crate::*;

mod contract;

const DAY: i64 = 86_400;
const NOW: i64 = 1_790_000_000;
const KEY: &str = "WTKP4-66NL5-HMKQW-GFSCZ";

fn server() -> SigningKey {
  SigningKey::from_bytes(&[7u8; 32].into()).unwrap()
}

fn public(server: &SigningKey) -> VerifyingKey {
  *server.verifying_key()
}

/// A record as the server would sign it, from claims written out by hand,
/// the way the server serializes them.
fn record_for(
  server: &SigningKey,
  installation: &str,
  key: &str,
  entitlements: &[&str],
) -> SignedRecord {
  let payload = serde_json::json!({
    "entitlements": entitlements,
    "keyHash": key_hash(key),
    "installationId": installation,
    "issuedAt": NOW,
  })
  .to_string();
  let signature: Signature = server.sign(payload.as_bytes());
  SignedRecord {
    payload: STANDARD.encode(payload),
    signature: STANDARD.encode(signature.to_bytes()),
  }
}

fn state() -> LicenseState {
  LicenseState {
    installation_id: "install-a".into(),
    trial_started_at: None,
    activations: Vec::new(),
  }
}

#[test]
fn a_record_verifies_only_for_its_installation_key_and_server() {
  let server = server();
  let record = record_for(&server, "install-a", KEY, &["ruler"]);
  let claims = record.verify(&public(&server), "install-a", KEY).unwrap();
  assert_eq!(claims.entitlements, ["ruler"]);
  // A pasted key's surrounding spaces do not make it another key.
  assert!(record
    .verify(&public(&server), "install-a", &format!(" {KEY}\n"))
    .is_ok());

  assert_eq!(
    record.verify(&public(&server), "install-b", KEY),
    Err(RecordError::OtherInstallation)
  );
  assert_eq!(
    record.verify(&public(&server), "install-a", "ANOTHER-KEY"),
    Err(RecordError::OtherKey)
  );
  let stranger = SigningKey::from_bytes(&[9u8; 32].into()).unwrap();
  assert_eq!(
    record.verify(&public(&stranger), "install-a", KEY),
    Err(RecordError::BadSignature)
  );
}

#[test]
fn a_record_with_its_claims_changed_fails() {
  let server = server();
  let mut record = record_for(&server, "install-a", KEY, &["ruler"]);
  let forged = String::from_utf8(STANDARD.decode(&record.payload).unwrap())
    .unwrap()
    .replace("[\"ruler\"]", "[\"ruler\",\"glide\",\"references\"]");
  record.payload = STANDARD.encode(forged);
  assert_eq!(
    record.verify(&public(&server), "install-a", KEY),
    Err(RecordError::BadSignature)
  );
}

#[test]
fn a_high_s_signature_verifies_as_its_low_s_twin() {
  let server = server();
  let mut record = record_for(&server, "install-a", KEY, &["ruler"]);
  let signature = Signature::from_slice(&STANDARD.decode(&record.signature).unwrap()).unwrap();
  let (r, s) = signature.split_scalars();
  let high = Signature::from_scalars(r, -*s).unwrap();
  record.signature = STANDARD.encode(high.to_bytes());
  assert!(record.verify(&public(&server), "install-a", KEY).is_ok());
}

#[test]
fn the_trial_starts_at_first_use_and_runs_for_trial_days() {
  let key = public(&server());
  let mut state = state();
  assert_eq!(state.access(Tool::Ruler, NOW, &key), Access::TrialUnused);
  assert_eq!(
    state.begin_use(Tool::Ruler, NOW, &key),
    Access::Trial {
      ends_at: NOW + TRIAL_DAYS * DAY
    }
  );
  // One trial for all the tools, counted from the first.
  assert_eq!(
    state.begin_use(Tool::Glide, NOW + 10 * DAY, &key),
    Access::Trial {
      ends_at: NOW + TRIAL_DAYS * DAY
    }
  );
  assert!(state
    .access(Tool::Glide, NOW + TRIAL_DAYS * DAY - 1, &key)
    .allows_use());
  assert_eq!(
    state.access(Tool::Glide, NOW + TRIAL_DAYS * DAY, &key),
    Access::Expired
  );
  assert!(!state
    .begin_use(Tool::Glide, NOW + 40 * DAY, &key)
    .allows_use());
}

#[test]
fn a_key_unlocks_only_its_tools_and_leaves_the_trial_for_the_rest() {
  let server = server();
  let key = public(&server);
  let mut state = state();
  let record = record_for(
    &server,
    "install-a",
    KEY,
    &["ruler", "a-tool-from-the-future"],
  );
  assert_eq!(state.add(KEY, record, NOW, &key), Ok(vec![Tool::Ruler]));

  // Using the licensed tool does not spend the trial.
  assert_eq!(state.begin_use(Tool::Ruler, NOW, &key), Access::Licensed);
  assert_eq!(state.trial_started_at, None);
  assert_eq!(state.access(Tool::Glide, NOW, &key), Access::TrialUnused);

  // Long after the trial, the licensed tool still opens and the others do not.
  state.begin_use(Tool::Glide, NOW, &key);
  assert_eq!(
    state.access(Tool::Ruler, NOW + 400 * DAY, &key),
    Access::Licensed
  );
  assert_eq!(
    state.access(Tool::Glide, NOW + 400 * DAY, &key),
    Access::Expired
  );
}

#[test]
fn a_bundle_buyer_holds_a_key_per_tool() {
  let server = server();
  let key = public(&server);
  let mut state = state();
  for (license, tool) in [
    ("KEY-RULER", "ruler"),
    ("KEY-GLIDE", "glide"),
    ("KEY-REF", "references"),
  ] {
    state
      .add(
        license,
        record_for(&server, "install-a", license, &[tool]),
        NOW,
        &key,
      )
      .unwrap();
  }
  assert_eq!(state.unlocked(&key).len(), 3);

  // Adding a key again replaces it rather than holding it twice.
  state
    .add(
      "KEY-GLIDE",
      record_for(&server, "install-a", "KEY-GLIDE", &["glide"]),
      NOW,
      &key,
    )
    .unwrap();
  assert_eq!(state.activations.len(), 3);

  state.forget("KEY-GLIDE");
  assert_eq!(state.access(Tool::Glide, NOW, &key), Access::TrialUnused);
  assert_eq!(state.access(Tool::Ruler, NOW, &key), Access::Licensed);
}

#[test]
fn a_record_for_another_installation_is_not_kept() {
  let server = server();
  let mut state = state();
  let record = record_for(&server, "install-b", KEY, &["ruler"]);
  assert_eq!(
    state.add(KEY, record, NOW, &public(&server)),
    Err(RecordError::OtherInstallation)
  );
  assert!(state.activations.is_empty());
}

#[test]
fn keys_come_due_after_three_weeks_or_when_their_record_stops_verifying() {
  let server = server();
  let key = public(&server);
  let mut state = state();
  state
    .add(
      KEY,
      record_for(&server, "install-a", KEY, &["ruler"]),
      NOW,
      &key,
    )
    .unwrap();
  assert!(state
    .due_for_check(NOW + RECHECK_DAYS * DAY - 1, &key)
    .is_empty());
  assert_eq!(state.due_for_check(NOW + RECHECK_DAYS * DAY, &key), [KEY]);

  // A new server key: every held record is asked about at once.
  let rotated = public(&SigningKey::from_bytes(&[11u8; 32].into()).unwrap());
  assert_eq!(state.due_for_check(NOW, &rotated), [KEY]);
}

#[test]
fn only_a_definite_refusal_takes_a_key_away() {
  let server = server();
  let key = public(&server);
  let mut state = state();
  state
    .add(
      KEY,
      record_for(&server, "install-a", KEY, &["ruler"]),
      NOW,
      &key,
    )
    .unwrap();
  let later = NOW + 30 * DAY;

  state.apply_check(KEY, ActivationOutcome::Unavailable, later, &key);
  assert_eq!(state.access(Tool::Ruler, later, &key), Access::Licensed);
  assert_eq!(state.due_for_check(later, &key), [KEY]);

  // A record that does not verify is ignored, not taken as a no.
  let wrong = record_for(&server, "install-b", KEY, &["ruler"]);
  state.apply_check(KEY, ActivationOutcome::Granted(wrong), later, &key);
  assert_eq!(state.access(Tool::Ruler, later, &key), Access::Licensed);

  let fresh = record_for(&server, "install-a", KEY, &["ruler"]);
  state.apply_check(KEY, ActivationOutcome::Granted(fresh), later, &key);
  assert!(state.due_for_check(later, &key).is_empty());

  state.apply_check(
    KEY,
    ActivationOutcome::Refused(Refusal::Disabled),
    later,
    &key,
  );
  assert!(state.activations.is_empty());
}

#[test]
fn a_new_state_has_a_random_installation_id_and_round_trips() {
  let first = LicenseState::new().unwrap();
  assert_eq!(first.installation_id.len(), 32);
  assert_ne!(
    first.installation_id,
    LicenseState::new().unwrap().installation_id
  );
  let saved = serde_json::to_string(&first).unwrap();
  assert_eq!(serde_json::from_str::<LicenseState>(&saved).unwrap(), first);
}
