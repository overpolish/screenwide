// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The microphone of a recording, the replay buffer or meeting notes, muted.
//!
//! Muted, what the microphone hears is handed on as silence: a recording's
//! track keeps its length and its sync, as a voice note's span does
//! (`note_gate`), and notes hear nothing to write. The microphone stays open,
//! so unmuting picks the voice up again at once. Dictation and voice notes
//! open their own microphone and are left as they are: each listens only
//! while its key is held.

use std::sync::atomic::{AtomicBool, Ordering};

static MUTED: AtomicBool = AtomicBool::new(false);

/// Mutes or unmutes the microphone of whatever is running.
pub fn set(muted: bool) {
  MUTED.store(muted, Ordering::Relaxed);
}

pub fn is_muted() -> bool {
  MUTED.load(Ordering::Relaxed)
}

/// `samples` as the session takes them: silence while muted.
pub(crate) fn heard(samples: &mut [f32]) {
  if is_muted() {
    samples.fill(0.0);
  }
}
