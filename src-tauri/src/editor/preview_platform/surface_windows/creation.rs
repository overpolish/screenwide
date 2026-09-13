// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl RecordingPreviewSurface {
  /// The compositor already open for one editor workspace's window, without
  /// creating one: the export path renders offscreen on the surface the window
  /// it is saving for has, and never opens a GPU device of its own.
  pub(crate) fn existing_for(
    kind: crate::editor::EditorKind,
  ) -> Result<std::sync::Arc<Self>, String> {
    let inner = surface_index()
      .lock()
      .map_err(|_| "The Windows GPU compositor registry is unusable".to_owned())?
      .by_kind
      .get(&kind)
      .map(std::sync::Arc::clone)
      .ok_or_else(|| "The Windows GPU compositor has not been opened".to_owned())?;
    Ok(std::sync::Arc::new(Self { inner }))
  }

  pub(crate) fn from_window(window: &WebviewWindow) -> Result<Self, String> {
    let host = window.hwnd().map_err(|error| error.to_string())?;
    let host = HWND(host.0);
    // The window's own workspace: an editor window is the only window a surface
    // is ever created for, and its label is what `existing_for` looks up later.
    let kind = crate::editor::EditorKind::from_window_label(window.label());
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
        let inner = std::sync::Arc::new(SurfaceInner {
          batch_depth: AtomicU32::new(0),
          selection_pending: std::sync::atomic::AtomicBool::new(false),
          callbacks: Mutex::new(EditorCallbacks::default()),
          editor,
          gpu,
          state: Mutex::new(SurfaceState {
            backdrop: [0.09, 0.09, 0.10, 1.0],
            camera_source: None,
            editor_active: false,
            frame_resize: None,
            frame_resize_committed: false,
            move_auto_fit: None,
            gesture: None,
            last_pointer: (0.0, 0.0),
            panes: Vec::new(),
            panel_fit_width: 0.0,
            primary_composition: None,
            scale: 1.0,
            selection: None,
            selection_visible: true,
            selection_snapping_enabled: false,
            selection_snap_guide_x: None,
            selection_snap_guide_y: None,
            selection_targets: Vec::new(),
            viewport: PreviewSurfaceRect {
              height: 0.0,
              width: 0.0,
              x: 0.0,
              y: 0.0,
            },
            workspace_allows_upscale: false,
            workspace_transform: WorkspaceTransform::default(),
            workspace_natural_size: None,
            workspace_transforms: HashMap::new(),
          }),
        });
        // Published only once the surface is whole: the editor `window_proc`
        // and the export path both find it through these, and both no-op
        // while a window is still opening its compositor.
        if let Ok(mut index) = surface_index().lock() {
          index.by_editor.insert(
            inner.editor.hwnd().0 as isize,
            std::sync::Arc::clone(&inner),
          );
          if let Some(kind) = kind {
            index.by_kind.insert(kind, std::sync::Arc::clone(&inner));
          }
        }
        Ok(inner)
      })
      .as_ref()
      .map_err(Clone::clone)?;
    Ok(Self {
      inner: std::sync::Arc::clone(inner),
    })
  }

  pub(crate) fn device(&self) -> ID3D11Device {
    self.inner.gpu.device.clone()
  }
}
