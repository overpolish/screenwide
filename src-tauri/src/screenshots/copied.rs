// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The island once a screenshot is on the clipboard: Copied at the row's
//! start, Save and Edit at its end, and the picture hanging beneath. It stays
//! out for a few seconds, kept there while the pointer is over it. Edit opens
//! the shot in the editor and Save keeps it in Projects without opening it,
//! each as a project; left to go, nothing more is made of it, and the shot is
//! on the clipboard alone.
//!
//! The island (`src/features/screenshots/island`) counts the time down, as it
//! is what knows where the pointer is, and says when it is done. Only the
//! latest shot is held: a new one takes its place.

mod thumbnail;

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{Mutex, MutexGuard, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter};
use ts_rs::TS;

use screenwide_annotations::Annotation;

use super::CapturedImage;
use crate::app_windows::island_placement::IslandPlacement;
use crate::app_windows::{self, WindowLabel};

const EVENT: &str = "copied-screenshot://island";

/// A shot just copied, as the shutter hands it over.
pub(crate) struct Shot {
  pub annotations: Vec<Annotation>,
  /// What went to the clipboard: the shot with its annotations drawn in.
  pub copied: CapturedImage,
  pub file_stem: String,
  /// The shot itself, which a project gets with its annotations as a layer.
  pub image: CapturedImage,
  /// The display scale it was taken at.
  pub scale: f64,
}

/// What the island shows.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CopiedScreenshot {
  /// Tells one shot from the next, so what the island asks of a shot already
  /// replaced is ignored.
  pub id: u32,
  /// The picture, scaled down, as a file the island loads.
  pub path: String,
  /// The size the picture is shown at, in points.
  pub width: f64,
  pub height: f64,
  /// Whether Save has kept it in Projects, which leaves nothing more to do.
  pub saved: bool,
}

/// What a project of the shot is made from, until one is made.
struct Kept {
  annotations: Vec<Annotation>,
  file_stem: String,
  image: CapturedImage,
  scale: f64,
}

struct Held {
  preview: CopiedScreenshot,
  kept: Option<Kept>,
}

static HELD: Mutex<Option<Held>> = Mutex::new(None);
/// A shot copied whose picture is still being made, so the island is kept
/// out for it as the screenshot session ends.
static COMING: AtomicBool = AtomicBool::new(false);
static NEXT_ID: AtomicU32 = AtomicU32::new(1);

fn held() -> MutexGuard<'static, Option<Held>> {
  HELD.lock().unwrap_or_else(PoisonError::into_inner)
}

fn tell(app: &AppHandle, preview: Option<CopiedScreenshot>) {
  // Advisory: the shot is on the clipboard whether or not the island heard.
  let _ = app.emit_to(WindowLabel::Island.as_str(), EVENT, preview);
}

/// Whether the island is out, or coming out, for a copied shot, so the Quick
/// Screenshot island it follows leaves the window to it.
pub(crate) fn is_up() -> bool {
  COMING.load(Ordering::Acquire) || held().is_some()
}

/// Brings the island out with `shot`. The shot is on the clipboard already,
/// so an island that cannot show it is only reported.
pub(crate) fn show(app: &AppHandle, shot: Shot) {
  COMING.store(true, Ordering::Release);
  // Before the island changes, so a recording under way never takes a frame
  // of the shot.
  app_windows::island_overlay::keep_still_out_of_recording(app, true);
  let app = app.clone();
  tauri::async_runtime::spawn(async move {
    let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
    let making_app = app.clone();
    let made = tauri::async_runtime::spawn_blocking(move || {
      let directory = thumbnail::directory(&making_app)?;
      let (path, width, height) = thumbnail::write(&directory, id, &shot.copied, shot.scale)?;
      Ok::<Held, String>(Held {
        preview: CopiedScreenshot {
          id,
          path: path.to_string_lossy().into_owned(),
          width,
          height,
          saved: false,
        },
        kept: Some(Kept {
          annotations: shot.annotations,
          file_stem: shot.file_stem,
          image: shot.image,
          scale: shot.scale,
        }),
      })
    })
    .await
    .map_err(|error| error.to_string())
    .and_then(|made| made);
    let next = match made {
      Ok(next) => next,
      Err(error) => {
        COMING.store(false, Ordering::Release);
        super::quick_island::withdraw(&app);
        step_back(&app);
        return eprintln!("Could not show the copied screenshot: {error}");
      }
    };
    let preview = next.preview.clone();
    if let Some(replaced) = held().replace(next) {
      thumbnail::remove(&replaced.preview.path);
    }
    COMING.store(false, Ordering::Release);
    tell(&app, Some(preview));
    super::quick_island::give_way(&app);
    crate::app_windows::sync_recording_ui_escape(&app, crate::ruler::is_active(&app));
    let shown = app_windows::island::place(&app, IslandPlacement::Top)
      .and_then(|()| app_windows::island::show(&app));
    if let Err(error) = shown {
      eprintln!("Could not show the island for the copied screenshot: {error}");
    }
  });
}

/// The island gives the copied shot up: it tucks away, unless Quick
/// Screenshot has it now or something else on the island keeps it
/// (`quick_island::keeps_island`), which takes it back where it shows.
fn step_back(app: &AppHandle) {
  if super::quick_island::is_up() || is_up() {
    return;
  }
  let result = if super::quick_island::keeps_island(app) {
    app_windows::island::hand_over(app, IslandPlacement::Top)
  } else {
    app_windows::island::hide(app)
  };
  if let Err(error) = result {
    eprintln!("Could not put the island away after the copied screenshot: {error}");
  }
  app_windows::island_overlay::keep_still_out_of_recording(app, false);
}

/// Lets the shot held as `id` go, if it is still the one held, and the
/// island with it; or whichever is held when `id` is none.
fn release(app: &AppHandle, id: Option<u32>) -> Option<Held> {
  let released = {
    let mut held = held();
    if id.is_some_and(|id| held.as_ref().is_some_and(|held| held.preview.id != id)) {
      return None;
    }
    held.take()?
  };
  thumbnail::remove(&released.preview.path);
  tell(app, None);
  crate::app_windows::sync_recording_ui_escape(app, crate::ruler::is_active(app));
  step_back(app);
  Some(released)
}

/// A new shot is being taken, which takes the island's place.
pub(crate) fn withdraw(app: &AppHandle) {
  release(app, None);
}

/// The shot the island shows, read as its page loads.
#[tauri::command]
pub fn copied_screenshot() -> Option<CopiedScreenshot> {
  held().as_ref().map(|held| held.preview.clone())
}

/// Shot `id`'s time is up: it goes, and is on the clipboard alone.
#[tauri::command]
pub fn dismiss_copied_screenshot(app: AppHandle, id: u32) {
  release(&app, Some(id));
}

/// Edit: shot `id` opens in the editor, kept out of Projects until the
/// editor's Save keeps it there.
#[tauri::command]
pub async fn edit_copied_screenshot(app: AppHandle, id: u32) -> Result<(), String> {
  let Some(kept) = release(&app, Some(id)).and_then(|released| released.kept) else {
    return Ok(());
  };
  crate::editor::reserve_screenshot_workspace(&app)?;
  crate::editor::present_screenshot(
    &app,
    kept.image,
    kept.scale,
    kept.annotations,
    kept.file_stem,
    true,
  )
  .inspect_err(|_| crate::editor::release_screenshot_workspace(&app))
}

/// Save: shot `id` is kept in Projects without opening it. The island says
/// so until its time is up.
#[tauri::command]
pub async fn save_copied_screenshot(app: AppHandle, id: u32) -> Result<(), String> {
  let kept = {
    let mut held = held();
    match held.as_mut() {
      Some(held) if held.preview.id == id => held.kept.take(),
      _ => None,
    }
  };
  let Some(kept) = kept else {
    return Ok(());
  };
  let saving_app = app.clone();
  tauri::async_runtime::spawn_blocking(move || {
    crate::editor::save_screenshot(
      &saving_app,
      kept.image,
      kept.scale,
      kept.annotations,
      &kept.file_stem,
    )
  })
  .await
  .map_err(|error| error.to_string())??;
  let preview = {
    let mut held = held();
    let Some(held) = held.as_mut().filter(|held| held.preview.id == id) else {
      return Ok(());
    };
    held.preview.saved = true;
    held.preview.clone()
  };
  tell(&app, Some(preview));
  Ok(())
}
