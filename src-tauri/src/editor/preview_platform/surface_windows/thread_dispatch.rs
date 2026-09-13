// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use super::*;

/// Creates the native input child window on the thread that owns the host
/// HWND. Win32 queues a window's messages on its creating thread, and only the
/// event-loop thread pumps them, so an editor created on a worker thread
/// (Tauri's blocking pool, an IPC command thread) would never receive a mouse
/// message and every native gesture would be dead.
pub(super) fn create_editor_on_owning_thread(
  window: &WebviewWindow,
  host: HWND,
) -> Result<editor::EditorWindow, String> {
  if unsafe { GetWindowThreadProcessId(host, None) } == unsafe { GetCurrentThreadId() } {
    // Already on the event-loop thread: create inline. Dispatching and then
    // blocking for the reply here would deadlock tao's loop against itself.
    return editor::EditorWindow::new(host);
  }
  // `HWND` is a raw pointer and so not `Send`; a window handle is just an
  // opaque process-wide token, safe to hand to the thread that owns it.
  struct HostHandle(HWND);
  unsafe impl Send for HostHandle {}

  let handle = HostHandle(host);
  let (sender, receiver) = std::sync::mpsc::channel();
  window
    .run_on_main_thread(move || {
      let handle = handle;
      let _ = sender.send(editor::EditorWindow::new(handle.0));
    })
    .map_err(|error| format!("The Windows preview editor could not be dispatched: {error}"))?;
  receiver
    .recv()
    .map_err(|_| "The Windows preview editor was never created on the main thread".to_owned())?
}
