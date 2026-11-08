// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Keep Awake runs on macOS and Windows only; elsewhere it is never allowed
//! to start.

use std::time::Duration;

pub(super) fn permitted() -> bool {
  false
}

pub(super) fn idle() -> Duration {
  Duration::ZERO
}

pub(super) fn screen_in_use() -> bool {
  false
}

pub(super) fn nudge() {}
