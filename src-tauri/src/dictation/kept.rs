// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Dictation's helper kept for a few minutes after each dictation, with its
//! models loaded, so the next one starts at once.

use std::time::Duration;

use screenwide_dictation::Helper;

use super::with;

/// How long dictation's helper is kept after a dictation.
const KEEP_HELPER: Duration = Duration::from_secs(180);

/// Keeps the helper for the next dictation, and ends it once none has come
/// for `KEEP_HELPER`, so its model's memory is given back.
pub(super) fn keep_helper(helper: Option<Helper>) {
  let used = with(|dictation| {
    dictation.helper = helper;
    dictation.used
  });
  std::thread::spawn(move || {
    std::thread::sleep(KEEP_HELPER);
    let idle = with(|dictation| {
      (dictation.used == used && dictation.listening.is_none())
        .then(|| dictation.helper.take())
        .flatten()
    });
    drop(idle);
  });
}
