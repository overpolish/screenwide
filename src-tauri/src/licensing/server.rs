// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use std::time::Duration;

use screenwide_licensing::{ActivationOutcome, ActivationRequest, ACTIVATION_URL};

/// Asks the licence server about a key. Without an answer (offline, a
/// timeout, the server failing) the outcome is `Unavailable`, which never
/// takes a tool away.
pub(super) async fn activate(license_key: &str, installation_id: &str) -> ActivationOutcome {
  let Ok(client) = client() else {
    return ActivationOutcome::Unavailable;
  };
  let request = ActivationRequest {
    license_key,
    installation_id,
  };
  // The app's reqwest is built without its JSON support, so the body is
  // written here.
  let Ok(body) = serde_json::to_vec(&request) else {
    return ActivationOutcome::Unavailable;
  };
  let sent = client
    .post(ACTIVATION_URL)
    .header(reqwest::header::CONTENT_TYPE, "application/json")
    .body(body)
    .send()
    .await;
  let Ok(response) = sent else {
    return ActivationOutcome::Unavailable;
  };
  let status = response.status().as_u16();
  match response.bytes().await {
    Ok(body) => ActivationOutcome::from_response(status, &body),
    Err(_) => ActivationOutcome::Unavailable,
  }
}

fn client() -> Result<reqwest::Client, String> {
  // reqwest is built without a TLS provider of its own; the updater and the
  // model downloads install the same one, whichever runs first.
  if rustls::crypto::CryptoProvider::get_default().is_none() {
    let _ = rustls::crypto::ring::default_provider().install_default();
  }
  reqwest::Client::builder()
    .user_agent(concat!("Screenwide/", env!("CARGO_PKG_VERSION")))
    .connect_timeout(Duration::from_secs(15))
    .timeout(Duration::from_secs(30))
    .build()
    .map_err(|error| error.to_string())
}
