// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A template's manifest: a `template.json` in the template's own folder,
//! beside anything else it brings.
//!
//! ```json
//! { "formatVersion": 5, "id": "supersize", "name": "Supersize",
//!   "author": "Screenwide", "version": "1.0.0", "description": "…",
//!   "previewBackdrop": "dark", "aliases": [], "style": { … } }
//! ```
//!
//! `style` holds the options the template sets apart from
//! [`TextStyle::default`], each replacing the default's whole value: a
//! font is given whole, not just its weight. The format version comes first
//! so a manifest written for a later format is turned away by name rather
//! than misread.
//!
//! Format 2 added `motion`, written whole or as a preset's name; format 1's
//! `entrance` named the same presets, and is read as it. Format 3 added the
//! template's own name, which it needs, and its author, version and a line
//! describing it, which it may give, and the fonts it brings. Format 4 added
//! the motion's exits, `captionOut` and `wordOut`, `visibleWords`, `shadow`,
//! the `underline` and `glow` backgrounds, and `previewBackdrop`: `light`,
//! the default, or `dark`, whichever the template's tiles read best over.
//! Format 5 added `controls`, the points a text's own grips move, and
//! `elements`, what the template draws to them: a `tail`, drawn from a box
//! background. It also added `"auto"` as the `textColor`, black or white on
//! the background colour.

use std::fmt;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::fonts::TemplateFont;
use super::{
  TextPreviewBackdrop, TextTemplate, TextTemplateId, TextTemplateInfo, TextTemplateSource,
};
use crate::style::{TextBackground, TextStyle};

/// The newest manifest format this app reads; every one since the first is
/// read too.
pub const FORMAT_VERSION: u64 = 5;

/// Why a manifest cannot be read, in English for a log or an author.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ManifestError(String);

impl ManifestError {
  pub(super) fn json(error: serde_json::Error) -> Self {
    Self(error.to_string())
  }
}

impl fmt::Display for ManifestError {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.0)
  }
}

impl std::error::Error for ManifestError {}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Manifest {
  id: String,
  #[serde(default)]
  aliases: Vec<String>,
  name: Option<String>,
  author: Option<String>,
  version: Option<String>,
  description: Option<String>,
  #[serde(rename = "previewBackdrop")]
  preview_backdrop: Option<TextPreviewBackdrop>,
  #[serde(default)]
  fonts: Vec<TemplateFont>,
  style: Map<String, Value>,
}

/// The longest each of a manifest's words about itself may run, in
/// characters: enough to say, short enough for a tile and a menu.
const MAX_NAME: usize = 40;
const MAX_AUTHOR: usize = 60;
const MAX_VERSION: usize = 20;
const MAX_DESCRIPTION: usize = 200;

/// What the manifest says of itself. Format 3 gives a template its own
/// name; an older one is named by its id.
fn info(version: u64, manifest: &Manifest) -> Result<TextTemplateInfo, ManifestError> {
  let field = |label: &str, value: &Option<String>, longest: usize| {
    let value = value.as_deref().map(str::trim);
    match value {
      Some("") => Err(ManifestError(format!("its {label} is empty"))),
      Some(text) if text.chars().count() > longest => Err(ManifestError(format!(
        "its {label} is longer than {longest} characters"
      ))),
      text => Ok(text.map(str::to_owned)),
    }
  };
  let name = match field("name", &manifest.name, MAX_NAME)? {
    Some(name) => name,
    None if version < 3 => manifest.id.clone(),
    None => return Err(ManifestError("it has no name".to_owned())),
  };
  Ok(TextTemplateInfo {
    name,
    author: field("author", &manifest.author, MAX_AUTHOR)?,
    version: field("version", &manifest.version, MAX_VERSION)?,
    description: field("description", &manifest.description, MAX_DESCRIPTION)?,
  })
}

/// `json`'s format version, checked, and the rest of it, which can then be
/// read strictly.
pub(super) fn versioned(json: &str) -> Result<(u64, Value), ManifestError> {
  let mut value: Value = serde_json::from_str(json).map_err(ManifestError::json)?;
  let version = value
    .as_object_mut()
    .and_then(|object| object.remove("formatVersion"))
    .and_then(|version| version.as_u64());
  match version {
    Some(version @ 1..=FORMAT_VERSION) => Ok((version, value)),
    Some(other) => Err(ManifestError(format!(
      "format version {other} is not one this app reads (1 to {FORMAT_VERSION}); it needs a newer version of Screenwide"
    ))),
    None => Err(ManifestError("there is no formatVersion".to_owned())),
  }
}

fn read(json: &str) -> Result<(u64, Manifest), ManifestError> {
  let (version, value) = versioned(json)?;
  let manifest: Manifest = serde_json::from_value(value).map_err(ManifestError::json)?;
  let is_id = |id: &str| {
    !id.is_empty()
      && id.split('-').all(|part| {
        !part.is_empty()
          && part
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
      })
  };
  if let Some(bad) = std::iter::once(&manifest.id)
    .chain(&manifest.aliases)
    .find(|id| !is_id(id))
  {
    return Err(ManifestError(format!(
      "\"{bad}\" is not an id: lowercase letters and digits, joined by single hyphens"
    )));
  }
  Ok((version, manifest))
}

/// A manifest's id and the ids it was known by, read without its style.
pub(super) fn identity(json: &str) -> Result<(String, Vec<String>), ManifestError> {
  read(json).map(|(_, manifest)| (manifest.id, manifest.aliases))
}

/// The template a manifest describes, from `source`: its options over the
/// defaults, every one an option a style has and within its range.
pub fn parse_manifest(
  json: &str,
  source: TextTemplateSource,
) -> Result<TextTemplate, ManifestError> {
  let (version, manifest) = read(json)?;
  let info = info(version, &manifest)?;
  super::fonts::check(&manifest.fonts).map_err(ManifestError)?;
  let Value::Object(mut style) =
    serde_json::to_value(TextStyle::default()).map_err(ManifestError::json)?
  else {
    unreachable!("a style is written as an object");
  };
  for (key, value) in manifest.style {
    // Format 1 named its motion's preset as the entrance.
    let key = if version == 1 && key == "entrance" {
      "motion".to_owned()
    } else if version == 1 && key == "motion" {
      return Err(ManifestError(
        "\"motion\" is a format 2 option; format 1 names it \"entrance\"".to_owned(),
      ));
    } else {
      key
    };
    if key == "template" || !style.contains_key(&key) {
      return Err(ManifestError(format!("\"{key}\" is not a style option")));
    }
    style.insert(key, value);
  }
  style.insert("template".to_owned(), Value::String(manifest.id.clone()));
  let style: TextStyle =
    serde_json::from_value(Value::Object(style)).map_err(ManifestError::json)?;
  let normalized = style.normalized();
  if normalized.controls != style.controls || normalized.elements != style.elements {
    return Err(ManifestError(format!(
      "\"{}\" declares a control point or an element it cannot draw: every control point \
       needs an id of letters, digits and hyphens of its own and an element drawn to it, \
       every element names a declared control point, and a text draws one tail at most",
      manifest.id
    )));
  }
  if !style.is_valid() {
    return Err(ManifestError(format!(
      "\"{}\" sets an option outside its range",
      manifest.id
    )));
  }
  // An older format could not say these, so one that does was not written
  // for it.
  if let Some(option) = format_4_option(&style).filter(|_| version < 4) {
    return Err(ManifestError(format!("{option} is a format 4 option")));
  }
  if let Some(option) = format_5_option(&style).filter(|_| version < 5) {
    return Err(ManifestError(format!("{option} is a format 5 option")));
  }
  if manifest.preview_backdrop.is_some() && version < 4 {
    return Err(ManifestError(
      "\"previewBackdrop\" is a format 4 option".to_owned(),
    ));
  }
  Ok(TextTemplate {
    id: TextTemplateId(manifest.id),
    info,
    source,
    fonts: manifest.fonts,
    preview_backdrop: manifest.preview_backdrop.unwrap_or_default(),
    style,
  })
}

/// The first option `style` sets that format 4 added, as a manifest names
/// it.
fn format_4_option(style: &TextStyle) -> Option<&'static str> {
  if style.motion.has_exit() {
    Some("the motion's \"captionOut\" or \"wordOut\"")
  } else if style.motion.turns() {
    Some("the motion's \"rotate\", \"rotateX\" or \"rotateY\"")
  } else if style.rolling_words().is_some() {
    Some("\"visibleWords\"")
  } else if style.shadow.is_some() {
    Some("\"shadow\"")
  } else if style.background == TextBackground::Underline {
    Some("the \"underline\" background")
  } else if style.background == TextBackground::Glow {
    Some("the \"glow\" background")
  } else {
    None
  }
}

/// The first option `style` sets that format 5 added, as a manifest names
/// it.
fn format_5_option(style: &TextStyle) -> Option<&'static str> {
  if !style.controls.is_empty() {
    Some("\"controls\"")
  } else if !style.elements.is_empty() {
    Some("\"elements\"")
  } else if style.text_color == crate::style::AUTO_INK {
    Some("the \"auto\" text colour")
  } else {
    None
  }
}
