// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::ecdsa::{signature::Verifier, Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// What the licence server hands back for a key it accepts. The signature
/// covers the payload's exact bytes, so both are kept as received: claims
/// serialized again, with another key order or spacing, would no longer match.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SignedRecord {
  /// Base64 of the claims' JSON.
  pub payload: String,
  /// Base64 of a P-256 ECDSA signature over the payload with SHA-256, as r
  /// and s of 32 bytes each: what WebCrypto, which signs on the server,
  /// produces.
  pub signature: String,
}

/// What the server vouches for in a record.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Claims {
  /// The tools the key's product grants, by [`crate::Tool::entitlement`].
  /// Names this version does not know are kept but unlock nothing.
  pub entitlements: Vec<String>,
  /// SHA-256 of the key, so a record cannot be passed off as another key's.
  pub key_hash: String,
  /// The installation that asked, so a record copied to another computer
  /// does not unlock it.
  pub installation_id: String,
  /// When the server signed it, in seconds since 1970.
  pub issued_at: i64,
  #[serde(default)]
  pub email: Option<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordError {
  /// Not base64, not a signature's length, or not the claims' JSON.
  Malformed,
  /// Not signed by the licence server's key.
  BadSignature,
  /// Signed for another installation.
  OtherInstallation,
  /// Signed for another key.
  OtherKey,
}

impl SignedRecord {
  pub fn verify(
    &self,
    server_key: &VerifyingKey,
    installation_id: &str,
    license_key: &str,
  ) -> Result<Claims, RecordError> {
    let payload = STANDARD
      .decode(&self.payload)
      .map_err(|_| RecordError::Malformed)?;
    let signature = STANDARD
      .decode(&self.signature)
      .ok()
      .and_then(|bytes| Signature::from_slice(&bytes).ok())
      .ok_or(RecordError::Malformed)?;
    // WebCrypto may hand out either of a signature's two valid forms; both
    // verify once put in the low-s form.
    let signature = signature.normalize_s().unwrap_or(signature);
    server_key
      .verify(&payload, &signature)
      .map_err(|_| RecordError::BadSignature)?;
    let claims: Claims = serde_json::from_slice(&payload).map_err(|_| RecordError::Malformed)?;
    if claims.installation_id != installation_id {
      return Err(RecordError::OtherInstallation);
    }
    if claims.key_hash != key_hash(license_key) {
      return Err(RecordError::OtherKey);
    }
    Ok(claims)
  }
}

/// SHA-256 of a key as lowercase hex, as the server computes it. Keys are
/// compared as typed, without the spaces a paste brings along.
pub(crate) fn key_hash(license_key: &str) -> String {
  Sha256::digest(license_key.trim().as_bytes())
    .iter()
    .map(|byte| format!("{byte:02x}"))
    .collect()
}
