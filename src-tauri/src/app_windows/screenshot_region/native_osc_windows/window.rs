// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// The region's own dispatch: the anchor child and the desktop peers are
/// built the same way, so one call covers both.
pub(super) fn create_on_owning_thread(
  window: &tauri::WebviewWindow,
  host: HWND,
  peer: Option<(Rect, bool)>,
) -> Result<HWND, String> {
  overlay_surface::create_on_owning_thread(window, host, move |host| match peer {
    None => create_overlay(host),
    Some((bounds, capturable)) => create_peer(host, bounds, capturable),
  })
}

fn create_overlay(parent: HWND) -> Result<HWND, String> {
  let atom = *OVERLAY_CLASS.get_or_init(|| register_class(w!("ScreenwideRegionOsc")));
  if atom == 0 {
    return Err("The Windows region OSC window class could not be registered".to_owned());
  }
  overlay_surface::create_child(parent, w!("ScreenwideRegionOsc"))
}

/// The peer window: `NSWindowStyleMaskNonactivatingPanel` becomes
/// `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, and `NSWindowSharingNone` becomes
/// `WDA_EXCLUDEFROMCAPTURE` unless the user records Screenwide's own windows.
fn create_peer(owner: HWND, bounds: Rect, capturable: bool) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  let atom = *PEER_CLASS.get_or_init(|| register_class(w!("ScreenwideRegionOscPeer")));
  if atom == 0 {
    return Err("The Windows region OSC peer class could not be registered".to_owned());
  }
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_TRANSPARENT,
      w!("ScreenwideRegionOscPeer"),
      PCWSTR::null(),
      WS_POPUP,
      bounds.origin.x as i32,
      bounds.origin.y as i32,
      (bounds.size.width as i32).max(1),
      (bounds.size.height as i32).max(1),
      // Owned, not parented: the peer stays above the region selector without
      // becoming part of its client area, and closes with it.
      Some(owner),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows region OSC peer could not be created: {error}"))?;
  // Top-level cross-process click-through requires the peer to be both layered
  // and transparent. Full opacity keeps DirectComposition's own per-pixel
  // alpha intact while activating the layered-window hit-test behavior.
  unsafe { SetLayeredWindowAttributes(hwnd, COLORREF(0), u8::MAX, LWA_ALPHA) }
    .map_err(|error| format!("The Windows region OSC peer could not enable layering: {error}"))?;
  disable_transitions(hwnd)?;
  if let Err(error) = overlay_surface::set_capture_affinity(hwnd, capturable) {
    eprintln!("The Windows region OSC peer could not set capture affinity: {error}");
  }
  Ok(hwnd)
}

/// The Tauri host owns passthrough for the anchor child. An independent
/// top-level peer uses the documented layered-plus-transparent combination so
/// mouse targeting can continue into windows belonging to another process.
pub(super) fn set_pointer_passthrough(hwnd: HWND, is_root: bool, passthrough: bool) -> bool {
  if is_root {
    return true;
  }
  let current = unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) };
  let next = peer_pointer_style(current, passthrough);
  if next != current {
    unsafe { SetWindowLongPtrW(hwnd, GWL_EXSTYLE, next) };
  }
  (unsafe { GetWindowLongPtrW(hwnd, GWL_EXSTYLE) }) == next
}

/// The region's classes: double clicks reach the controller as the modifier
/// bit the region gesture uses to expand to the full monitor.
fn register_class(name: PCWSTR) -> u16 {
  overlay_surface::register_class(name, Some(window_proc::window_proc), CS_DBLCLKS)
}

const fn peer_pointer_style(style: isize, passthrough: bool) -> isize {
  if passthrough {
    style | WS_EX_TRANSPARENT.0 as isize
  } else {
    style & !(WS_EX_TRANSPARENT.0 as isize)
  }
}

#[cfg(test)]
mod tests {
  use super::{peer_pointer_style, WS_EX_LAYERED, WS_EX_TRANSPARENT};

  #[test]
  fn peer_passthrough_preserves_layering_and_every_unrelated_extended_style() {
    let base = (0x0012_3456_isize | WS_EX_LAYERED.0 as isize) & !(WS_EX_TRANSPARENT.0 as isize);
    let transparent = peer_pointer_style(base, true);
    assert_ne!(transparent & WS_EX_TRANSPARENT.0 as isize, 0);
    assert_ne!(transparent & WS_EX_LAYERED.0 as isize, 0);
    assert_eq!(peer_pointer_style(transparent, false), base);
  }
}
