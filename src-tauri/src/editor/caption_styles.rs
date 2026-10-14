// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What the Captions panel offers to style captions with: the templates as
//! they ship, the font families installed on the system, and each style's
//! preview, drawn as the export draws it.

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;
use std::sync::{LazyLock, Mutex};

use screenwide_annotations::text::app_fonts::{add_app_fonts, AppFamily};
use screenwide_annotations::text::installed::{
  system_families, with_app_families, InstalledFamily,
};
use screenwide_captions::{
  CaptionLayout, CaptionStyle, CaptionTemplate, CaptionTemplateId, RecordingCaptions,
};
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
/// Changed whenever the preview is drawn differently, so no older strip is
/// shown in its place.
const PREVIEW_VERSION: u32 = 11;

/// A style's preview: a strip of frames one above another in one picture,
/// which the tile steps through.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionPreviewStrip {
  pub path: PathBuf,
  pub width: u32,
  pub height: u32,
  pub frames: u32,
  pub frames_per_second: f64,
}
/// A font family installed on the system or brought by a template, the
/// weights it has upright on the CSS scale of 100 to 900, lightest first,
/// and whether it has an italic.
#[derive(Clone, Debug, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct FontFamily {
  pub name: String,
  pub weights: Vec<u16>,
  pub italic: bool,
}

/// Every caption template, the app's own first in the order the panel
/// offers them, then yours: each with what its manifest says of it. A
/// template saved as a default in the general settings takes the place of
/// its own style in the panel.
#[tauri::command]
pub fn caption_templates() -> Vec<CaptionTemplate> {
  screenwide_captions::caption_templates()
}

/// Where your own templates are kept, a folder each.
pub(crate) fn user_templates_folder(app: &AppHandle) -> Option<PathBuf> {
  app
    .path()
    .app_data_dir()
    .ok()
    .map(|data| data.join("caption-templates"))
}

/// Reads your own templates, logging any left out and why: a template
/// that cannot be read never stops the app. Then hands every template's
/// fonts to the text engines, so captions are set in them from now on.
fn load_user_templates(app: &AppHandle) {
  if let Some(folder) = user_templates_folder(app) {
    for problem in screenwide_captions::load_user_templates(&folder) {
      eprintln!("A caption template was left out: {problem}");
    }
  }
  hand_over_fonts();
}

/// Hands every template's fonts to the text engines. They keep a font
/// handed over twice once, so this may run each time templates are read.
pub(crate) fn hand_over_fonts() {
  let fonts = screenwide_captions::template_font_files();
  let families: Vec<AppFamily> = fonts
    .iter()
    .map(|(font, _)| AppFamily {
      name: font.family.clone(),
      weights: font.weights.clone(),
      italic: font.italic,
    })
    .collect();
  let files: Vec<&[u8]> = fonts.iter().map(|(_, bytes)| bytes.as_ref()).collect();
  add_app_fonts(&families, &files);
}

/// The font families installed on the system, read once: reading every
/// family's members takes half a second, and the panel asks each time it
/// opens. [`warm`] reads them at launch, so the panel never waits.
static SYSTEM_FAMILIES: LazyLock<Vec<InstalledFamily>> = LazyLock::new(system_families);

/// The compositor previews are drawn with, made once: making one compiles
/// its pipelines, which takes longer than drawing every template's preview.
/// [`warm`] makes it at launch.
static PREVIEWER: LazyLock<Result<Mutex<CaptionPreviewer>, String>> = LazyLock::new(|| {
  Ok(Mutex::new(
    CaptionPreviewer::new(screenwide_gpu::shared()?)?,
  ))
});

/// Readies the Captions panel off the main thread, so it has everything the
/// moment it opens: reads the font families, and draws each template's
/// preview as the panel will ask for it, the way `useCaptionTemplates` and
/// the tiles lay them out: its saved look, with the layout options kept for
/// new captions. A preview drawn before is only found, and the drawing
/// readies the previewer.
pub(crate) fn warm(app: &AppHandle) {
  // Read before anything asks for the templates, which takes a moment for
  // a few small files.
  load_user_templates(app);
  let app = app.clone();
  std::thread::spawn(move || {
    LazyLock::force(&SYSTEM_FAMILIES);
    let Ok(directory) = preview_directory(&app) else {
      return;
    };
    let settings = crate::settings::current(&app);
    let sample = screenwide_i18n::t!("editor-panels-captions-sample");
    for template in screenwide_captions::caption_templates() {
      let style = settings
        .caption_layout
        .applied_to(template_style(&settings.caption_styles, &template.id));
      // A preview that cannot be drawn now is drawn when the panel asks.
      let _ = preview_strip(&directory, &style, &sample);
    }
    prune_previews(&directory);
  });
}

/// `template` as it was last set as default, or as it ships: a saved look,
/// over the layout options it ships with, which only the layout set by hand
/// changes.
fn template_style(saved: &[CaptionStyle], template: &CaptionTemplateId) -> CaptionStyle {
  let shipped = template.shipped_style();
  let look = saved
    .iter()
    .find(|style| style.template == *template)
    .cloned()
    .unwrap_or_else(|| shipped.clone());
  CaptionLayout::of(&shipped).applied_to(look)
}

/// What new captions start as, as `recordingCaptions` gives the editor: the
/// default template, as saved where it was saved, with the layout options
/// kept for new captions, saying what the default rules have it say. Off
/// until they are turned on.
pub(crate) fn default_captions(app: &AppHandle) -> RecordingCaptions {
  let settings = crate::settings::current(app);
  let style = template_style(&settings.caption_styles, &settings.caption_template);
  RecordingCaptions {
    settings: settings.caption_settings,
    style: settings.caption_layout.applied_to(style),
    layout: settings.caption_layout,
    ..RecordingCaptions::default()
  }
}

/// The font families installed on the system, as read at launch, with
/// those the templates bring, by name.
#[tauri::command]
pub async fn installed_font_families() -> Result<Vec<FontFamily>, String> {
  // Waits, off the main thread, only where launch has not read them yet.
  tauri::async_runtime::spawn_blocking(|| {
    with_app_families(&SYSTEM_FAMILIES)
      .into_iter()
      .map(|family| FontFamily {
        name: family.name,
        weights: family.weights,
        italic: family.italic,
      })
      .collect()
  })
  .await
  .map_err(|error| error.to_string())
}

/// Where the previews live: one strip per style and sample, named by a hash
/// of both, so a style already drawn is never drawn again.
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

/// `style`'s preview of the `sample` words, drawn the first time it is
/// asked for and kept.
fn preview_strip(
  directory: &std::path::Path,
  style: &CaptionStyle,
  sample: &str,
) -> Result<CaptionPreviewStrip, String> {
  let description = serde_json::to_string(style).map_err(|error| error.to_string())?;
  let mut hasher = DefaultHasher::new();
  (PREVIEW_VERSION, description, sample).hash(&mut hasher);
  let path = directory.join(format!("{:016x}.png", hasher.finish()));
  let words: Vec<&str> = sample.split_whitespace().collect();
  let strip = CaptionPreviewStrip {
    path: path.clone(),
    width: PREVIEW.width,
    height: PREVIEW.height,
    frames: PREVIEW.frame_count(words.len()) as u32,
    frames_per_second: PREVIEW.frames_per_second,
  };
  if path.is_file() {
    // Marked as used, so pruning keeps the strips the panel shows.
    if let Ok(file) = std::fs::File::options().append(true).open(&path) {
      let _ = file.set_modified(std::time::SystemTime::now());
    }
    return Ok(strip);
  }
  let previewer = PREVIEWER.as_ref().map_err(Clone::clone)?;
  let frames = previewer
    .lock()
    .map_err(|_| "The caption previewer is poisoned".to_owned())?
    .frames(style, &words, &PREVIEW)?;
  let picture = CapturedImage {
    width: PREVIEW.width,
    height: PREVIEW.height * frames.len() as u32,
    rgba: frames.concat(),
  };
  let png = screenwide_output::encoding::encode_truecolor_png(&picture)?;
  // Written whole under another name first, so a strip half written when
  // the app stops is never taken for a finished one.
  let partial = path.with_extension("partial");
  std::fs::write(&partial, png)
    .and_then(|()| std::fs::rename(&partial, &path))
    .map_err(|error| format!("The caption preview could not be saved: {error}"))?;
  Ok(strip)
}

/// More strips than the panel ever shows at once, with room for the styles
/// tried on the way to one: the oldest past this are let go.
const KEPT_PREVIEWS: usize = 60;

/// Lets go of every strip in `directory` but the [`KEPT_PREVIEWS`] most
/// recently used. A strip let go is drawn again if it is asked for.
fn prune_previews(directory: &std::path::Path) {
  let Ok(entries) = std::fs::read_dir(directory) else {
    return;
  };
  let mut strips: Vec<(std::time::SystemTime, PathBuf)> = entries
    .flatten()
    .filter_map(|entry| Some((entry.metadata().ok()?.modified().ok()?, entry.path())))
    .collect();
  if strips.len() <= KEPT_PREVIEWS {
    return;
  }
  strips.sort_unstable_by_key(|strip| std::cmp::Reverse(strip.0));
  for (_, path) in &strips[KEPT_PREVIEWS..] {
    let _ = std::fs::remove_file(path);
  }
}

/// Each of `styles` previewed with the `sample` words, in order: a strip of
/// frames drawn by the compositor, so a template's tile is true to what the
/// export draws.
#[tauri::command]
pub async fn render_caption_previews(
  app: AppHandle,
  styles: Vec<CaptionStyle>,
  sample: String,
) -> Result<Vec<CaptionPreviewStrip>, String> {
  let directory = preview_directory(&app)?;
  tauri::async_runtime::spawn_blocking(move || {
    let strips = styles
      .iter()
      .map(|style| preview_strip(&directory, style, &sample))
      .collect();
    prune_previews(&directory);
    strips
  })
  .await
  .map_err(|error| error.to_string())?
}
