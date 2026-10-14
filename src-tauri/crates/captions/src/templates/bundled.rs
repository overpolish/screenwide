// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The templates the app carries, compiled in from `templates/`.

use std::sync::LazyLock;

use serde::Deserialize;

use super::manifest::{identity, parse_manifest, versioned, ManifestError};
use super::{CaptionTemplate, CaptionTemplateSource};

/// `templates/index.json`: which bundled template is the default, and the
/// order the panel offers them in.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Index {
  pub(super) default: String,
  pub(super) templates: Vec<String>,
}

pub(super) const INDEX: &str = include_str!("../../templates/index.json");

/// Every bundled manifest, one per template folder; the index orders them.
pub(super) const MANIFESTS: [&str; 12] = [
  include_str!("../../templates/box/template.json"),
  include_str!("../../templates/clean/template.json"),
  include_str!("../../templates/pop/template.json"),
  include_str!("../../templates/highlight/template.json"),
  include_str!("../../templates/karaoke/template.json"),
  include_str!("../../templates/punch/template.json"),
  include_str!("../../templates/editorial/template.json"),
  include_str!("../../templates/supersize/template.json"),
  include_str!("../../templates/stack/template.json"),
  include_str!("../../templates/staircase/template.json"),
  include_str!("../../templates/rise/template.json"),
  include_str!("../../templates/stagger/template.json"),
];

/// The font files and licences the app's own templates share, by their
/// place in a package, as a manifest names them.
pub(super) const FONT_FILES: [(&str, &[u8]); 9] = [
  (
    "fonts/Anton-Regular.ttf",
    include_bytes!("../../templates/fonts/Anton-Regular.ttf"),
  ),
  (
    "fonts/Anton-OFL-1.1.txt",
    include_bytes!("../../templates/fonts/Anton-OFL-1.1.txt"),
  ),
  (
    "fonts/DMSans-VariableFont_opsz,wght.ttf",
    include_bytes!("../../templates/fonts/DMSans-VariableFont_opsz,wght.ttf"),
  ),
  (
    "fonts/DMSans-OFL-1.1.txt",
    include_bytes!("../../templates/fonts/DMSans-OFL-1.1.txt"),
  ),
  (
    "fonts/InstrumentSerif-Regular.ttf",
    include_bytes!("../../templates/fonts/InstrumentSerif-Regular.ttf"),
  ),
  (
    "fonts/InstrumentSerif-Italic.ttf",
    include_bytes!("../../templates/fonts/InstrumentSerif-Italic.ttf"),
  ),
  (
    "fonts/InstrumentSerif-OFL-1.1.txt",
    include_bytes!("../../templates/fonts/InstrumentSerif-OFL-1.1.txt"),
  ),
  (
    "fonts/Montserrat-VariableFont_wght.ttf",
    include_bytes!("../../templates/fonts/Montserrat-VariableFont_wght.ttf"),
  ),
  (
    "fonts/Montserrat-OFL-1.1.txt",
    include_bytes!("../../templates/fonts/Montserrat-OFL-1.1.txt"),
  ),
];

/// The shared file at `path` in a package, if the app carries it.
pub(super) fn font_file(path: &str) -> Option<&'static [u8]> {
  FONT_FILES
    .iter()
    .find(|(name, _)| *name == path)
    .map(|(_, bytes)| *bytes)
}

pub(super) fn index() -> Result<Index, ManifestError> {
  serde_json::from_value(versioned(INDEX)?.1).map_err(ManifestError::json)
}

/// The bundled manifest of template `id`, as written.
pub(super) fn manifest(id: &str) -> Option<&'static str> {
  MANIFESTS
    .into_iter()
    .find(|json| identity(json).is_ok_and(|(each, _)| each == id))
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
  let index = index().expect("the bundled template index is valid");
  let renamed = MANIFESTS
    .iter()
    .flat_map(|json| {
      let (id, aliases) = identity(json).expect("every bundled template manifest is valid");
      aliases.into_iter().map(move |alias| (alias, id.clone()))
    })
    .collect();
  Identities {
    default: index.default,
    renamed,
  }
});

/// The template `id` became, if a bundled one was renamed from it.
pub(super) fn renamed(id: &str) -> Option<&'static str> {
  IDENTITIES
    .renamed
    .iter()
    .find(|(old, _)| old == id)
    .map(|(_, new)| new.as_str())
}

pub(super) fn default_id() -> &'static str {
  &IDENTITIES.default
}

static TEMPLATES: LazyLock<Vec<CaptionTemplate>> = LazyLock::new(|| {
  let index = index().expect("the bundled template index is valid");
  let parsed: Vec<CaptionTemplate> = MANIFESTS
    .iter()
    .map(|json| {
      parse_manifest(json, CaptionTemplateSource::Bundled)
        .expect("every bundled template manifest is valid")
    })
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

pub(super) fn templates() -> &'static [CaptionTemplate] {
  &TEMPLATES
}
