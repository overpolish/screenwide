// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The transparent visual the selection chrome is drawn into, composed above
//! the workspace.

use windows::Win32::Graphics::DirectComposition::{IDCompositionDevice, IDCompositionVisual};

use screenwide_gpu::surface::Surface;
use screenwide_workspace::editing::ChromeRenderer;

pub(super) struct SelectionOverlay {
  pub(super) renderer: ChromeRenderer,
  /// Held only to keep the composition visual alive: the surface is attached
  /// once and no property is mutated after construction.
  _visual: IDCompositionVisual,
}

impl SelectionOverlay {
  /// The overlay's visual under `root`. The caller commits the tree.
  pub(super) fn new(
    shared: &'static screenwide_gpu::Gpu,
    composition: &IDCompositionDevice,
    root: &IDCompositionVisual,
  ) -> Result<Self, String> {
    let visual = unsafe { composition.CreateVisual() }.map_err(|error| error.to_string())?;
    unsafe { root.AddVisual(&visual, true, None::<&IDCompositionVisual>) }
      .map_err(|error| error.to_string())?;
    let surface = Surface::on_visual(shared, &visual)?;
    Ok(Self {
      renderer: ChromeRenderer::new(shared, surface),
      _visual: visual,
    })
  }
}
