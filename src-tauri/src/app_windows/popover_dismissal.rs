// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Putting transient popovers away when the user presses elsewhere.

use tauri::AppHandle;

use super::{options, source_selector};

#[derive(Clone, Copy, Default)]
struct PopoversOpenOnPress {
  source_selector: bool,
  standalone_listbox: bool,
}

impl PopoversOpenOnPress {
  fn capture() -> Self {
    Self {
      source_selector: source_selector::is_expanded(),
      standalone_listbox: options::is_standalone_listbox_open(),
    }
  }
}

/// Which popover an outside press is being answered for. A pop-up panel goes
/// on the press itself, the way a native menu does, so dragging the window or
/// its edge does not leave it hanging until release; its trigger is excluded
/// by its anchor. The source selector waits for release: it has no anchor, and
/// its trigger toggles on release, which would open it straight back up.
#[derive(Clone, Copy)]
enum OutsidePress {
  Listbox,
  SourceSelector,
}

impl OutsidePress {
  fn dismiss(self, app: &AppHandle, x: f64, y: f64) {
    match self {
      Self::Listbox => options::dismiss_standalone_listbox_if_outside(app, x, y),
      Self::SourceSelector => source_selector::dismiss_if_outside(app, x, y),
    }
  }
}

#[cfg(target_os = "windows")]
pub fn manage_transient_popover_dismissal(app: &AppHandle) {
  use std::sync::mpsc;

  use rdev::{listen, Button, EventType};

  let (dismiss_tx, dismiss_rx) = mpsc::channel::<(OutsidePress, f64, f64)>();
  let dismiss_app = app.clone();
  std::thread::spawn(move || {
    while let Ok((press, x, y)) = dismiss_rx.recv() {
      press.dismiss(&dismiss_app, x, y);
    }
  });
  std::thread::spawn(move || {
    let mut position = (0.0, 0.0);
    let mut open_on_press = PopoversOpenOnPress::default();
    // rdev invokes this callback before CallNextHookEx. Dismissal is deferred
    // because its window geometry queries synchronously wait for the UI
    // thread, which cannot process them while the hook callback is live.
    let result = listen(move |event| match event.event_type {
      EventType::MouseMove { x, y } => {
        position = (x, y);
      }
      EventType::ButtonPress(Button::Left) => {
        open_on_press = PopoversOpenOnPress::capture();
        if open_on_press.standalone_listbox {
          let (x, y) = position;
          let _ = dismiss_tx.send((OutsidePress::Listbox, x, y));
        }
      }
      EventType::ButtonRelease(Button::Left) => {
        if open_on_press.source_selector {
          let (x, y) = position;
          let _ = dismiss_tx.send((OutsidePress::SourceSelector, x, y));
        }
        open_on_press = PopoversOpenOnPress::default();
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
    let mut open_on_press = PopoversOpenOnPress::default();

    loop {
      let is_pressed = EventSrcState::CombinedSession.button_state(MouseButton::Left);
      if was_pressed != is_pressed {
        let Some(event) = Event::with_src(None) else {
          break;
        };
        let position = event.location();
        if is_pressed {
          open_on_press = PopoversOpenOnPress::capture();
          if open_on_press.standalone_listbox {
            OutsidePress::Listbox.dismiss(&app, position.x, position.y);
          }
        } else {
          if open_on_press.source_selector {
            OutsidePress::SourceSelector.dismiss(&app, position.x, position.y);
          }
          open_on_press = PopoversOpenOnPress::default();
        }
      }

      was_pressed = is_pressed;
      std::thread::sleep(std::time::Duration::from_millis(8));
    }
  });
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
pub fn manage_transient_popover_dismissal(_app: &AppHandle) {}
