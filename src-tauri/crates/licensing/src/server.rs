// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

use base64::{engine::general_purpose::STANDARD, Engine};
use p256::ecdsa::VerifyingKey;

/// Where the licence server (`workers/license`) takes `POST /activate`.
pub const ACTIVATION_URL: &str = "https://license.getscreenwide.com/activate";

/// The public half of the key the licence server signs records with, as the
/// raw point WebCrypto exports. Its private half is the Worker's
/// `LICENSE_SIGNING_KEY` secret and is kept nowhere else. A new key pair makes
/// every held record fail to verify, so the app activates each key again.
const PUBLIC_KEY: &str =
  "BF8nNRHUy5HEV5qEBEMlrgDoqhXCJAdNEFYsLZDzNdcLNYJaMgN1RGAHgn9W110Bebi9HUXreLcDgP6J71d+rQU=";

/// The licence server's key, which every record must be signed with.
pub fn server_key() -> VerifyingKey {
  STANDARD
    .decode(PUBLIC_KEY)
    .ok()
    .and_then(|bytes| VerifyingKey::from_sec1_bytes(&bytes).ok())
    .expect("the built-in licence server key is a P-256 point")
}
