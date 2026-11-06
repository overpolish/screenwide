// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use serde::{Deserialize, Serialize};

use crate::record::SignedRecord;

/// The body the app posts to the licence server's `/activate`, both to add a
/// key and to check one again.
#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivationRequest<'a> {
  pub license_key: &'a str,
  pub installation_id: &'a str,
}

/// What the server's answer means.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ActivationOutcome {
  /// The key is good; here is its record.
  Granted(SignedRecord),
  /// The key is definitely not good.
  Refused(Refusal),
  /// No answer to go on: offline, a timeout, the server or Payhip failing,
  /// or an answer this version does not understand. Never taken as a no.
  Unavailable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
  /// Payhip does not know the key, for any of the tools' products.
  InvalidKey,
  /// Payhip knows the key but has disabled it, as it does on a refund.
  Disabled,
}

#[derive(Deserialize)]
struct ErrorBody {
  error: String,
}

impl ActivationOutcome {
  /// Reads the server's answer. Only the two refusals the server names
  /// count as a no: anything else, a misconfigured server's "bad request"
  /// included, must not take tools away from everyone who has paid.
  pub fn from_response(status: u16, body: &[u8]) -> ActivationOutcome {
    match status {
      200 => serde_json::from_slice::<SignedRecord>(body)
        .map(ActivationOutcome::Granted)
        .unwrap_or(ActivationOutcome::Unavailable),
      400 | 403 => match serde_json::from_slice::<ErrorBody>(body) {
        Ok(ErrorBody { error }) if error == "invalid_key" => {
          ActivationOutcome::Refused(Refusal::InvalidKey)
        }
        Ok(ErrorBody { error }) if error == "disabled" => {
          ActivationOutcome::Refused(Refusal::Disabled)
        }
        _ => ActivationOutcome::Unavailable,
      },
      _ => ActivationOutcome::Unavailable,
    }
  }
}
