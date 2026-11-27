// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use std::sync::{Arc, Mutex, PoisonError};

use super::platform;

/// Keeps the display on until dropped, while the screen is unlocked. Locking
/// says the user has left, so the display may sleep then; it is held again
/// on unlock.
pub struct ScreenOn {
  // Declared first so it drops first: once the watch has stopped, nothing
  // takes the hold again behind it.
  _watch: platform::LockWatch,
  _hold: Arc<Mutex<Option<platform::Hold>>>,
}

impl ScreenOn {
  /// `reason` is what the system lists as keeping the display on. It starts
  /// held: it is turned on from the tray, a shortcut, or at login, none of
  /// which reach a locked screen.
  pub fn start(reason: &str) -> Result<Self, String> {
    let hold = Arc::new(Mutex::new(Some(platform::Hold::display(reason)?)));
    let held = Arc::clone(&hold);
    let reason = reason.to_owned();
    let watch = platform::LockWatch::start(move |locked| {
      let mut held = held.lock().unwrap_or_else(PoisonError::into_inner);
      if locked {
        held.take();
      } else if held.is_none() {
        match platform::Hold::display(&reason) {
          Ok(hold) => *held = Some(hold),
          Err(error) => eprintln!("{error}"),
        }
      }
    })?;
    Ok(Self {
      _watch: watch,
      _hold: hold,
    })
  }
}
