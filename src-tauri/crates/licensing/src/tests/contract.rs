// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! What the licence server sends and expects, as it sends and expects it.

use base64::{engine::general_purpose::STANDARD, Engine};

use crate::*;

const KEY: &str = "WTKP4-66NL5-HMKQW-GFSCZ";

/// Made by WebCrypto in Node, as the server signs: the raw public key, the
/// claims' JSON and the signature in WebCrypto's r and s form.
const WEBCRYPTO_PUBLIC_KEY: &str =
  "BImtRBcmsfnL6Gmttv4R9QwbAQdnshwftXOQElO1YX7i9EdnO7udNJl5w+faoYn/7866dHSLP3M1UM5QF3WCPmk=";
const WEBCRYPTO_PAYLOAD: &str = "eyJlbnRpdGxlbWVudHMiOlsiZ2xpZGUiXSwia2V5SGFzaCI6IjE4MGM4ZmY5NmE0YTc5MjQwOWEyMTNiMTEzN2ZkMmQ0YmI0MzgyZmFkMWU2YzdhOGY4YjRkM2I1ZjhkOTRmZDMiLCJpbnN0YWxsYXRpb25JZCI6IjBmMWUyZDNjNGI1YTY5Nzg4Nzk2YTViNGMzZDJlMWYwIiwiaXNzdWVkQXQiOjE3OTAwMDAwMDB9";
const WEBCRYPTO_SIGNATURE: &str =
  "3L1kehO6BbWPGJlc1zmrcB3tJa/8VdudKhHQmqtPCRgCn34FDaH6F+vmfw9L9xbhhD9NdSteYVjpelpu8yCO+g==";
const WEBCRYPTO_INSTALLATION: &str = "0f1e2d3c4b5a69788796a5b4c3d2e1f0";

#[test]
fn a_record_signed_by_webcrypto_verifies() {
  let server_key =
    VerifyingKey::from_sec1_bytes(&STANDARD.decode(WEBCRYPTO_PUBLIC_KEY).unwrap()).unwrap();
  let record = SignedRecord {
    payload: WEBCRYPTO_PAYLOAD.into(),
    signature: WEBCRYPTO_SIGNATURE.into(),
  };
  let claims = record
    .verify(&server_key, WEBCRYPTO_INSTALLATION, KEY)
    .unwrap();
  assert_eq!(claims.entitlements, ["glide"]);
  assert_eq!(claims.issued_at, 1_790_000_000);
}

#[test]
fn the_servers_answers_read_as_granted_refused_or_unavailable() {
  let record = br#"{"payload":"cA==","signature":"cw=="}"#;
  assert!(matches!(
    ActivationOutcome::from_response(200, record),
    ActivationOutcome::Granted(_)
  ));
  assert_eq!(
    ActivationOutcome::from_response(400, br#"{"error":"invalid_key"}"#),
    ActivationOutcome::Refused(Refusal::InvalidKey)
  );
  assert_eq!(
    ActivationOutcome::from_response(403, br#"{"error":"disabled"}"#),
    ActivationOutcome::Refused(Refusal::Disabled)
  );
  for (status, body) in [
    (400, &br#"{"error":"bad request"}"#[..]),
    (500, br#"{"error":"product not configured"}"#),
    (502, br#"{"error":"upstream"}"#),
    (200, b"<html>captive portal</html>"),
    (404, b"Not found"),
  ] {
    assert_eq!(
      ActivationOutcome::from_response(status, body),
      ActivationOutcome::Unavailable
    );
  }
}

#[test]
fn the_request_uses_the_servers_field_names() {
  let body = serde_json::to_value(ActivationRequest {
    license_key: KEY,
    installation_id: "install-a",
  })
  .unwrap();
  assert_eq!(
    body,
    serde_json::json!({ "licenseKey": KEY, "installationId": "install-a" })
  );
}
