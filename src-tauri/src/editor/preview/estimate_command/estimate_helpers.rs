// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use crate::editor::EditorState;

/// What one estimate is kept under: the recording, which of its baked
/// layers and which file it is, its compression and its scale.
pub(super) type EstimateKey = (u64, u8, u8, u16);

/// The estimate kept under `key`, if one was made.
pub(super) fn kept_estimate(state: &EditorState, key: &EstimateKey) -> Option<u64> {
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .get(key)
    .copied()
}

/// Keeps `bytes` as the estimate under `key`, answering it.
pub(super) fn keep_estimate(state: &EditorState, key: EstimateKey, bytes: u64) -> u64 {
  state
    .compression_estimates
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner())
    .insert(key, bytes);
  bytes
}

/// Cancels the estimate running before this one, whose answer nobody waits
/// for any more, and answers the flag that cancels this one in turn.
pub(super) fn replace_running(state: &EditorState) -> Arc<AtomicBool> {
  let cancelled = Arc::new(AtomicBool::new(false));
  std::mem::replace(
    &mut *state
      .running_estimate
      .lock()
      .unwrap_or_else(|poisoned| poisoned.into_inner()),
    cancelled.clone(),
  )
  .store(true, Ordering::Release);
  cancelled
}

/// `media`, the file's streams, with its MP4 tables: small but real. Half a
/// percent plus the fixed headers.
pub(super) fn with_container(media: u64) -> u64 {
  media.saturating_add(media / 200).saturating_add(4_096)
}
