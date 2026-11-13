// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A drag of sounds over the editor's page shows whether letting go there
//! adds anything: over a lane with room, the system's copy pointer; over a
//! full song rail, or anywhere off the lanes, the plain one, and letting go
//! slides the drag back. The webview answers every drag over it as a copy
//! (`wry`'s `draggingUpdated:`), so on the Mac that answer is wrapped: the
//! page says where a drop would be refused, and the wrapper answers none
//! there.

/// Says whether a sound let go where the pointer is now would be refused.
#[tauri::command]
pub fn set_sound_drop_refused(refused: bool) {
  #[cfg(target_os = "macos")]
  macos::set(refused);
  #[cfg(not(target_os = "macos"))]
  let _ = refused;
}

#[cfg(target_os = "macos")]
mod macos {
  use std::sync::atomic::{AtomicBool, Ordering};
  use std::sync::LazyLock;

  use objc2::runtime::{AnyClass, AnyObject, Imp, Sel};
  use objc2::sel;
  use objc2_app_kit::NSDragOperation;

  static REFUSED: AtomicBool = AtomicBool::new(false);
  /// The webview's own answer, wrapped the first time a drop is refused: by
  /// then the webview's class is registered.
  static ORIGINAL: LazyLock<Option<Imp>> = LazyLock::new(|| {
    let method = AnyClass::get(c"WryWebView")?.instance_method(sel!(draggingUpdated:))?;
    // Safety: the replacement takes and returns what `draggingUpdated:`
    // does, and calls through to what it replaces.
    let replacement: Imp =
      unsafe { std::mem::transmute::<DraggingUpdated, Imp>(dragging_updated as DraggingUpdated) };
    Some(unsafe { method.set_implementation(replacement) })
  });

  type DraggingUpdated =
    unsafe extern "C-unwind" fn(*mut AnyObject, Sel, *mut AnyObject) -> NSDragOperation;

  /// The webview's answer, unless the page refuses the drop: then none.
  unsafe extern "C-unwind" fn dragging_updated(
    this: *mut AnyObject,
    selector: Sel,
    info: *mut AnyObject,
  ) -> NSDragOperation {
    // Only ever called once wrapped, so the original is there.
    let Some(original) = *ORIGINAL else {
      return NSDragOperation::Copy;
    };
    // Safety: the implementation was the webview's own for this selector,
    // whose signature this is.
    let original: DraggingUpdated = unsafe { std::mem::transmute(original) };
    // Asked first in every case, so the page still hears where the drag is.
    let answer = unsafe { original(this, selector, info) };
    if REFUSED.load(Ordering::Acquire) {
      NSDragOperation::None
    } else {
      answer
    }
  }

  pub(super) fn set(refused: bool) {
    if refused {
      LazyLock::force(&ORIGINAL);
    }
    REFUSED.store(refused, Ordering::Release);
  }
}
