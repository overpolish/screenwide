// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The font families the app's caption templates bring, so a template set
//! in one looks the same on every system: the app's own templates' and
//! those of any template you add, handed over as the templates are read.
//! The text engines set a family found here before any family the system
//! has of the same name: on macOS through [`screenwide_app_font`], on
//! Windows into the engine's own collection. Each time fonts are added the
//! generation moves on, so an engine knows to read them again.

use std::collections::HashSet;
use std::hash::{DefaultHasher, Hash, Hasher};
use std::sync::{Arc, LazyLock, PoisonError, RwLock};

/// A family the templates bring, with the weights it has on the CSS scale,
/// lightest first, and whether it has an italic.
#[derive(Clone, Debug, PartialEq)]
pub struct AppFamily {
  pub name: String,
  pub weights: Vec<u16>,
  pub italic: bool,
}

#[derive(Default)]
struct Registry {
  families: Vec<AppFamily>,
  /// Each font file. The registry owns them and never lets one go, so a
  /// face an engine made from one, reading it in place, stays good.
  files: Vec<Arc<[u8]>>,
  /// What each file held, so the same font handed over again is not kept
  /// twice.
  held: HashSet<u64>,
  generation: u32,
}

static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(RwLock::default);

fn contents(bytes: &[u8]) -> u64 {
  let mut hasher = DefaultHasher::new();
  bytes.hash(&mut hasher);
  hasher.finish()
}

/// Adds `families`, merged with any of the same name, and the font `files`
/// they are set from.
pub fn add_app_fonts(families: &[AppFamily], files: &[&[u8]]) {
  let mut registry = REGISTRY.write().unwrap_or_else(PoisonError::into_inner);
  let mut changed = false;
  for family in families {
    let mut weights = family.weights.clone();
    match registry
      .families
      .iter_mut()
      .find(|known| known.name == family.name)
    {
      Some(known) => {
        weights.extend(&known.weights);
        weights.sort_unstable();
        weights.dedup();
        let italic = known.italic || family.italic;
        changed |= weights != known.weights || italic != known.italic;
        known.weights = weights;
        known.italic = italic;
      }
      None => {
        weights.sort_unstable();
        weights.dedup();
        registry.families.push(AppFamily {
          name: family.name.clone(),
          weights,
          italic: family.italic,
        });
        changed = true;
      }
    }
  }
  for file in files {
    if registry.held.insert(contents(file)) {
      registry.files.push(Arc::from(*file));
      changed = true;
    }
  }
  if changed {
    registry.generation = registry.generation.wrapping_add(1);
  }
}

/// Every family the templates bring.
pub fn app_families() -> Vec<AppFamily> {
  REGISTRY
    .read()
    .unwrap_or_else(PoisonError::into_inner)
    .families
    .clone()
}

/// Every font file the templates bring, and the generation they are of.
pub fn app_font_files() -> (u32, Vec<Arc<[u8]>>) {
  let registry = REGISTRY.read().unwrap_or_else(PoisonError::into_inner);
  (registry.generation, registry.files.clone())
}

/// How many times fonts have been added, so a cache of anything measured
/// or made from them knows when to read them again.
pub fn app_fonts_generation() -> u32 {
  REGISTRY
    .read()
    .unwrap_or_else(PoisonError::into_inner)
    .generation
}

/// [`app_fonts_generation`], for `annotation_font.m`.
#[cfg(target_os = "macos")]
#[no_mangle]
pub extern "C" fn screenwide_app_fonts_generation() -> u32 {
  app_fonts_generation()
}

/// The font file at `index`, with its length in `length`, or null past the
/// last: what `annotation_font.m` reads its faces from. The registry owns
/// the bytes and never lets them go, so they are read in place.
///
/// # Safety
///
/// `length` is null or points to a `u32` the caller owns.
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_app_font(index: u32, length: *mut u32) -> *const u8 {
  let registry = REGISTRY.read().unwrap_or_else(PoisonError::into_inner);
  let Some(file) = registry.files.get(index as usize) else {
    return std::ptr::null();
  };
  if !length.is_null() {
    // Safety: the caller passes a `u32` of its own.
    unsafe { *length = file.len() as u32 };
  }
  file.as_ptr()
}

#[cfg(test)]
mod tests {
  use super::{add_app_fonts, app_families, app_font_files, AppFamily};

  #[test]
  fn keeps_each_family_and_file_once_and_moves_on_only_when_something_is_new() {
    let family = |weights: &[u16], italic| AppFamily {
      name: "Test Family Only Here".to_owned(),
      weights: weights.to_vec(),
      italic,
    };
    add_app_fonts(&[family(&[700, 400], false)], &[b"one font file"]);
    let (generation, files) = app_font_files();
    add_app_fonts(&[family(&[400], false)], &[b"one font file"]);
    assert_eq!(app_font_files().0, generation, "nothing new");
    assert_eq!(app_font_files().1.len(), files.len());
    add_app_fonts(&[family(&[400], true)], &[b"its italic"]);
    assert_ne!(app_font_files().0, generation);
    let merged = app_families()
      .into_iter()
      .find(|known| known.name == "Test Family Only Here")
      .unwrap();
    assert_eq!(merged.weights, [400, 700]);
    assert!(merged.italic);
  }
}
