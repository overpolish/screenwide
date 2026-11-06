// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Manager};
use ts_rs::TS;

use screenwide_licensing::{Access, Tool, TRIAL_DAYS};

use super::LicensingState;

/// A paid tool, as the pages name it.
#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub enum PaidTool {
  Ruler,
  References,
  Glide,
  Annotate,
}

impl From<Tool> for PaidTool {
  fn from(tool: Tool) -> Self {
    match tool {
      Tool::Ruler => PaidTool::Ruler,
      Tool::References => PaidTool::References,
      Tool::Glide => PaidTool::Glide,
      Tool::Annotate => PaidTool::Annotate,
    }
  }
}

impl From<PaidTool> for Tool {
  fn from(tool: PaidTool) -> Self {
    match tool {
      PaidTool::Ruler => Tool::Ruler,
      PaidTool::References => Tool::References,
      PaidTool::Glide => Tool::Glide,
      PaidTool::Annotate => Tool::Annotate,
    }
  }
}

/// Whether a tool may be used, and why.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum ToolAccess {
  Licensed,
  /// No paid tool has been used yet; the trial starts with the first.
  TrialUnused,
  /// In the trial, which ends at `endsAt`, in seconds since 1970.
  Trial {
    #[serde(rename = "endsAt")]
    #[ts(rename = "endsAt", type = "number")]
    ends_at: i64,
  },
  Expired,
}

#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct ToolLicence {
  pub tool: PaidTool,
  pub access: ToolAccess,
}

/// A key held here, named without showing it.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct HeldKey {
  /// What to hand back to remove the key.
  pub id: String,
  /// The key's last characters, for telling keys apart.
  pub hint: String,
  pub tools: Vec<PaidTool>,
}

/// Everything the Licences pane shows.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct Licences {
  pub tools: Vec<ToolLicence>,
  pub keys: Vec<HeldKey>,
  #[ts(type = "number")]
  pub trial_days: i64,
}

pub(super) fn current(app: &AppHandle) -> Licences {
  let licensing = app.state::<LicensingState>();
  let now = super::now();
  let Ok(state) = licensing.state.read() else {
    return Licences {
      tools: Vec::new(),
      keys: Vec::new(),
      trial_days: TRIAL_DAYS,
    };
  };
  let key = &licensing.server_key;
  let tools = Tool::ALL
    .into_iter()
    .map(|tool| ToolLicence {
      tool: tool.into(),
      access: match state.access(tool, now, key) {
        Access::Licensed => ToolAccess::Licensed,
        Access::TrialUnused => ToolAccess::TrialUnused,
        Access::Trial { ends_at } => ToolAccess::Trial { ends_at },
        Access::Expired => ToolAccess::Expired,
      },
    })
    .collect();
  let keys = state
    .activations
    .iter()
    .map(|activation| HeldKey {
      id: activation.id(),
      hint: hint(&activation.license_key),
      tools: activation
        .record
        .verify(key, &state.installation_id, &activation.license_key)
        .map(|claims| {
          claims
            .entitlements
            .iter()
            .filter_map(|name| Tool::from_entitlement(name))
            .map(PaidTool::from)
            .collect()
        })
        .unwrap_or_default(),
    })
    .collect();
  Licences {
    tools,
    keys,
    trial_days: TRIAL_DAYS,
  }
}

/// The last five characters, enough to tell a buyer's keys apart.
fn hint(license_key: &str) -> String {
  let characters: Vec<char> = license_key.chars().collect();
  characters[characters.len().saturating_sub(5)..]
    .iter()
    .collect()
}
