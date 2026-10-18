// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Where a picture dropped on the workspace goes; the input view is the drop
//! target.

use super::*;

impl RecordingPreviewSurface {
  /// Installs the callback a picture dropped on the workspace is handed to.
  /// Without one, nothing can be dropped there.
  pub(crate) fn set_image_drop_callback(&mut self, callback: ImageDropCallback) {
    if let Ok(mut slot) = mac(&self.inner).image_drop.lock() {
      *slot = Some(callback);
    }
  }
}
