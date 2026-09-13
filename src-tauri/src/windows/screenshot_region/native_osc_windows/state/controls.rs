// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

pub(crate) fn set_monitor(window: &WebviewWindow, width: f64, height: f64) -> bool {
  with_context(window, |context| {
    context
      .runtime
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

pub(crate) fn set_allow_drawing(window: &WebviewWindow, allow_drawing: bool) -> bool {
  with_context(window, |context| {
    context
      .runtime
      .allow_drawing
      .store(allow_drawing, std::sync::atomic::Ordering::Relaxed);
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.allow_drawing = allow_drawing;
    }
  })
  .is_some()
}

pub(crate) fn set_magnifier_source(
  window: &WebviewWindow,
  display_id: u32,
  rgba: &[u8],
  width: u32,
  height: u32,
) -> bool {
  with_surfaces(window, |set| {
    set
      .for_display_mut(display_id)
      .is_some_and(|surface| surface.set_magnifier_source(rgba, width, height))
  })
  .unwrap_or(false)
}

pub(crate) fn set_aspect(window: &WebviewWindow, aspect: Option<f64>) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.aspect = aspect;
    }
    context
      .runtime
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

/// Port of `screenwide_region_osc_set_input_enabled` (`+state.m:25-76`):
/// disabling while a gesture runs cancels it through the runtime first, so the
/// controller never keeps a half-finished drag across a workflow change.
pub(crate) fn set_input_enabled(window: &WebviewWindow, enabled: bool) -> bool {
  let Some(cancel) = with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.interaction.input_enabled = enabled;
    }
    context
      .surfaces
      .lock()
      .map(|mut set| {
        !enabled
          && set
            .all_mut()
            .any(|surface| surface.input_enabled && surface.gesture_active)
      })
      .unwrap_or(false)
  }) else {
    return false;
  };
  if cancel {
    let cancelled = with_context(window, |context| {
      dispatch_input(context, 5, Point::default(), 0)
    });
    if let Some(result) = cancelled {
      if result.ruler_flags & 1 == 0 {
        let region = if result.has_region == 0 {
          Rect::default()
        } else {
          Rect::from_xywh(result.x, result.y, result.width, result.height)
        };
        with_surfaces(window, |set| {
          let visible = set.root_mut().visible;
          set.apply_region(region, visible);
        });
      }
    }
  }
  let Some(surfaces) = with_surfaces(window, |set| {
    let mut surfaces = Vec::with_capacity(set.peers.len() + 1);
    for surface in set.all_mut() {
      // Ready OCR packets refresh the compositor after every changed caret.
      // They repeat `input_enabled = true` and must not end the Win32 mouse
      // capture that owns the current drag. Only disabling input cancels a
      // gesture; a redundant enabled update preserves it.
      surface.gesture_active = gesture_after_input_update(surface.gesture_active, enabled);
      surface.input_enabled = enabled;
      surfaces.push((surface.hwnd(), surface.is_root()));
      if !enabled {
        surface.magnifier = None;
        surface.release_pointer();
      }
      surface.draw();
    }
    surfaces
  }) else {
    return false;
  };
  // Keep native style changes outside the SurfaceSet mutex.
  surfaces
    .iter()
    .all(|(hwnd, is_root)| surface::set_pointer_passthrough(*hwnd, *is_root, !enabled))
}

pub(super) fn gesture_after_input_update(active: bool, enabled: bool) -> bool {
  active && enabled
}

pub(crate) fn set_show_handles(window: &WebviewWindow, show_handles: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.chrome.handles_visible = show_handles;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.show_handles = show_handles;
      }
    }
  })
  .is_some()
}

pub(crate) fn set_show_frame(window: &WebviewWindow, show_frame: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.chrome.frame_visible = show_frame;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        surface.show_frame = show_frame;
        surface.draw();
      }
    }
  })
  .is_some()
}

/// The exclusion rect is the webview's own toolbar, so it belongs to the
/// anchor surface only (`+state.m:78-88` zeroes it on every peer).
pub(super) fn set_exclusion_rect(window: &WebviewWindow, rect: Rect) -> bool {
  with_surfaces(window, |set| {
    set.root_mut().exclusion_rect = rect;
    for peer in set.peers.iter_mut() {
      peer.exclusion_rect = Rect::default();
    }
  })
  .is_some()
}

/// Port of `screenwide_region_osc_set_desktop_presented` (`+desktop.m:247-271`):
/// window ordering only. With no peers - the single-monitor case - this stays
/// a scene mirror and nothing is ordered.
pub(crate) fn set_desktop_presented(window: &WebviewWindow, presented: bool) -> bool {
  with_context(window, |context| {
    if let Ok(mut scene) = context.runtime.scene.lock() {
      scene.desktop_presented = presented;
    }
    if let Ok(mut set) = context.surfaces.lock() {
      for surface in set.all_mut() {
        if !presented {
          surface.release_pointer();
        }
        // The root ignores the flag when it draws; it is mirrored there only
        // so a peer rebuild can inherit the current presentation.
        surface.desktop_presented = presented;
        surface.draw();
      }
    }
  })
  .is_some()
}

/// Port of `claimPointerSurfaceNow` (`+input.m:141-157`): the surface whose
/// window contains the pointer takes the cursor, and every other one lets go.
pub(crate) fn claim_pointer_surface(window: &WebviewWindow) -> bool {
  let target = with_surfaces(window, |set| {
    let pointer = surface::cursor_position();
    let mut claimed = false;
    let mut target = None;
    if let Some(pointer) = pointer {
      for surface in set.all_mut() {
        if !claimed
          && surface.input_enabled
          && surface.visible
          && surface.contains_screen_point(pointer)
        {
          surface.claim_pointer();
          claimed = true;
          target = Some(surface.hwnd());
        } else {
          surface.release_pointer();
        }
      }
    }
    // With no peers the anchor surface is the only candidate; claiming it
    // keeps the single-monitor path identical to stage 1.
    if !claimed && set.peers.is_empty() {
      set.root_mut().claim_pointer();
      target = Some(set.root_hwnd());
    }
    target
  });
  if let Some(Some(hwnd)) = target {
    // WebView2 can apply its arrow cursor at the end of the same focus/show
    // turn. Reassert ours on the next owner-thread message, mirroring macOS's
    // immediate + dispatch_async cursor claim.
    let _ = unsafe {
      PostMessageW(
        Some(hwnd),
        input::RULER_CURSOR_EVENT,
        windows::Win32::Foundation::WPARAM(0),
        windows::Win32::Foundation::LPARAM(0),
      )
    };
  }
  target.is_some()
}
