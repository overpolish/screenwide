// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use serde::{Deserialize, Serialize};
use tauri::AppHandle;
use tauri_plugin_opener::OpenerExt;
use ts_rs::TS;

use screenwide_licensing::{ActivationOutcome, Refusal, Tool, BUNDLE_CHECKOUT_URL};

use super::view::{self, Licences, PaidTool};
use super::{emit_changed, licensing, now, save, server};

/// What came of adding a key.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum AddKeyOutcome {
  /// Kept; it unlocks these tools.
  Added { tools: Vec<PaidTool> },
  /// The store does not know the key.
  InvalidKey,
  /// The store has disabled the key, as on a refund.
  Disabled,
  /// No answer from the licence server; nothing changed.
  Unavailable,
}

/// What a checkout sells: one tool, or every tool together.
#[derive(Clone, Copy, Debug, Deserialize, TS)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[ts(export)]
pub enum Checkout {
  Tool { tool: PaidTool },
  Bundle,
}

#[tauri::command]
pub fn get_licences(app: AppHandle) -> Licences {
  view::current(&app)
}

#[tauri::command]
pub async fn add_licence_key(app: AppHandle, key: String) -> AddKeyOutcome {
  let license_key = key.trim().to_owned();
  if license_key.is_empty() {
    return AddKeyOutcome::InvalidKey;
  }
  let installation_id = installation_id(&app);
  let outcome = server::activate(&license_key, &installation_id).await;
  let licensing = licensing(&app);
  let (result, saved) = {
    let Ok(mut state) = licensing.state.write() else {
      return AddKeyOutcome::Unavailable;
    };
    match outcome {
      ActivationOutcome::Granted(record) => {
        match state.add(&license_key, record, now(), &licensing.server_key) {
          Ok(tools) => (
            AddKeyOutcome::Added {
              tools: tools.into_iter().map(PaidTool::from).collect(),
            },
            Some(state.clone()),
          ),
          // A record that does not verify is no answer to go on.
          Err(_) => (AddKeyOutcome::Unavailable, None),
        }
      }
      ActivationOutcome::Refused(Refusal::InvalidKey) => (AddKeyOutcome::InvalidKey, None),
      ActivationOutcome::Refused(Refusal::Disabled) => (AddKeyOutcome::Disabled, None),
      ActivationOutcome::Unavailable => (AddKeyOutcome::Unavailable, None),
    }
  };
  if let Some(state) = saved {
    save(licensing.path.clone(), &state);
    emit_changed(&app);
  }
  result
}

#[tauri::command]
pub fn remove_licence_key(app: AppHandle, id: String) -> Licences {
  let licensing = licensing(&app);
  let saved = licensing.state.write().ok().map(|mut state| {
    state.activations.retain(|activation| activation.id() != id);
    state.clone()
  });
  if let Some(state) = saved {
    save(licensing.path.clone(), &state);
    emit_changed(&app);
  }
  view::current(&app)
}

#[tauri::command]
pub fn open_licence_checkout(app: AppHandle, checkout: Checkout) -> Result<(), String> {
  let url = match checkout {
    Checkout::Tool { tool } => Tool::from(tool).checkout_url(),
    Checkout::Bundle => BUNDLE_CHECKOUT_URL,
  };
  app
    .opener()
    .open_url(url, None::<&str>)
    .map_err(|error| error.to_string())
}

/// Checks again with the server every key that is due: not confirmed for a
/// few weeks, or whose record no longer verifies. A refusal drops the key; no
/// answer leaves it, to be asked about at the next launch.
pub(super) async fn recheck_due(app: &AppHandle) {
  let (due, installation_id) = {
    let licensing = licensing(app);
    let Ok(state) = licensing.state.read() else {
      return;
    };
    (
      state.due_for_check(now(), &licensing.server_key),
      state.installation_id.clone(),
    )
  };
  for license_key in due {
    let outcome = server::activate(&license_key, &installation_id).await;
    if outcome == ActivationOutcome::Unavailable {
      continue;
    }
    let licensing = licensing(app);
    let saved = licensing.state.write().ok().map(|mut state| {
      state.apply_check(&license_key, outcome, now(), &licensing.server_key);
      state.clone()
    });
    if let Some(state) = saved {
      save(licensing.path.clone(), &state);
      emit_changed(app);
    }
  }
}

fn installation_id(app: &AppHandle) -> String {
  licensing(app)
    .state
    .read()
    .map(|state| state.installation_id.clone())
    .unwrap_or_default()
}
