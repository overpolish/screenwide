// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(super) fn clear_active_export(app: &AppHandle, kind: ExportKind, artifact_id: u64) {
  let state = app.state::<ExportState>();
  let mut active = state
    .slot(kind)
    .active_export
    .lock()
    .unwrap_or_else(|poisoned| poisoned.into_inner());
  if active
    .as_ref()
    .is_some_and(|job| job.artifact_id == artifact_id)
  {
    active.take();
  }
}
