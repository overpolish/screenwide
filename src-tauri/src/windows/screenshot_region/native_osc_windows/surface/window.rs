// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Replaces `effectiveAppearance`: the shell's app theme preference.
pub(super) fn light_mode() -> bool {
  let mut value = 0_u32;
  let mut length = size_of::<u32>() as u32;
  let status = unsafe {
    RegGetValueW(
      HKEY_CURRENT_USER,
      w!("Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize"),
      w!("AppsUseLightTheme"),
      RRF_RT_REG_DWORD,
      None,
      Some((&raw mut value).cast::<c_void>()),
      Some(&mut length),
    )
  };
  status == ERROR_SUCCESS && value != 0
}

/// Creates a window on the thread that owns the host HWND: Win32 queues a
/// window's messages on its creating thread, so a surface window created on a
/// worker thread would never see a mouse message.
pub(crate) fn create_on_owning_thread(
  window: &tauri::WebviewWindow,
  host: HWND,
  peer: Option<(Rect, bool)>,
) -> Result<HWND, String> {
  struct HostHandle(HWND);
  unsafe impl Send for HostHandle {}
  struct Created(Result<HWND, String>);
  unsafe impl Send for Created {}

  let build = move |host: HWND| match peer {
    None => create_overlay(host),
    Some((bounds, capturable)) => create_peer(host, bounds, capturable),
  };
  if unsafe { GetWindowThreadProcessId(host, None) } == unsafe { GetCurrentThreadId() } {
    return build(host);
  }
  let handle = HostHandle(host);
  let (sender, receiver) = std::sync::mpsc::sync_channel(1);
  window
    .run_on_main_thread(move || {
      let handle = handle;
      let _ = sender.send(Created(build(handle.0)));
    })
    .map_err(|error| format!("The Windows region OSC window could not be dispatched: {error}"))?;
  receiver
    .recv()
    .map_err(|_| "The Windows region OSC window was never created".to_owned())?
    .0
}

pub(super) fn create_overlay(parent: HWND) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  let atom = *OVERLAY_CLASS.get_or_init(|| register_class(instance.0, w!("ScreenwideRegionOsc")));
  if atom == 0 {
    return Err("The Windows region OSC window class could not be registered".to_owned());
  }
  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_NOREDIRECTIONBITMAP,
      w!("ScreenwideRegionOsc"),
      PCWSTR::null(),
      WS_CHILD | WS_CLIPSIBLINGS,
      0,
      0,
      1,
      1,
      Some(parent),
      Some(HMENU::default()),
      Some(HINSTANCE(instance.0)),
      None,
    )
  }
  .map_err(|error| format!("The Windows region OSC overlay could not be created: {error}"))?;
  Ok(hwnd)
}

/// The peer window: `NSWindowStyleMaskNonactivatingPanel` becomes
/// `WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW`, and `NSWindowSharingNone` becomes
/// `WDA_EXCLUDEFROMCAPTURE` unless the user records Screenwide's own windows.
pub(super) fn create_peer(owner: HWND, bounds: Rect, capturable: bool) -> Result<HWND, String> {
  let instance = unsafe { GetModuleHandleW(None) }.map_err(|error| error.to_string())?;
  let atom = *PEER_CLASS.get_or_init(|| register_class(instance.0, w!("ScreenwideRegionOscPeer")));
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
  if let Err(error) = set_capture_affinity(hwnd, capturable) {
    eprintln!("The Windows region OSC peer could not set capture affinity: {error}");
  }
  Ok(hwnd)
}

/// The Tauri host owns passthrough for the anchor child. An independent
/// top-level peer uses the documented layered-plus-transparent combination so
/// mouse targeting can continue into windows belonging to another process.
pub(crate) fn set_pointer_passthrough(hwnd: HWND, is_root: bool, passthrough: bool) -> bool {
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

pub(super) fn disable_transitions(hwnd: HWND) -> Result<(), String> {
  let disabled = windows::core::BOOL(1);
  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_TRANSITIONS_FORCEDISABLED,
      (&raw const disabled).cast(),
      std::mem::size_of::<windows::core::BOOL>() as u32,
    )
  }
  .map_err(|error| error.to_string())
}

pub(crate) fn set_capture_affinity(hwnd: HWND, capturable: bool) -> Result<(), String> {
  let affinity = if capturable {
    WDA_NONE
  } else {
    WDA_EXCLUDEFROMCAPTURE
  };
  unsafe { SetWindowDisplayAffinity(hwnd, affinity) }.map_err(|error| error.to_string())
}

pub(super) fn register_class(instance: *mut c_void, name: PCWSTR) -> u16 {
  unsafe {
    RegisterClassW(&WNDCLASSW {
      // Double clicks reach the controller as the modifier bit the region
      // gesture uses to expand to the full monitor.
      style: CS_DBLCLKS,
      lpfnWndProc: Some(input::window_proc),
      hInstance: HINSTANCE(instance),
      hCursor: LoadCursorW(None, IDC_ARROW).unwrap_or_default(),
      lpszClassName: name,
      ..Default::default()
    })
  }
}
