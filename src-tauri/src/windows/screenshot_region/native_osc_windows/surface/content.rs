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

  /// Port of `+snapshot.m:13-20`: the frozen desktop is per display, so only
  /// the surface owning `display_id` keeps the pixels.
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

  pub(super) fn upload(&self, rgba: &[u8], width: u32, height: u32) -> Option<Texture> {
    let expected = width as usize * height as usize * 4;
    if width == 0 || height == 0 || rgba.len() != expected {
      return None;
    }
    // The caller only lends the buffer for this call, so it is copied into a
    // texture before returning.
    match upload_rgba(self.gpu.device(), rgba, width, height) {
      Ok(view) => Some(Texture {
        view,
        size: (width, height),
      }),
      Err(error) => {
        eprintln!("The Windows region OSC could not upload a texture: {error}");
        None
      }
    }
  }

  /// Raises this surface and takes cursor ownership for the pointer.
  pub(crate) fn claim_pointer(&mut self) {
    self.raise();
    self.cursor = input::CursorShape::Crosshair;
    if let Ok(cursor) = unsafe { LoadCursorW(None, IDC_CROSS) } {
      unsafe { SetCursor(Some(cursor)) };
    }
  }

  pub(crate) fn release_pointer(&mut self) {
    self.cursor = input::CursorShape::None;
  }

  /// Keeps the root overlay sized to the host client area and above its
  /// WebView2 sibling, and each peer covering its own monitor above everything
  /// else - the Win32 form of `orderFrontRegardless` with the parent's level.
  pub(super) fn raise(&self) {
    match self.kind {
      Kind::Root { .. } => {
        let (width, height) = self.client_size();
        let _ = unsafe {
          SetWindowPos(
            self.hwnd,
            Some(HWND_TOP),
            0,
            0,
            width.max(1) as i32,
            height.max(1) as i32,
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
          )
        };
      }
      Kind::Peer { bounds, .. } => {
        let _ = unsafe {
          SetWindowPos(
            self.hwnd,
            Some(HWND_TOPMOST),
            bounds.origin.x as i32,
            bounds.origin.y as i32,
            (bounds.size.width as i32).max(1),
            (bounds.size.height as i32).max(1),
            SWP_ASYNCWINDOWPOS | SWP_NOACTIVATE | SWP_NOOWNERZORDER,
          )
        };
      }
    }
  }

  /// Drives the 16ms animation frames macOS scheduled with `dispatch_after`.
  pub(super) fn set_animating(&mut self, animating: bool) {
    if self.animating == animating {
      return;
    }
    self.animating = animating;
    if animating {
      let _ = unsafe { SetTimer(Some(self.hwnd), input::ANIMATION_TIMER, 16, None) };
    } else {
      let _ = unsafe { KillTimer(Some(self.hwnd), input::ANIMATION_TIMER) };
    }
  }
}
