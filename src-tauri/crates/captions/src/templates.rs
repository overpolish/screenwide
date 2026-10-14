// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The caption templates: each a style under an id, read from a manifest
//! in the template's own folder. The app's own are bundled from
//! `templates/` in this crate and listed in order by its `index.json`;
//! yours are read from a folder of them by [`load_user_templates`], by the
//! same [`parse_manifest`], so a template is one kind of thing wherever it
//! came from. A manifest gives only the options it sets apart from
//! [`CaptionStyle::default`], which `base.json` spells out.

mod bundled;
mod fonts;
mod manifest;
mod package;
mod user;

use std::fmt;

use serde::{Deserialize, Deserializer, Serialize};
use ts_rs::TS;

use crate::style::CaptionStyle;

pub use fonts::{template_font_files, TemplateFont};
pub use manifest::{parse_manifest, ManifestError, FORMAT_VERSION};
pub use package::{remove_user_template, TemplatePackage};
pub use user::{load_user_templates, user_template_fonts, MANIFEST_FILE};

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
    let templates = caption_templates();
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

  /// What the template's tiles preview it over: light where there is no
  /// such template.
  pub fn preview_backdrop(&self) -> CaptionPreviewBackdrop {
    caption_templates()
      .iter()
      .find(|template| template.id == *self)
      .map_or_else(CaptionPreviewBackdrop::default, |template| {
        template.preview_backdrop
      })
  }
}

impl From<&str> for CaptionTemplateId {
  /// `id` as a template's, or the template it was renamed to.
  fn from(id: &str) -> Self {
    Self(bundled::renamed(id).unwrap_or(id).to_owned())
  }
}

impl Default for CaptionTemplateId {
  fn default() -> Self {
    Self(bundled::default_id().to_owned())
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

/// What a template's manifest says of it: its name, and who made it, which
/// version of it this is, and a line on what it is, where it says.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionTemplateInfo {
  pub name: String,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub author: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub version: Option<String>,
  #[serde(skip_serializing_if = "Option::is_none")]
  #[ts(optional)]
  pub description: Option<String>,
}

/// Where a template came from: the app, or the folder of your own.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, TS)]
#[serde(rename_all = "kebab-case")]
#[ts(export)]
pub enum CaptionTemplateSource {
  Bundled,
  User,
}

/// The picture a template's tiles preview its captions over: whichever its
/// captions and their effects read best on, as its manifest asks.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, Hash, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum CaptionPreviewBackdrop {
  #[default]
  Light,
  Dark,
}

/// A template: its id, what its manifest says of it, where it came from,
/// the fonts it brings, and the style it ships with.
#[derive(Clone, Debug, PartialEq, Serialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct CaptionTemplate {
  pub id: CaptionTemplateId,
  pub info: CaptionTemplateInfo,
  pub source: CaptionTemplateSource,
  /// The text engines are handed these as templates are read, and the
  /// panel asks for families by name, so they stay here.
  #[serde(skip)]
  #[ts(skip)]
  pub fonts: Vec<TemplateFont>,
  /// Only the previews are drawn over it, and they are drawn natively.
  #[serde(skip)]
  #[ts(skip)]
  pub preview_backdrop: CaptionPreviewBackdrop,
  pub style: CaptionStyle,
}

/// The app's own templates, in the order the panel offers them.
pub fn bundled_templates() -> &'static [CaptionTemplate] {
  bundled::templates()
}

/// Every template, the app's own first in their order, then yours by name.
pub fn caption_templates() -> Vec<CaptionTemplate> {
  let mut templates = bundled_templates().to_vec();
  templates.extend(user::templates());
  templates
}

#[cfg(test)]
mod tests;
