// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub fn configure_desktop(view: *mut c_void, binding: DesktopBinding, local: Option<Rect>) -> bool {
  with_context(view, |context| {
    if binding.anchor().is_none() {
      return false;
    }
    let committed = super::super::desktop::global_committed(&binding, local);
    let controller = RegionController::new(binding.virtual_monitor(), committed, None);
    let Ok(mut current_controller) = context.controller.lock() else {
      return false;
    };
    let Ok(mut desktop) = context.desktop.lock() else {
      return false;
    };
    *current_controller = controller;
    *desktop = Some(binding);
    true
  })
  .unwrap_or(false)
}

pub fn set_monitor(view: *mut c_void, width: f64, height: f64) -> bool {
  with_context(view, |context| {
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_monitor(Monitor {
          size: Size { width, height },
        })
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_allow_drawing(view: *mut c_void, allow_drawing: bool) -> bool {
  with_context(view, |context| {
    context
      .allow_drawing
      .store(allow_drawing, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.allow_drawing = allow_drawing;
    }
  })
  .is_some()
}

pub fn set_aspect(view: *mut c_void, aspect: Option<f64>) -> bool {
  with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.aspect = aspect;
    }
    context
      .controller
      .lock()
      .map(|mut controller| {
        controller.set_aspect(aspect);
        true
      })
      .unwrap_or(false)
  })
  .unwrap_or(false)
}

pub fn set_input_enabled(view: *mut c_void, enabled: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.interaction.input_enabled = enabled;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_input_enabled(view, i32::from(enabled)) };
  true
}

pub fn set_show_handles(view: *mut c_void, show_handles: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.chrome.handles_visible = show_handles;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_handles(view, i32::from(show_handles)) };
  true
}

pub fn set_show_frame(view: *mut c_void, show_frame: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.chrome.frame_visible = show_frame;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_show_frame(view, i32::from(show_frame)) };
  true
}

pub fn set_desktop_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.desktop_presented = presented;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_desktop_presented(view, i32::from(presented)) };
  true
}

pub fn claim_pointer_surface(view: *mut c_void) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_claim_pointer_surface(view) };
  true
}

pub fn refresh_ruler_pointer(view: *mut c_void) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_ruler_refresh_pointer(view) };
  true
}

pub fn set_ruler_transient_chrome(view: *mut c_void, visible: bool) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_ruler_set_transient_chrome(view, i32::from(visible)) };
  true
}

pub fn set_snapshot(
  view: *mut c_void,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  if with_context(view, |_| ()).is_none() {
    return false;
  }
  unsafe {
    ffi::screenwide_region_osc_set_snapshot(
      view,
      display_id,
      rgba.as_ptr(),
      rgba.len(),
      width,
      height,
    ) != 0
  }
}

pub fn set_snapshot_presented(view: *mut c_void, presented: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.snapshot.presented = presented;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_presented(view, i32::from(presented)) };
  true
}

pub fn set_snapshot_composited(view: *mut c_void, composited: bool) -> bool {
  if with_context(view, |context| {
    if let Ok(mut scene) = context.scene.lock() {
      scene.snapshot.composited = composited;
    }
  })
  .is_none()
  {
    return false;
  }
  unsafe { ffi::screenwide_region_osc_set_snapshot_composited(view, i32::from(composited)) };
  true
}
