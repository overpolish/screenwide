// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake runs on macOS and Windows only.

const UNAVAILABLE: &str = "Keep Awake is not available on this system";

pub(super) struct Hold;

impl Hold {
  pub(super) fn system(_: &str) -> Result<Self, String> {
    Err(UNAVAILABLE.to_owned())
  }

  pub(super) fn display(_: &str) -> Result<Self, String> {
    Err(UNAVAILABLE.to_owned())
  }
}

pub(crate) struct LockWatch;

impl LockWatch {
  pub(crate) fn start(_: impl Fn(bool) + Send + Sync + 'static) -> Result<Self, String> {
    Err(UNAVAILABLE.to_owned())
  }
}
