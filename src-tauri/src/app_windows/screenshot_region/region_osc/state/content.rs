// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The frozen desktop is pushed per display and kept by the surface that
/// owns that display.
pub(crate) fn set_snapshot(
  window: &WebviewWindow,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  with_surfaces(window, |set| {
    set
      .for_display_mut(display_id)
      .is_some_and(|surface| surface.set_snapshot(rgba, width, height))
  })
  .unwrap_or(false)
}

/// Every surface keeps the share of the highlights that lands on it.
pub(crate) fn set_ocr(window: &WebviewWindow, phase: u32, rects: &[OcrRectPacket]) -> bool {
  if phase > 3 {
    eprintln!("The region OSC refused an unknown OCR phase: {phase}");
    return false;
  }
  with_surfaces(window, |set| {
    for surface in set.all_mut() {
      let bounds = surface.logical_size();
      let offset = surface.desktop_offset();
      surface.ocr.apply(phase, rects, offset, bounds);
      surface.draw();
    }
    true
  })
  .unwrap_or(false)
}

/// Port of `ocr::reset_input`: the next press
/// starts a fresh selection instead of a text interaction.
pub(crate) fn reset_text_recognition_input(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    if context.runtime.purpose != Purpose::TextRecognition {
      return false;
    }
    context
      .runtime
      .completed
      .store(false, std::sync::atomic::Ordering::Release);
    if let Ok(mut controller) = context.runtime.controller.lock() {
      let _ = controller.set_committed(None);
    }
    true
  })
  .unwrap_or(false)
}

/// The desktop behind window picking: each display's frozen picture, by
/// display id, for the surface covering it. None lets them go.
pub(crate) fn set_backdrops(window: &WebviewWindow, images: &[(u32, &[u8], u32, u32)]) -> bool {
  with_surfaces(window, |set| {
    let mut backdrops = HashMap::with_capacity(images.len());
    for &(display_id, rgba, width, height) in images {
      let Some(texture) = set.root.upload_backdrop(rgba, width, height) else {
        return false;
      };
      backdrops.insert(display_id, texture);
    }
    set.backdrops = backdrops;
    set.share_backdrops();
    true
  })
  .unwrap_or(false)
}

pub(crate) fn set_snapshot_presented(window: &WebviewWindow, presented: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.snapshot.presented = presented;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.snapshot_presented = presented;
        surface.draw();
      }
    }
  })
  .is_some()
}

pub(crate) fn set_snapshot_composited(window: &WebviewWindow, composited: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.snapshot.composited = composited;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.snapshot_composited = composited;
        surface.draw();
      }
    }
  })
  .is_some()
}
