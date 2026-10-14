// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! Each caption style's preview, drawn as the export draws it and kept on
//! disk: an animated PNG the template tiles play, and one still frame a tile
//! shows at once while the animation loads.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::sync::{LazyLock, Mutex};

use screenwide_captions::CaptionStyle;
use screenwide_compositor::compositor::caption_preview::{CaptionPreview, CaptionPreviewer};
use screenwide_imaging::CapturedImage;
use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

/// A preview's frames, in pixels and per second: a tile shows them at half
/// their size, sharp on a high-density display.
const PREVIEW: CaptionPreview = CaptionPreview {
  width: 240,
  height: 135,
  frames_per_second: 20.0,
};
/// Changed whenever the preview is drawn differently, so no older clip is
/// shown in its place.
const PREVIEW_VERSION: u32 = 14;
/// Which frame stands for the whole preview: one a little way in, with a
/// caption up and a word being said.
const STILL_SHARE: f64 = 0.3;
/// What a clip's still adds to the clip's name.
const STILL_SUFFIX: &str = "-still.png";

/// A style's preview: an animated PNG that plays over and over, starting
/// from its still, and that still on its own, which a tile shows until the
/// animation is ready, or for good where motion is reduced.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionPreviewClip {
  pub path: PathBuf,
  pub still: PathBuf,
  pub width: u32,
  pub height: u32,
}

/// The compositor previews are drawn with, made once: making one compiles
/// its pipelines, which takes longer than drawing every template's preview.
/// [`warm`] makes it at launch.
static PREVIEWER: LazyLock<Result<Mutex<CaptionPreviewer>, String>> = LazyLock::new(|| {
  Ok(Mutex::new(
    CaptionPreviewer::new(screenwide_gpu::shared()?)?,
  ))
});

/// Draws each of `styles`' preview of the `sample` words, so it is only
/// found when the panel asks. A preview drawn before is only found, and the
/// drawing readies the previewer.
pub(crate) fn warm(app: &AppHandle, styles: &[CaptionStyle], sample: &str) {
  // A preview that cannot be drawn now is drawn when the panel asks.
  let _ = previews(app, styles, sample);
}

/// Each of `styles` previewed with the `sample` words, in order, drawn
/// where never drawn before; then lets go of the clips least recently used.
fn previews(
  app: &AppHandle,
  styles: &[CaptionStyle],
  sample: &str,
) -> Result<Vec<CaptionPreviewClip>, String> {
  let directory = preview_directory(app)?;
  let clips = styles
    .iter()
    .map(|style| preview_clip(&directory, style, sample))
    .collect();
  prune_previews(&directory);
  clips
}

/// Where the previews live: one clip and its still per style and sample,
/// named by a hash of both, so a style already drawn is never drawn again.
fn preview_directory(app: &AppHandle) -> Result<PathBuf, String> {
  let directory = app
    .path()
    .app_cache_dir()
    .map_err(|error| format!("The caption preview directory is not available: {error}"))?
    .join("caption-previews");
  std::fs::create_dir_all(&directory)
    .map_err(|error| format!("The caption preview directory could not be made: {error}"))?;
  Ok(directory)
}

/// The still that goes with the clip at `clip`.
fn still_path(clip: &Path) -> PathBuf {
  let stem = clip.file_stem().unwrap_or_default().to_string_lossy();
  clip.with_file_name(format!("{stem}{STILL_SUFFIX}"))
}

/// `style`'s preview of the `sample` words, drawn the first time it is
/// asked for and kept.
fn preview_clip(
  directory: &Path,
  style: &CaptionStyle,
  sample: &str,
) -> Result<CaptionPreviewClip, String> {
  let backdrop = style.template.preview_backdrop();
  let description = serde_json::to_string(style).map_err(|error| error.to_string())?;
  let mut hasher = DefaultHasher::new();
  // The backdrop is named apart, as a manifest can change it without the
  // style.
  (PREVIEW_VERSION, description, sample, backdrop).hash(&mut hasher);
  let path = directory.join(format!("{:016x}.png", hasher.finish()));
  let clip = CaptionPreviewClip {
    still: still_path(&path),
    path,
    width: PREVIEW.width,
    height: PREVIEW.height,
  };
  if clip.path.is_file() && clip.still.is_file() {
    // Marked as used, so pruning keeps the clips the panel shows.
    if let Ok(file) = std::fs::File::options().append(true).open(&clip.path) {
      let _ = file.set_modified(std::time::SystemTime::now());
    }
    return Ok(clip);
  }
  let words: Vec<&str> = sample.split_whitespace().collect();
  let mut frames = PREVIEWER
    .as_ref()
    .map_err(Clone::clone)?
    .lock()
    .map_err(|_| "The caption previewer is poisoned".to_owned())?
    .frames(style, &words, &PREVIEW, backdrop)?;
  let still = (frames.len().saturating_sub(1) as f64 * STILL_SHARE).round() as usize;
  // The clip loops, so starting it at the still lets a tile go from the
  // still to the animation without a jump.
  frames.rotate_left(still);
  let first = frames
    .first()
    .ok_or_else(|| "The caption preview drew no frames".to_owned())?;
  let picture = CapturedImage {
    width: PREVIEW.width,
    height: PREVIEW.height,
    rgba: first.clone(),
  };
  save(
    &clip.still,
    &screenwide_output::encoding::encode_truecolor_png(&picture)?,
  )?;
  let frame_ms = (1_000.0 / PREVIEW.frames_per_second).round() as u16;
  let animation = screenwide_output::encoding::encode_animated_png(
    &frames,
    PREVIEW.width,
    PREVIEW.height,
    frame_ms,
  )?;
  // The clip is saved last, so a clip on disk always has its still.
  save(&clip.path, &animation)?;
  Ok(clip)
}

/// Saves `bytes` at `path`, written whole under another name first, so a
/// file half written when the app stops is never taken for a finished one.
fn save(path: &Path, bytes: &[u8]) -> Result<(), String> {
  let partial = path.with_extension("partial");
  std::fs::write(&partial, bytes)
    .and_then(|()| std::fs::rename(&partial, path))
    .map_err(|error| format!("The caption preview could not be saved: {error}"))
}

/// More clips than the panel ever shows at once, with room for the styles
/// tried on the way to one: the oldest past this are let go.
const KEPT_PREVIEWS: usize = 60;

/// Lets go of every clip in `directory` but the [`KEPT_PREVIEWS`] most
/// recently used, each with its still, and of any still whose clip is gone.
/// A preview let go is drawn again if it is asked for.
fn prune_previews(directory: &Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  let mut clips = Vec::new();
  let mut stills = Vec::new();
  for entry in entries.flatten() {
    let path = entry.path();
    if path.to_string_lossy().ends_with(STILL_SUFFIX) {
      stills.push(path);
    } else if let Some(modified) = entry.metadata().ok().and_then(|meta| meta.modified().ok()) {
      clips.push((modified, path));
    }
  }
  for still in stills {
    let name = still.file_name().unwrap_or_default().to_string_lossy();
    let clip = still.with_file_name(format!("{}.png", name.trim_end_matches(STILL_SUFFIX)));
    if !clip.is_file() {
      let _ = std::fs::remove_file(&still);
    }
  }
  if clips.len() <= KEPT_PREVIEWS {
    return;
  }
  clips.sort_unstable_by_key(|clip| std::cmp::Reverse(clip.0));
  for (_, path) in &clips[KEPT_PREVIEWS..] {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(still_path(path));
  }
}

/// Each of `styles` previewed with the `sample` words, in order: a clip
/// drawn by the compositor, so a template's tile is true to what the export
/// draws.
#[tauri::command]
pub async fn render_caption_previews(
  app: AppHandle,
  styles: Vec<CaptionStyle>,
  sample: String,
) -> Result<Vec<CaptionPreviewClip>, String> {
  tauri::async_runtime::spawn_blocking(move || previews(&app, &styles, &sample))
    .await
    .map_err(|error| error.to_string())?
}

/// The previews launch draws, of every template as new captions would take
/// it up, so the panel window can load them before the panel first opens.
#[tauri::command]
pub async fn launch_caption_previews(
  app: AppHandle,
  sample: String,
) -> Result<Vec<CaptionPreviewClip>, String> {
  tauri::async_runtime::spawn_blocking(move || {
    previews(&app, &super::caption_styles::launch_styles(&app), &sample)
  })
  .await
  .map_err(|error| error.to_string())?
}
