// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

impl Surface {
  pub(crate) fn set_region(&mut self, region: Rect, visible: bool) {
    self.region = region;
    self.visible = visible;
    if !visible {
      self.magnifier = None;
    }
    self.draw();
  }

  pub(crate) fn set_magnifier_source(&mut self, rgba: &[u8], width: u32, height: u32) -> bool {
    match self.upload(rgba, width, height) {
      Some(texture) => {
        self.magnifier_source = Some(texture);
        self.draw();
        true
      }
      None => false,
    }
  }

  pub(crate) fn has_magnifier_source(&self) -> bool {
    self.magnifier_source.is_some()
  }

  /// The frozen desktop is per display, so only the surface owning
  /// `display_id` keeps the pixels.
  pub(crate) fn set_snapshot(&mut self, rgba: &[u8], width: u32, height: u32) -> bool {
    match self.upload(rgba, width, height) {
      Some(texture) => {
        self.snapshot = Some(texture);
        self.draw();
        true
      }
      None => false,
    }
  }

  /// A display's frozen desktop behind window picking, with the mip levels
  /// its blur reads.
  pub(crate) fn upload_backdrop(&self, rgba: &[u8], width: u32, height: u32) -> Option<Texture> {
    if width == 0 || height == 0 || rgba.len() != width as usize * height as usize * 4 {
      return None;
    }
    let shared = self.gpu.device();
    let texture = upload_rgba_with_levels(shared, rgba, width, height);
    self.gpu.mips.fill(shared, &texture);
    Some(Texture {
      view: texture.create_view(&Default::default()),
      size: (width, height),
    })
  }

  /// This surface's display's backdrop, or none.
  pub(crate) fn show_backdrop(&mut self, backdrop: Option<Texture>) {
    self.backdrop = backdrop;
    self.draw();
  }

  pub(super) fn upload(&self, rgba: &[u8], width: u32, height: u32) -> Option<Texture> {
    let expected = width as usize * height as usize * 4;
    if width == 0 || height == 0 || rgba.len() != expected {
      return None;
    }
    // The caller only lends the buffer for this call; the queue copies it
    // before returning.
    Some(Texture {
      view: upload_rgba(self.gpu.device(), rgba, width, height),
      size: (width, height),
    })
  }

  /// Raises this surface and takes cursor ownership for the pointer.
  pub(crate) fn claim_pointer(&mut self) {
    self.raise();
    self.cursor = input::CursorShape::Crosshair;
    host().set_cursor(input::CursorShape::Crosshair);
  }

  pub(crate) fn release_pointer(&mut self) {
    self.cursor = input::CursorShape::None;
  }

  /// Keeps the root overlay sized to the host client area and above the
  /// webview, and each peer covering its own monitor above everything else -
  /// the form `orderFrontRegardless` with the parent's level takes.
  pub(super) fn raise(&self) {
    match self.kind {
      Kind::Root { .. } => host().raise(self.hwnd, self.client_size()),
      Kind::Peer { bounds, .. } => host().place_peer(self.hwnd, bounds),
    }
  }

  /// Drives the 16ms animation frames macOS scheduled with `dispatch_after`.
  pub(super) fn set_animating(&mut self, animating: bool) {
    if self.animating == animating {
      return;
    }
    self.animating = animating;
    if animating {
      host().set_timer(self.hwnd, input::ANIMATION_TIMER, 16);
    } else {
      host().kill_timer(self.hwnd, input::ANIMATION_TIMER);
    }
  }
}
