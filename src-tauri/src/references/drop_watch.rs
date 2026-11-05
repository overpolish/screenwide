// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Watching for a picture dragged with Option or Alt held (`references::drop`),
//! the same on every platform. A press anywhere on the desktop starts a
//! short watch for as long as the button is held: once a drag has begun from
//! it with the modifier held, a catcher is laid over each display; letting
//! the modifier go takes them away again, so the drop lands where it would
//! have. The hint island comes out once the drag is known to carry a picture:
//! with the catchers where the platform can see what a drag carries (macOS),
//! else as one reaches a catcher (Windows), which steps aside for anything
//! else. The platform notices presses, reads the pointer and keys, and puts
//! the catchers up (`drop_macos`, `drop_windows`). Everything here runs on
//! the main thread.

use std::cell::RefCell;
use std::time::Duration;

use tauri::{AppHandle, Position};

use super::drop::Dropped;
#[cfg(target_os = "macos")]
use super::drop_macos as platform;
#[cfg(target_os = "windows")]
use super::drop_windows as platform;
use platform::{Catchers, Mark};

/// How often the pointer is looked at while a button is held.
const POLL: Duration = Duration::from_millis(33);
/// How long the catchers stay after the button is let go, for a drop on
/// them to arrive.
const LINGER: Duration = Duration::from_millis(400);

struct Watch {
  app: AppHandle,
  /// The press under way, as the platform measures a drag begun from it.
  pressed: Option<Mark>,
  catchers: Option<Catchers>,
  /// Whether the drag is known to carry a picture, and the hint is out.
  picture: bool,
  /// A catcher saw the drag carry something else; they stay away until the
  /// next press.
  refused: bool,
}

thread_local! {
  static WATCH: RefCell<Option<Watch>> = const { RefCell::new(None) };
}

/// Starts watching for presses. Main thread only.
pub(crate) fn start(app: &AppHandle) {
  WATCH.with(|watch| {
    *watch.borrow_mut() = Some(Watch {
      app: app.clone(),
      pressed: None,
      catchers: None,
      picture: false,
      refused: false,
    });
  });
  platform::listen(app);
}

/// The button went down somewhere on the desktop.
pub(super) fn pressed() {
  let app = WATCH.with(|watch| {
    let mut watch = watch.borrow_mut();
    let watch = watch.as_mut()?;
    if watch.pressed.is_some() {
      return None;
    }
    watch.pressed = Some(platform::mark());
    watch.refused = false;
    Some(watch.app.clone())
  });
  let Some(app) = app else {
    return;
  };
  std::thread::spawn(move || loop {
    std::thread::sleep(POLL);
    let (sender, receiver) = std::sync::mpsc::sync_channel(1);
    let looked = app.run_on_main_thread(move || {
      let _ = sender.send(look());
    });
    if looked.is_err() || !receiver.recv().unwrap_or(false) {
      break;
    }
  });
}

/// Puts the catchers up or away as the press stands. Answers whether the
/// button is still held, so the watch goes on.
fn look() -> bool {
  let held = platform::held();
  let modifier = platform::modifier();
  WATCH.with(|watch| {
    let mut watch = watch.borrow_mut();
    let Some(watch) = watch.as_mut() else {
      return false;
    };
    let dragging = !watch.refused && watch.pressed.is_some_and(platform::dragging);
    if held && modifier && dragging {
      if watch.catchers.is_none() {
        watch.catchers = Some(Catchers::put_up());
        if platform::SEES_PICTURE_BEFORE_CATCHERS {
          show_hint(watch);
        }
      }
    } else if watch.catchers.is_some() && held {
      put_away(watch);
    }
    if !held {
      watch.pressed = None;
      if watch.picture {
        let app = watch.app.clone();
        std::thread::spawn(move || {
          std::thread::sleep(LINGER);
          let _ = app.run_on_main_thread(|| with_watch(put_away));
        });
      } else {
        put_away(watch);
      }
    }
    held
  })
}

fn with_watch(work: impl FnOnce(&mut Watch)) {
  WATCH.with(|watch| {
    if let Some(watch) = watch.borrow_mut().as_mut() {
      work(watch);
    }
  });
}

fn show_hint(watch: &mut Watch) {
  watch.picture = true;
  super::drop_island::show(&watch.app);
}

fn put_away(watch: &mut Watch) {
  if let Some(catchers) = watch.catchers.take() {
    catchers.put_away();
  }
  if std::mem::take(&mut watch.picture) {
    super::drop_island::withdraw(&watch.app);
  }
}

/// A catcher has the drag over it, carrying a picture or, for `false`,
/// something else, which the catchers step aside for.
#[cfg(target_os = "windows")]
pub(super) fn over_catcher(picture: bool) {
  with_watch(|watch| {
    if picture {
      if !watch.picture {
        show_hint(watch);
      }
    } else {
      watch.refused = true;
      put_away(watch);
    }
  });
}

/// A picture was dropped on a catcher at `at`, on a display of `scale`
/// pixels to the point.
pub(super) fn dropped(forms: Vec<Dropped>, at: Position, scale: f64) {
  with_watch(|watch| {
    put_away(watch);
    super::drop::keep(&watch.app, forms, at, scale);
  });
}
