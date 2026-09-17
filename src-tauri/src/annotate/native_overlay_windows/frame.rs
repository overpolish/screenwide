// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame per attached surface.

use super::*;

/// Redraws every attached surface. Owning thread only: the child windows and
/// their swap chains belong to the thread that created the hosts.
pub(super) fn draw_all() {
  let mut overlay = overlay();
  let overlay = &mut *overlay;
  let Some(renderer) = overlay.renderer.as_ref() else {
    return;
  };
  for surface in &mut overlay.surfaces {
    if let Err(error) = renderer.draw(surface) {
      eprintln!("The annotate overlay could not draw a display: {error}");
    }
  }
}

impl Renderer {
  fn draw(&self, surface: &mut Surface) -> Result<(), String> {
    let Some(display) = display(surface.display) else {
      return Ok(());
    };
    let size = surface.fit()?;
    if size.0 == 0 || size.1 == 0 {
      return Ok(());
    }
    let target = surface.chain.back_buffer_view(self.device().device())?;
    self.draw_arrows(&target, size, &scene::scene(display))?;
    surface.chain.present()
  }
}
