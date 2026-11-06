// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use p256::ecdsa::VerifyingKey;
use serde::{Deserialize, Serialize};

use crate::{
  activation::ActivationOutcome,
  record::{key_hash, RecordError, SignedRecord},
  tool::Tool,
};

/// How long the trial runs from the first time a tool is used.
pub const TRIAL_DAYS: i64 = 14;
/// How often a held key is checked with the licence server again.
pub const RECHECK_DAYS: i64 = 21;
const DAY: i64 = 86_400;

/// A key added to this installation and the server's record for it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Activation {
  /// Kept so the key can be checked again, and activated again if the record
  /// stops verifying. It unlocks the tools for whoever holds it, as a pasted
  /// key would; there is no more to protect.
  pub license_key: String,
  pub record: SignedRecord,
  /// When the server last vouched for the key, in seconds since 1970.
  pub checked_at: i64,
}

/// Everything the gate knows, saved by the app beside its settings. Times are
/// seconds since 1970, passed in by the app.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LicenseState {
  /// Random, made once: never derived from the hardware.
  pub installation_id: String,
  #[serde(default)]
  pub trial_started_at: Option<i64>,
  #[serde(default)]
  pub activations: Vec<Activation>,
}

/// Whether a tool may be used now, and why.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Access {
  /// A key held here grants the tool.
  Licensed,
  /// No tool has been used yet; the trial starts with the first.
  TrialUnused,
  /// In the trial, which ends at this time.
  Trial { ends_at: i64 },
  /// The trial is over and no key grants the tool.
  Expired,
}

impl Access {
  pub fn allows_use(self) -> bool {
    !matches!(self, Access::Expired)
  }
}

impl LicenseState {
  /// A fresh state with a new random installation ID.
  pub fn new() -> Result<LicenseState, getrandom::Error> {
    let mut bytes = [0u8; 16];
    getrandom::fill(&mut bytes)?;
    Ok(LicenseState {
      installation_id: bytes.iter().map(|byte| format!("{byte:02x}")).collect(),
      trial_started_at: None,
      activations: Vec::new(),
    })
  }

  pub fn access(&self, tool: Tool, now: i64, server_key: &VerifyingKey) -> Access {
    if self.unlocked(server_key).contains(&tool) {
      return Access::Licensed;
    }
    match self.trial_started_at {
      None => Access::TrialUnused,
      Some(started) if now < started + TRIAL_DAYS * DAY => Access::Trial {
        ends_at: started + TRIAL_DAYS * DAY,
      },
      Some(_) => Access::Expired,
    }
  }

  /// The tool is starting: starts the trial if this is the first use, and
  /// says whether it may run. A tool its key already grants leaves the
  /// trial unstarted, to be had in full for the others.
  pub fn begin_use(&mut self, tool: Tool, now: i64, server_key: &VerifyingKey) -> Access {
    if self.access(tool, now, server_key) == Access::TrialUnused {
      self.trial_started_at = Some(now);
    }
    self.access(tool, now, server_key)
  }

  /// The tools the keys held here grant, from the records that verify.
  pub fn unlocked(&self, server_key: &VerifyingKey) -> Vec<Tool> {
    let mut tools = Vec::new();
    for activation in &self.activations {
      let Ok(claims) =
        activation
          .record
          .verify(server_key, &self.installation_id, &activation.license_key)
      else {
        continue;
      };
      for tool in claims
        .entitlements
        .iter()
        .filter_map(|name| Tool::from_entitlement(name))
      {
        if !tools.contains(&tool) {
          tools.push(tool);
        }
      }
    }
    tools
  }

  /// Keeps a key the server has just granted, replacing what was held for
  /// it. Returns the tools its record grants.
  pub fn add(
    &mut self,
    license_key: &str,
    record: SignedRecord,
    now: i64,
    server_key: &VerifyingKey,
  ) -> Result<Vec<Tool>, RecordError> {
    let license_key = license_key.trim();
    let claims = record.verify(server_key, &self.installation_id, license_key)?;
    self.forget(license_key);
    self.activations.push(Activation {
      license_key: license_key.to_owned(),
      record,
      checked_at: now,
    });
    Ok(
      claims
        .entitlements
        .iter()
        .filter_map(|name| Tool::from_entitlement(name))
        .collect(),
    )
  }

  pub fn forget(&mut self, license_key: &str) {
    let hash = key_hash(license_key);
    self
      .activations
      .retain(|activation| key_hash(&activation.license_key) != hash);
  }

  /// The keys to check with the server again: those not vouched for in
  /// [`RECHECK_DAYS`], and those whose record no longer verifies (a new
  /// server key, or a state carried over from another installation).
  pub fn due_for_check(&self, now: i64, server_key: &VerifyingKey) -> Vec<String> {
    self
      .activations
      .iter()
      .filter(|activation| {
        now - activation.checked_at >= RECHECK_DAYS * DAY
          || activation
            .record
            .verify(server_key, &self.installation_id, &activation.license_key)
            .is_err()
      })
      .map(|activation| activation.license_key.clone())
      .collect()
  }

  /// Applies the server's answer to checking a held key again. Only a
  /// definite refusal drops the key; without an answer, or with a record
  /// that does not verify, the key stays as it was and is asked about again
  /// next time.
  pub fn apply_check(
    &mut self,
    license_key: &str,
    outcome: ActivationOutcome,
    now: i64,
    server_key: &VerifyingKey,
  ) {
    match outcome {
      ActivationOutcome::Granted(record) => {
        if record
          .verify(server_key, &self.installation_id, license_key.trim())
          .is_ok()
        {
          let _ = self.add(license_key, record, now, server_key);
        }
      }
      ActivationOutcome::Refused(_) => self.forget(license_key),
      ActivationOutcome::Unavailable => {}
    }
  }
}
