// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Putting the pop-up lists away when the user presses elsewhere: on the
//! press itself, the way a native menu does, so dragging a window or its edge
//! does not leave one hanging until release. A list's trigger is excluded by
//! its anchor.

use tauri::AppHandle;

use super::options;

#[cfg(target_os = "windows")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use std::sync::mpsc;

  use rdev::{listen, Button, EventType};

  let (dismiss_tx, dismiss_rx) = mpsc::channel::<(f64, f64)>();
  let dismiss_app = app.clone();
  std::thread::spawn(move || {
    while let Ok((x, y)) = dismiss_rx.recv() {
      options::dismiss_standalone_listbox_if_outside(&dismiss_app, x, y);
    }
  });
  std::thread::spawn(move || {
    let mut position = (0.0, 0.0);
    // rdev invokes this callback before CallNextHookEx. Dismissal is deferred
    // because its window geometry queries synchronously wait for the UI
    // thread, which cannot process them while the hook callback is live.
    let result = listen(move |event| match event.event_type {
      EventType::MouseMove { x, y } => {
        position = (x, y);
      }
      EventType::ButtonPress(Button::Left) => {
        if options::is_standalone_listbox_open() {
          let _ = dismiss_tx.send(position);
        }
      }
      _ => {}
    });

    if let Err(error) = result {
      eprintln!("Could not monitor clicks for transient popover dismissal: {error:?}");
    }
  });
}

#[cfg(target_os = "macos")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use cidre::cg::{Event, EventSrcState, MouseButton};

  let app = app.clone();
  std::thread::spawn(move || {
    let mut was_pressed = EventSrcState::CombinedSession.button_state(MouseButton::Left);

    loop {
      let is_pressed = EventSrcState::CombinedSession.button_state(MouseButton::Left);
      if is_pressed && !was_pressed && options::is_standalone_listbox_open() {
        let Some(event) = Event::with_src(None) else {
          break;
        };
        let position = event.location();
        options::dismiss_standalone_listbox_if_outside(&app, position.x, position.y);
      }

      was_pressed = is_pressed;
      std::thread::sleep(std::time::Duration::from_millis(8));
    }
  });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn manage_transient_popover_dismissal(_app: &AppHandle) {}
