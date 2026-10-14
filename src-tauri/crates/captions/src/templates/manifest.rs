// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! A template's manifest: a `template.json` in the template's own folder,
//! beside anything else it brings.
//!
//! ```json
//! { "formatVersion": 2, "id": "supersize", "aliases": [], "style": { … } }
//! ```
//!
//! `style` holds the options the template sets apart from
//! [`CaptionStyle::default`], each replacing the default's whole value: a
//! font is given whole, not just its weight. The format version comes first
//! so a manifest written for a later format is turned away by name rather
//! than misread.
//!
//! Format 2 added `motion`, written whole or as a preset's name; format 1's
//! `entrance` named the same presets, and is read as it.

use std::fmt;

use serde::Deserialize;
use serde_json::{Map, Value};

use super::{CaptionTemplate, CaptionTemplateId};
use crate::style::CaptionStyle;

/// The newest manifest format this app reads; every one since the first is
/// read too.
pub const FORMAT_VERSION: u64 = 2;

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
  style: Map<String, Value>,
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

/// The template a manifest describes: its options over the defaults, every
/// one an option a style has and within its range.
pub fn parse_manifest(json: &str) -> Result<CaptionTemplate, ManifestError> {
  let (version, manifest) = read(json)?;
  let Value::Object(mut style) =
    serde_json::to_value(CaptionStyle::default()).map_err(ManifestError::json)?
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
  let style: CaptionStyle =
    serde_json::from_value(Value::Object(style)).map_err(ManifestError::json)?;
  if !style.is_valid() {
    return Err(ManifestError(format!(
      "\"{}\" sets an option outside its range",
      manifest.id
    )));
  }
  Ok(CaptionTemplate {
    id: CaptionTemplateId(manifest.id),
    style,
  })
}
