// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: Elastic-2.0

//! Who may use the paid tools, listed in [`Tool`]. Each tool is free for a
//! 14-day trial that starts the first time any of them is used; after
//! that it needs a licence key bought for it. The licence server checks a key
//! with Payhip and answers with a record it signs, which this crate checks
//! offline from then on against the server's public key, so a tool opens
//! without a connection. Every few weeks the app asks the server again, so a
//! key Payhip has disabled (a refund) stops working; an unreachable server
//! never takes a tool away.
//!
//! Nothing here touches the network, the disk or the clock: the app sends the
//! request, saves [`LicenseState`] beside its settings and passes the time in.

mod activation;
mod record;
mod server;
mod state;
mod tool;

pub use activation::{ActivationOutcome, ActivationRequest, Refusal};
pub use p256::ecdsa::VerifyingKey;
pub use record::{Claims, RecordError, SignedRecord};
pub use server::{server_key, ACTIVATION_URL};
pub use state::{Access, Activation, LicenseState, RECHECK_DAYS, TRIAL_DAYS};
pub use tool::{Tool, BUNDLE_CHECKOUT_URL};

#[cfg(test)]
mod tests;
