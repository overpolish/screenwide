// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The shortcut strip's compositor for a preview session.

use super::*;
use crate::editor::timeline_edit::TimelinePlan;

/// The shortcut strip recorded at `path`, with the shortcuts deleted and
/// placed as the session's `settings` say, or as the saved edit `persisted`
/// does where a headless session sends none.
pub(super) fn keyboard_compositor(
  path: &std::path::Path,
  settings: Option<&PreviewPlayerSettings>,
  persisted: Option<&TimelinePlan>,
) -> Result<Arc<KeyboardCompositor>, String> {
  let deleted_ids = settings
    .map(|settings| settings.deleted_keyboard_shortcut_ids.clone())
    .unwrap_or_else(|| {
      persisted.map_or_else(Vec::new, |plan| {
        plan.deleted_keyboard_shortcut_ids().to_vec()
      })
    });
  let deleted_ranges = settings
    .map(|settings| settings.deleted_keyboard_shortcut_ranges.clone())
    .unwrap_or_else(|| {
      persisted.map_or_else(Vec::new, |plan| {
        plan.deleted_keyboard_shortcut_ranges().to_vec()
      })
    });
  let compositor = KeyboardCompositor::open_with_deleted(path, &deleted_ids, &deleted_ranges)?;
  let positions = settings
    .map(|settings| settings.keyboard_shortcut_positions.as_slice())
    .or_else(|| persisted.map(|plan| plan.keyboard_shortcut_positions()))
    .unwrap_or(&[]);
  compositor.set_shortcut_positions(positions);
  Ok(Arc::new(compositor))
}
