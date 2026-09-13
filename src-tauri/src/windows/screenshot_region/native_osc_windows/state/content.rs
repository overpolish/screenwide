// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Port of `+snapshot.m:13-20`: the frozen desktop is pushed per display and
/// kept by the surface that owns that display.
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

/// Port of `screenwide_region_osc_set_ocr` (`+ocr.m:137-185`). Every surface
/// keeps the share of the highlights that lands on it, and the surface showing
/// most of the selection hosts the status pill or the ready toolbar.
pub(crate) fn set_ocr(
  window: &WebviewWindow,
  phase: u32,
  rects: &[OcrRectPacket],
  message: &str,
) -> bool {
  if phase > 3 {
    eprintln!("The Windows region OSC refused an unknown OCR phase: {phase}");
    return false;
  }
  with_surfaces(window, |set| {
    let mut areas = Vec::new();
    for surface in set.all_mut() {
      let bounds = surface.logical_size();
      let offset = surface.desktop_offset();
      let local = surface.local_rect(surface.region);
      surface.ocr.apply(phase, rects, message, offset, bounds);
      areas.push(super::super::ocr::overlap_area(local, bounds));
    }
    let target = areas
      .iter()
      .copied()
      .enumerate()
      .filter(|(_, area)| *area > 0.0)
      .max_by(|left, right| left.1.total_cmp(&right.1))
      .map(|(index, _)| index);
    for (index, surface) in set.all_mut().enumerate() {
      surface.ocr.set_target(target == Some(index));
      surface.draw();
    }
    true
  })
  .unwrap_or(false)
}

pub(crate) fn set_ocr_cancel_visible(window: &WebviewWindow, visible: bool) -> bool {
  with_surfaces(window, |set| {
    for surface in set.all_mut() {
      surface.ocr.set_cancel_visible(visible);
      surface.draw();
    }
  })
  .is_some()
}

/// Port of `ocr::reset_input` (`native_osc_macos/ocr.rs:33`): the next press
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
