// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Lets WebView2 shrink the pages of windows while they are hidden. Every
//! window's page is loaded at launch so the window opens at once, and each
//! holds tens of MB doing nothing until then. While one of [`TRIMMED`] is
//! hidden its page's memory target is low, so WebView2 trims it; shown, it is
//! normal again. Windows that must appear the moment a gesture or shortcut
//! asks (the island, tooltips, the region selector, Glide, menus) keep their
//! pages as they are.
//!
//! Visibility is read from the windows themselves, through a hook on this
//! process's show and hide events, rather than from each of the many places
//! that show or hide one.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicIsize, Ordering};

use tauri::{AppHandle, Manager};
use webview2_com::Microsoft::Web::WebView2::Win32::{
  ICoreWebView2_19, COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW,
  COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL,
};
use windows::core::Interface;
use windows::Win32::{
  Foundation::HWND,
  System::Threading::GetCurrentProcessId,
  UI::{
    Accessibility::{SetWinEventHook, HWINEVENTHOOK},
    WindowsAndMessaging::{
      IsWindowVisible, CHILDID_SELF, EVENT_OBJECT_HIDE, EVENT_OBJECT_SHOW, OBJID_WINDOW,
      WINEVENT_OUTOFCONTEXT,
    },
  },
};

use crate::app_windows::WindowLabel;

/// The windows whose pages may be trimmed while hidden: ones a person opens
/// deliberately, where a moment's wait the first time after a long idle goes
/// unnoticed.
const TRIMMED: &[WindowLabel] = &[
  WindowLabel::EditorRecording,
  WindowLabel::EditorScreenshot,
  WindowLabel::ToolPanelRecording,
  WindowLabel::ToolPanelScreenshot,
  WindowLabel::Projects,
  WindowLabel::Settings,
];

/// The hook lives as long as the process; it is never unhooked.
static HOOK: AtomicIsize = AtomicIsize::new(0);

thread_local! {
  /// Each trimmed window's page, by its window. Filled and read on the main
  /// thread, where the pages live and the hook delivers.
  static PAGES: RefCell<HashMap<isize, ICoreWebView2_19>> = RefCell::new(HashMap::new());
}

/// Sets each trimmed page's target from whether its window is shown, and
/// follows it from then on. Must run on the main thread, which owns the
/// windows and pumps the messages the hook delivers on.
pub fn initialize(app: &AppHandle) -> std::io::Result<()> {
  for label in TRIMMED {
    let Some(window) = app.get_webview_window(label.as_str()) else {
      continue;
    };
    let Ok(hwnd) = window.hwnd() else {
      continue;
    };
    let handle = hwnd.0 as isize;
    let _ = window.with_webview(move |webview| {
      // A WebView2 runtime older than the memory target has no
      // ICoreWebView2_19; its pages are left as they are.
      let Some(page) = (unsafe { webview.controller().CoreWebView2() })
        .ok()
        .and_then(|page| page.cast::<ICoreWebView2_19>().ok())
      else {
        return;
      };
      set_target(
        &page,
        unsafe { IsWindowVisible(HWND(handle as _)) }.as_bool(),
      );
      PAGES.with(|pages| pages.borrow_mut().insert(handle, page));
    });
  }
  if HOOK.load(Ordering::Acquire) != 0 {
    return Ok(());
  }
  let hook = unsafe {
    SetWinEventHook(
      EVENT_OBJECT_SHOW,
      EVENT_OBJECT_HIDE,
      None,
      Some(on_show_or_hide),
      GetCurrentProcessId(),
      0,
      WINEVENT_OUTOFCONTEXT,
    )
  };
  if hook.is_invalid() {
    return Err(std::io::Error::last_os_error());
  }
  HOOK.store(hook.0 as isize, Ordering::Release);
  Ok(())
}

fn set_target(page: &ICoreWebView2_19, shown: bool) {
  let level = if shown {
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_NORMAL
  } else {
    COREWEBVIEW2_MEMORY_USAGE_TARGET_LEVEL_LOW
  };
  let _ = unsafe { page.SetMemoryUsageTargetLevel(level) };
}

unsafe extern "system" fn on_show_or_hide(
  _hook: HWINEVENTHOOK,
  event: u32,
  window: HWND,
  object: i32,
  child: i32,
  _thread: u32,
  _time: u32,
) {
  // The window itself, not one of the objects inside it.
  if object != OBJID_WINDOW.0 || child != CHILDID_SELF as i32 {
    return;
  }
  PAGES.with(|pages| {
    if let Some(page) = pages.borrow().get(&(window.0 as isize)) {
      set_target(page, event == EVENT_OBJECT_SHOW);
    }
  });
}
