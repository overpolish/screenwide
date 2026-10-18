// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  pub(crate) fn from_window(window: &WebviewWindow) -> Result<Self, String> {
    let host = window.hwnd().map_err(|error| error.to_string())?;
    let host = HWND(host.0);
    // An editor window is the only window a surface is ever created for.
    let slot = {
      let mut surfaces = preview_surfaces()
        .lock()
        .map_err(|_| "The Windows GPU compositor registry is unusable".to_owned())?;
      std::sync::Arc::clone(surfaces.entry(host.0 as isize).or_default())
    };
    let inner = slot
      .get_or_init(|| {
        let editor = create_editor_on_owning_thread(window, host)?;
        let gpu = Gpu::new(host, editor.hwnd())?;
        let editor_hwnd = editor.hwnd();
        let engine = WorkspaceEditor::new(Box::new(host::WindowsHost {
          editor,
          gpu,
          image_drop: Mutex::new(None),
        }));
        // Published only once the surface is whole: the editor `window_proc`
        // finds it through this, and no-ops while a window is still opening
        // its compositor.
        if let Ok(mut index) = surface_index().lock() {
          index
            .by_editor
            .insert(editor_hwnd.0 as isize, std::sync::Arc::clone(&engine.inner));
        }
        Ok(engine.inner)
      })
      .as_ref()
      .map_err(Clone::clone)?;
    Ok(Self {
      editor: WorkspaceEditor {
        inner: std::sync::Arc::clone(inner),
      },
    })
  }

  /// The Direct3D 11 device decoders write frames on: the layer over the
  /// shared wgpu device, so the frames are copied without leaving it.
  pub(crate) fn device(&self) -> ID3D11Device {
    windows(&self.inner).gpu.d3d11.device.clone()
  }
}
