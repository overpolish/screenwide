// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! How the options window is put on screen: attached to its editor as a native
//! child, and held invisible until the frontend has said how tall its form is.
//!
//! The window opens at whatever height it was configured with, which is only
//! ever a starting point, so presenting it plainly means showing the wrong size
//! for as long as it takes the webview to lay out and answer. Instead it is
//! ordered front at zero alpha - on screen, focused, laying out - and the
//! resize that fits it is also what reveals it.

use tauri::{AppHandle, WebviewWindow};

/// Runs an AppKit closure on the main thread, without blocking when the caller
/// is already there. Synchronous commands arrive on the main thread, so the
/// common path never leaves it and never waits on it.
#[cfg(target_os = "macos")]
fn on_main_thread<F: FnOnce() + Send + 'static>(app: &AppHandle, work: F) -> tauri::Result<()> {
  if objc2::MainThreadMarker::new().is_some() {
    work();
    return Ok(());
  }
  app.run_on_main_thread(work)
}

/// Attaches the options window to its editor as a native child, and takes away
/// its ability to be dragged: it belongs where the editor puts it.
#[cfg(target_os = "macos")]
pub(super) fn attach(
  app: &AppHandle,
  editor: &WebviewWindow,
  options: &WebviewWindow,
) -> tauri::Result<()> {
  let parent = editor.ns_window()? as usize;
  let child = options.ns_window()? as usize;
  on_main_thread(app, move || {
    use objc2_app_kit::{NSWindow, NSWindowOrderingMode};

    // Both pointers come straight from the windows AppKit itself owns, and
    // this runs on the thread that owns them.
    let parent: &NSWindow = unsafe { &*(parent as *const NSWindow) };
    let child: &NSWindow = unsafe { &*(child as *const NSWindow) };
    child.setMovable(false);
    // Ordering a child out can clear the relationship, so it is restored on
    // every presentation rather than only the first.
    if child.parentWindow().is_none() {
      unsafe { parent.addChildWindow_ordered(child, NSWindowOrderingMode::Above) };
    }
  })
}

/// Detaches before hiding: ordering a still-attached child out drags its
/// parent with it.
#[cfg(target_os = "macos")]
pub(super) fn detach(
  app: &AppHandle,
  editor: &WebviewWindow,
  options: &WebviewWindow,
) -> tauri::Result<()> {
  let parent = editor.ns_window()? as usize;
  let child = options.ns_window()? as usize;
  on_main_thread(app, move || {
    use objc2_app_kit::NSWindow;

    let parent: &NSWindow = unsafe { &*(parent as *const NSWindow) };
    let child: &NSWindow = unsafe { &*(child as *const NSWindow) };
    if child.parentWindow().is_some() {
      parent.removeChildWindow(child);
    }
  })
}

/// Takes the window's pixels away without taking the window away, so it can be
/// ordered front and focused at the wrong size unseen.
///
/// Applied on the main thread without leaving it, so that by the time the
/// caller reaches `windows::show` the alpha is already zero: tao runs
/// `makeKeyAndOrderFront:` through `run_on_main`, which is likewise immediate
/// on the main thread.
#[cfg(target_os = "macos")]
pub(super) fn conceal(app: &AppHandle, options: &WebviewWindow) -> tauri::Result<()> {
  let window = options.ns_window()? as usize;
  on_main_thread(app, move || {
    use objc2_app_kit::NSWindow;

    let window: &NSWindow = unsafe { &*(window as *const NSWindow) };
    window.setAlphaValue(0.0);
  })?;
  // The webview normally reveals the window itself once it has fitted the
  // height. Should it never get to (a script error, a webview that failed to
  // come up), a window at the wrong size still beats an invisible one.
  let deadline = dispatch2::DispatchTime::try_from(std::time::Duration::from_secs(1))
    .unwrap_or(dispatch2::DispatchTime::NOW);
  let _ = dispatch2::DispatchQueue::main().after(deadline, move || {
    use objc2_app_kit::NSWindow;

    let window: &NSWindow = unsafe { &*(window as *const NSWindow) };
    if window.isVisible() {
      window.setAlphaValue(1.0);
    }
  });
  Ok(())
}

/// Brings the window back to full alpha, but only once the resize asked for
/// immediately before it has actually landed.
///
/// tao 0.35.3 does not call `setContentSize:` inside `set_inner_size`: it hands
/// the call to `DispatchQueue::main().exec_async` (see
/// `platform_impl/macos/util/async.rs::set_content_size_async`), so on return
/// the window is still the height it was. Queueing the alpha restore on that
/// same queue is therefore the one ordering that is guaranteed - the main
/// dispatch queue is serial, so this block cannot outrun the block that
/// resizes. Neither alternative holds: `run_on_main_thread` posts through the
/// tao event loop, a separate channel with no ordering relationship to the
/// dispatch queue, and `on_main_thread` above would run the restore outright,
/// ahead of the resize, whenever the caller is already on the main thread -
/// which a synchronous command always is.
///
/// The pointer outlives the hop because these windows are never destroyed:
/// closing one hides it (see `initialize`).
#[cfg(target_os = "macos")]
pub(super) fn reveal_after_resize(options: &WebviewWindow) -> tauri::Result<()> {
  let window = options.ns_window()? as usize;
  dispatch2::DispatchQueue::main().exec_async(move || {
    use objc2_app_kit::NSWindow;

    let window: &NSWindow = unsafe { &*(window as *const NSWindow) };
    window.setAlphaValue(1.0);
  });
  Ok(())
}

// TODO(windows): parent the options window to its editor so the pair orders
// and minimises together, as `addChildWindow:` already does on macOS.
#[cfg(not(target_os = "macos"))]
pub(super) fn attach(
  _app: &AppHandle,
  _editor: &WebviewWindow,
  _options: &WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(super) fn detach(
  _app: &AppHandle,
  _editor: &WebviewWindow,
  _options: &WebviewWindow,
) -> tauri::Result<()> {
  Ok(())
}

// TODO(windows): hold the window back the same way. `WS_EX_LAYERED` alpha is
// available (see `windows::platform::set_opacity`) but has to be reconciled
// with the window's own transparency before it can be used here, so until then
// Windows shows the configured height and takes the jump.
#[cfg(not(target_os = "macos"))]
pub(super) fn conceal(_app: &AppHandle, _options: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}

#[cfg(not(target_os = "macos"))]
pub(super) fn reveal_after_resize(_options: &WebviewWindow) -> tauri::Result<()> {
  Ok(())
}
