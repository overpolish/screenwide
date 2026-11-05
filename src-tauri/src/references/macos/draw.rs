// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Drawing a reference on macOS: the picture's and the chrome's Metal
//! layers, presented through the shared OSC renderer at the window's scale
//! (`references::drawing` says what they hold).

use objc2::MainThreadMarker;
use objc2_quartz_core::CATransaction;

use super::super::drawing::{chrome_vertices, constants, picture_vertices, view_size};
use super::Native;

/// Draws the picture and its chrome at their present size.
pub(super) fn draw(mtm: MainThreadMarker, native: &mut Native) {
  let scale = native.window.backingScaleFactor();
  let size = native.stage.size;
  let view = view_size(size);
  let pixels = (
    (view.width * scale).ceil().max(1.0) as u32,
    (view.height * scale).ceil().max(1.0) as u32,
  );
  let constants = constants(crate::app_windows::platform::light_appearance(mtm), scale);
  let handles = native.stage.hovered || native.stage.resizing();
  let drawn = native
    .picture
    .present(pixels, &picture_vertices(size), &constants)
    .and_then(|()| {
      native
        .chrome
        .present(pixels, &chrome_vertices(size, scale, handles), &constants)
    });
  if let Err(error) = drawn {
    eprintln!("Could not draw a reference: {error}");
  }
}

/// Runs `change` with implicit animations off, so a layer lands at once.
pub(super) fn transaction(change: impl FnOnce()) {
  CATransaction::begin();
  CATransaction::setDisableActions(true);
  change();
  CATransaction::commit();
}
