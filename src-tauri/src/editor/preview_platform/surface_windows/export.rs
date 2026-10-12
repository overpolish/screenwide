// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;
use screenwide_export::cursor_export::windows::export_surface::{ExportSurface, Held};

impl ExportSurface for RecordingPreviewSurface {
  fn d3d11(&self) -> &'static D3d11Layer {
    self.inner.gpu.d3d11
  }

  fn compositor(&self) -> &compositor::Compositor {
    &self.inner.gpu.compositor
  }

  fn gpu(&self) -> &'static screenwide_gpu::Gpu {
    self.inner.gpu.shared
  }

  fn hold(&self) -> Result<Box<dyn Held + '_>, String> {
    let state = self
      .inner
      .state
      .lock()
      .map_err(|_| "The Windows GPU compositor is unavailable".to_owned())?;
    Ok(Box::new(state))
  }
}

/// The surface a recording's export composites on: the recording
/// workspace's own window owns the GPU device it draws with; the screenshot
/// window has a separate one.
pub(crate) fn recording_export_surface() -> Result<std::sync::Arc<dyn ExportSurface>, String> {
  Ok(RecordingPreviewSurface::existing_for(
    crate::editor::EditorKind::Recording,
  )?)
}
