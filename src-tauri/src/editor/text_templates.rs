// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The templates text is styled with, the app's own and yours, and the font
//! families installed on the system; and what new captions start as. Each
//! template's preview is drawn in `template_previews`.

use std::path::PathBuf;
use std::sync::LazyLock;

use screenwide_text::captions::RecordingCaptions;
use screenwide_text::engine::app_fonts::{add_app_fonts, AppFamily};
use screenwide_text::engine::installed::{system_families, with_app_families, InstalledFamily};
use screenwide_text::{TextLayout, TextStyle, TextTemplate, TextTemplateId};
use serde::Serialize;
use tauri::{AppHandle, Manager};
use ts_rs::TS;

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

/// Every template, the app's own first in the order the panel
/// offers them, then yours: each with what its manifest says of it. A
/// template saved as a default in the general settings takes the place of
/// its own style in the panel.
#[tauri::command]
pub fn text_templates() -> Vec<TextTemplate> {
  screenwide_text::text_templates()
}

/// Where your own templates are kept, a folder each.
pub(crate) fn user_templates_folder(app: &AppHandle) -> Option<PathBuf> {
  app
    .path()
    .app_data_dir()
    .ok()
    .map(|data| data.join("text-templates"))
}

/// Reads your own templates, logging any left out and why: a template
/// that cannot be read never stops the app. Then hands every template's
/// fonts to the text engines, so text is set in them from now on.
fn load_user_templates(app: &AppHandle) {
  if let Some(folder) = user_templates_folder(app) {
    for problem in screenwide_text::load_user_templates(&folder) {
      eprintln!("A template was left out: {problem}");
    }
  }
  hand_over_fonts();
}

/// Hands every template's fonts to the text engines. They keep a font
/// handed over twice once, so this may run each time templates are read.
pub(crate) fn hand_over_fonts() {
  let fonts = screenwide_text::template_font_files();
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

/// Readies the Captions panel off the main thread, so it has everything the
/// moment it opens: reads the font families, and draws each template's
/// preview of [`launch_styles`].
pub(crate) fn warm(app: &AppHandle) {
  // Read before anything asks for the templates, which takes a moment for
  // a few small files.
  load_user_templates(app);
  let app = app.clone();
  std::thread::spawn(move || {
    LazyLock::force(&SYSTEM_FAMILIES);
    let sample = screenwide_i18n::t!("editor-panels-text-sample");
    super::template_previews::warm(&app, &launch_styles(&app), &sample);
  });
}

/// Every template as the panel shows it for new captions, the way
/// `useTextTemplates` and the tiles lay them out: its saved look, with
/// the layout options kept for new captions.
pub(crate) fn launch_styles(app: &AppHandle) -> Vec<TextStyle> {
  let settings = crate::settings::current(app);
  screenwide_text::text_templates()
    .into_iter()
    .map(|template| {
      settings
        .caption_layout
        .applied_to(template_style(&settings.caption_styles, &template.id))
    })
    .collect()
}

/// `template` as it was last set as default, or as it ships: a saved look,
/// over the layout options it ships with, which only the layout set by hand
/// changes.
fn template_style(saved: &[TextStyle], template: &TextTemplateId) -> TextStyle {
  let shipped = template.shipped_style();
  let look = saved
    .iter()
    .find(|style| style.template == *template)
    .cloned()
    .unwrap_or_else(|| shipped.clone());
  TextLayout::of(&shipped).applied_to(look)
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
