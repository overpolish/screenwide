// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! One frame per attached surface.

use super::*;

/// Redraws every attached surface. Owning thread only: the child windows and
/// their surfaces belong to the thread that created the hosts.
pub(super) fn draw_all() {
  let mut overlay = overlay();
  let overlay = &mut *overlay;
  let Some(renderer) = overlay.renderer.as_ref() else {
    return;
  };
  for surface in &mut overlay.surfaces {
    if let Err(error) = draw(renderer, surface) {
      eprintln!("The annotate overlay could not draw a display: {error}");
    }
  }
}

fn draw(renderer: &Renderer, surface: &mut Surface) -> Result<(), String> {
  let Some(display) = display(surface.display) else {
    return Ok(());
  };
  let size = surface.fit(renderer.gpu())?;
  if size.0 == 0 || size.1 == 0 {
    return Ok(());
  }
  let crate::gpu::surface::Frame::Ready(frame) = surface.chain.acquire(renderer.gpu())? else {
    return Ok(());
  };
  renderer.draw_display(
    &frame.texture.create_view(&Default::default()),
    size,
    surface.display,
    display.origin,
    display.scale,
    &mut surface.underlays,
  )?;
  renderer.gpu().queue.present(frame);
  Ok(())
}
