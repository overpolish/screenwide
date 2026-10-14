// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caption templates: each a style under an id, read from a manifest.
//! The app's own are bundled from `templates/` in this crate, a folder each,
//! and listed in order by its `index.json`; a manifest gives only the
//! options it sets apart from [`CaptionStyle::default`], which `base.json`
//! spells out. A template from anywhere else would be read by the same
//! [`parse_manifest`], so it is one more entry, not a new kind of thing.

mod bundled;
mod manifest;

use std::fmt;
use std::sync::LazyLock;

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::style::CaptionStyle;

pub use manifest::{parse_manifest, ManifestError, FORMAT_VERSION};

/// Which template a style was taken from. Any id is kept as it was saved,
/// so a template the app no longer has does not lose a style that names it;
/// an id a template has since been renamed from reads as its new one.
#[derive(Clone, Debug, Eq, Hash, PartialEq, Serialize, TS)]
#[ts(export)]
pub struct CaptionTemplateId(String);

impl CaptionTemplateId {
  pub fn as_str(&self) -> &str {
    &self.0
  }

  /// The style the template ships with, or the default template's where
  /// there is no such template.
  pub fn shipped_style(&self) -> CaptionStyle {
    let templates = &*TEMPLATES;
    templates
      .iter()
      .find(|template| template.id == *self)
      .or_else(|| {
        templates
          .iter()
          .find(|template| template.id == CaptionTemplateId::default())
      })
      .map(|template| template.style.clone())
      .unwrap_or_default()
  }
}

impl From<&str> for CaptionTemplateId {
  /// `id` as a template's, or the template it was renamed to.
  fn from(id: &str) -> Self {
    Self(
      IDENTITIES
        .renamed
        .iter()
        .find(|(old, _)| old == id)
        .map_or(id, |(_, new)| new.as_str())
        .to_owned(),
    )
  }
}

impl Default for CaptionTemplateId {
  fn default() -> Self {
    Self(IDENTITIES.default.clone())
  }
}

impl fmt::Display for CaptionTemplateId {
  fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
    formatter.write_str(&self.0)
  }
}

impl<'de> Deserialize<'de> for CaptionTemplateId {
  fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
    String::deserialize(deserializer).map(|id| Self::from(id.as_str()))
  }
}

/// A template: its id and the style it ships with.
#[derive(Clone, Debug, PartialEq)]
pub struct CaptionTemplate {
  pub id: CaptionTemplateId,
  pub style: CaptionStyle,
}

/// The app's own templates, in the order the panel offers them.
pub fn bundled_templates() -> &'static [CaptionTemplate] {
  &TEMPLATES
}

/// The bundled templates' ids and the ones they were renamed from, read
/// apart from their styles: reading a style reads its template's id, so
/// these must be known before any style is.
struct Identities {
  default: String,
  /// Each id a template was known by, and its id now.
  renamed: Vec<(String, String)>,
}

static IDENTITIES: LazyLock<Identities> = LazyLock::new(|| {
  let index = bundled::index().expect("the bundled template index is valid");
  let renamed = bundled::MANIFESTS
    .iter()
    .flat_map(|json| {
      let (id, aliases) =
        manifest::identity(json).expect("every bundled template manifest is valid");
      aliases.into_iter().map(move |alias| (alias, id.clone()))
    })
    .collect();
  Identities {
    default: index.default,
    renamed,
  }
});

static TEMPLATES: LazyLock<Vec<CaptionTemplate>> = LazyLock::new(|| {
  let index = bundled::index().expect("the bundled template index is valid");
  let parsed: Vec<CaptionTemplate> = bundled::MANIFESTS
    .iter()
    .map(|json| parse_manifest(json).expect("every bundled template manifest is valid"))
    .collect();
  index
    .templates
    .iter()
    .map(|id| {
      parsed
        .iter()
        .find(|template| template.id.as_str() == id)
        .cloned()
        .expect("every template the index lists is bundled")
    })
    .collect()
});

#[cfg(test)]
mod tests;
