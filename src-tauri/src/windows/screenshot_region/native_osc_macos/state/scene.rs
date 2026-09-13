// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn set_committed(view: *mut c_void, rect: Option<Rect>) -> bool {
  with_context(view, |context| {
    context
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
pub fn clear_region(view: *mut c_void) -> bool {
  if with_context(view, |context| {
    if let Ok(mut controller) = context.controller.lock() {
      let _ = controller.set_committed(None);
    }
    if let Ok(mut scene) = context.scene.lock() {
      scene.region = Rect::default();
      scene.visible = false;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set(view, 0.0, 0.0, 0.0, 0.0, 0) != 0 }
}

pub fn present_region(view: *mut c_void, rect: Option<Rect>) -> bool {
  let rect = rect.unwrap_or_default();
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.region = rect;
      scene.visible = true;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set(
      view,
      rect.origin.x,
      rect.origin.y,
      rect.size.width,
      rect.size.height,
      1,
    ) != 0
  }
}

pub fn region_scene(view: *mut c_void) -> Option<RegionScene> {
  with_context(view, |context| {
    context.scene.lock().ok().map(|scene| scene.presented())
  })
  .flatten()
}

pub fn region_scene_request_base(
  view: *mut c_void,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .map(|scene| scene.request_base(owner))
  })
  .flatten()
}

pub fn reconcile_region_scene_request(
  view: *mut c_void,
  requested: RegionScene,
  owner: RegionSceneOwner,
) -> Option<RegionScene> {
  with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .and_then(|mut state| state.reconcile_request(requested, owner))
  })
  .flatten()
}

pub fn restore_normal_region_scene(view: *mut c_void) -> bool {
  let Some(scene) = with_context(view, |context| {
    context
      .scene
      .lock()
      .ok()
      .and_then(|state| state.normal_presentation())
  })
  .flatten() else {
    return false;
  };
  apply_region_scene(view, scene)
}

/// Applies the portable Region scene to the macOS compositor. The adapter
/// diffs lifecycle-owned fields so a workflow refresh cannot needlessly
/// re-present desktop or snapshot surfaces.
pub fn apply_region_scene(view: *mut c_void, next: RegionScene) -> bool {
  if next.overlay != overlay_palette() {
    return false;
  }
  let Some(previous) = with_context(view, |context| {
    let mut scene = context.scene.lock().ok()?;
    let previous = scene.presented();
    scene.set_presented(next);
    context.allow_drawing.store(
      next.interaction.allow_drawing,
      std::sync::atomic::Ordering::Relaxed,
    );
    let mut controller = context.controller.lock().ok()?;
    controller.set_aspect(next.interaction.aspect);
    Some(previous)
  })
  .flatten() else {
    return false;
  };

  unsafe {
    if previous.chrome.frame_visible != next.chrome.frame_visible {
      ffi::screenwide_region_osc_set_show_frame(view, i32::from(next.chrome.frame_visible));
    }
    if previous.chrome.handles_visible != next.chrome.handles_visible {
      ffi::screenwide_region_osc_set_show_handles(view, i32::from(next.chrome.handles_visible));
    }
    if previous.interaction.input_enabled != next.interaction.input_enabled {
      ffi::screenwide_region_osc_set_input_enabled(view, i32::from(next.interaction.input_enabled));
    }
    if previous.interaction.exclusion_rect != next.interaction.exclusion_rect {
      let rect = next.interaction.exclusion_rect.unwrap_or_default();
      ffi::screenwide_region_osc_set_exclusion_rect(
        view,
        rect.origin.x,
        rect.origin.y,
        rect.size.width,
        rect.size.height,
      );
    }
    if previous.snapshot.presented != next.snapshot.presented {
      ffi::screenwide_region_osc_set_snapshot_presented(view, i32::from(next.snapshot.presented));
    }
    if previous.snapshot.composited != next.snapshot.composited {
      ffi::screenwide_region_osc_set_snapshot_composited(view, i32::from(next.snapshot.composited));
    }
    // Geometry is submitted before desktop peers are presented so a newly
    // shown surface can never expose the previous tool's cutout for a frame.
    let presented = ffi::screenwide_region_osc_set(
      view,
      next.region.origin.x,
      next.region.origin.y,
      next.region.size.width,
      next.region.size.height,
      i32::from(next.visible),
    ) != 0;
    if !presented {
      return false;
    }
    if previous.desktop_presented != next.desktop_presented {
      ffi::screenwide_region_osc_set_desktop_presented(view, i32::from(next.desktop_presented));
    }
    true
  }
}
