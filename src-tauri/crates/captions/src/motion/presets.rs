// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The motions the app ships under a name, compiled in from
//! `templates/motion-presets.json`. A template, or a style saved before
//! motion was data, names one of these rather than writing it out.

use std::collections::HashMap;
use std::sync::LazyLock;

use serde::Deserialize;

use super::CaptionMotion;

#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct Presets {
  #[allow(dead_code, reason = "checked by the templates' version test")]
  format_version: u64,
  presets: HashMap<String, CaptionMotion>,
}

const PRESETS: &str = include_str!("../../templates/motion-presets.json");

static BUNDLED: LazyLock<Presets> =
  LazyLock::new(|| serde_json::from_str(PRESETS).expect("the bundled motion presets are valid"));

/// What styles saved before motion was data called how a caption comes on,
/// as the presets that do the same: plain captions and words revealed one
/// by one fade their words in.
const RENAMED: [(&str, &str); 2] = [("none", "fade"), ("words", "fade")];

/// The preset called `name`, or one an older style called by it.
pub(super) fn named(name: &str) -> Option<CaptionMotion> {
  let name = RENAMED
    .iter()
    .find(|(old, _)| *old == name)
    .map_or(name, |(_, new)| new);
  BUNDLED.presets.get(name).cloned()
}

/// The bundled presets' names and the format version their file is in.
#[cfg(test)]
pub(super) fn bundled() -> (u64, Vec<&'static str>) {
  (
    BUNDLED.format_version,
    BUNDLED.presets.keys().map(String::as_str).collect(),
  )
}
