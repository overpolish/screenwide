// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn set_committed(window: &WebviewWindow, rect: Option<Rect>) -> bool {
  with_context(window, |context| {
    context
      .runtime
      .controller
      .lock()
      .map(|mut controller| controller.set_committed(rect))
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

/// Clears the borrowed OSC before its window can be presented for a quick
/// screenshot. The recording region remains in frontend storage and will be
/// synchronized back when the normal Region editor resumes.
pub(crate) fn clear_region(window: &WebviewWindow) -> bool {
  with_context(window, |context| {
    if let Ok(mut controller) = context.runtime.controller.lock() {
      let _ = controller.set_committed(None);
    }
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.region = Rect::default();
      scene.visible = false;
    }
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    set.apply_region(Rect::default(), false);
    true
  })
  .unwrap_or(false)
}

pub(crate) fn present_region(window: &WebviewWindow, rect: Option<Rect>) -> bool {
  let rect = rect.unwrap_or_default();
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.region = rect;
      scene.visible = true;
    }
    let Ok(mut set) = context.surfaces.lock() else {
      return false;
    };
    set.apply_region(rect, true);
    true
  })
  .unwrap_or(false)
}

pub(crate) fn region_scene(window: &WebviewWindow) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .map(|scene| scene.presented())
  })
  .flatten()
}

pub(crate) fn region_scene_request_base(
  window: &WebviewWindow,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .map(|scene| scene.request_base(owner))
  })
  .flatten()
}

pub(crate) fn reconcile_region_scene_request(
  window: &WebviewWindow,
  requested: RegionScene,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .and_then(|mut state| state.reconcile_request(requested, owner))
  })
  .flatten()
}

pub(crate) fn restore_normal_region_scene(window: &WebviewWindow) -> bool {
  let Some(scene) = with_context(window, |context| {
    context
      .runtime
      .scene
      .lock()
      .ok()
      .and_then(|state| state.normal_presentation())
  })
  .flatten() else {
    return false;
  };
  apply_region_scene(window, scene)
}

/// Applies the portable Region scene to the Windows compositor. The adapter
/// diffs lifecycle-owned fields so a workflow refresh cannot needlessly
/// re-present desktop or snapshot surfaces.
pub(crate) fn apply_region_scene(window: &WebviewWindow, next: RegionScene) -> bool {
  if next.overlay != overlay_palette() {
    eprintln!("The Windows region OSC refused a scene with a foreign overlay palette");
    return false;
  }
  let Some(previous) = with_context(window, |context| {
    let mut scene = context.runtime.scene.lock().ok()?;
    let previous = scene.presented();
    scene.set_presented(next);
    context.runtime.allow_drawing.store(
      next.interaction.allow_drawing,
      std::sync::atomic::Ordering::Relaxed,
    );
    let mut controller = context.runtime.controller.lock().ok()?;
    controller.set_aspect(next.interaction.aspect);
    Some(previous)
  })
  .flatten() else {
    return false;
  };

  if previous.chrome.frame_visible != next.chrome.frame_visible {
    set_show_frame(window, next.chrome.frame_visible);
  }
  if previous.chrome.handles_visible != next.chrome.handles_visible {
    set_show_handles(window, next.chrome.handles_visible);
  }
  if previous.interaction.input_enabled != next.interaction.input_enabled {
    set_input_enabled(window, next.interaction.input_enabled);
  }
  if previous.interaction.exclusion_rect != next.interaction.exclusion_rect {
    set_exclusion_rect(window, next.interaction.exclusion_rect.unwrap_or_default());
  }
  if previous.snapshot.presented != next.snapshot.presented {
    set_snapshot_presented(window, next.snapshot.presented);
  }
  if previous.snapshot.composited != next.snapshot.composited {
    set_snapshot_composited(window, next.snapshot.composited);
  }
  // Geometry is submitted before desktop peers are presented so a newly shown
  // surface can never expose the previous tool's cutout for a frame.
  let presented = with_surfaces(window, |set| {
    set.apply_region(next.region, next.visible);
    true
  })
  .unwrap_or(false);
  if !presented {
    return false;
  }
  if previous.desktop_presented != next.desktop_presented {
    set_desktop_presented(window, next.desktop_presented);
  }
  true
}
