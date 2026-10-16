// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The font families captions may be set in: those installed on the
//! system, and those the caption templates bring.

use super::app_fonts::app_families;

/// A family captions may be set in, the weights it has upright on the CSS
/// scale of 100 to 900, lightest first, and whether it has an italic.
#[derive(Clone, Debug, PartialEq)]
pub struct InstalledFamily {
  pub name: String,
  pub weights: Vec<u16>,
  pub italic: bool,
}

/// Every family installed on the system, by name. Reading them takes a
/// moment, so a caller keeps them.
pub fn system_families() -> Vec<InstalledFamily> {
  platform::families()
}

/// `system`'s families with those the templates bring, by name, each once.
/// A family a template brings is listed as it brings it, though the system
/// has its own copy, since that is the one set.
pub fn with_app_families(system: &[InstalledFamily]) -> Vec<InstalledFamily> {
  let brought = app_families();
  let mut families: Vec<InstalledFamily> = system
    .iter()
    .filter(|family| brought.iter().all(|app| app.name != family.name))
    .cloned()
    .collect();
  families.extend(brought.into_iter().map(|app| InstalledFamily {
    name: app.name,
    weights: app.weights,
    italic: app.italic,
  }));
  for family in &mut families {
    family.weights.sort_unstable();
    family.weights.dedup();
  }
  families.sort_by_key(|family| family.name.to_lowercase());
  families.dedup_by(|a, b| a.name == b.name);
  families
}

#[cfg(target_os = "macos")]
mod platform {
  use super::InstalledFamily;

  unsafe extern "C" {
    fn screenwide_installed_families(
      context: *mut std::ffi::c_void,
      found: extern "C" fn(*mut std::ffi::c_void, *const u8, u32, *const u16, u32, u32),
    );
  }

  extern "C" fn found(
    context: *mut std::ffi::c_void,
    name: *const u8,
    length: u32,
    weights: *const u16,
    count: u32,
    italic: u32,
  ) {
    // Safety: the context is the list `families` passed in, and the name and
    // weights are valid for the call.
    let families = unsafe { &mut *context.cast::<Vec<InstalledFamily>>() };
    let (name, weights) = unsafe {
      (
        std::slice::from_raw_parts(name, length as usize),
        std::slice::from_raw_parts(weights, count as usize),
      )
    };
    families.push(InstalledFamily {
      name: String::from_utf8_lossy(name).into_owned(),
      weights: weights.to_vec(),
      italic: italic != 0,
    });
  }

  pub(super) fn families() -> Vec<InstalledFamily> {
    let mut families = Vec::new();
    // Safety: `found` only runs during the call, while `families` lives.
    unsafe {
      screenwide_installed_families((&raw mut families).cast(), found);
    }
    families
  }
}

#[cfg(target_os = "windows")]
mod platform {
  use super::InstalledFamily;

  pub(super) fn families() -> Vec<InstalledFamily> {
    crate::engine::type_device::installed_families().unwrap_or_default()
  }
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
mod platform {
  use super::InstalledFamily;

  pub(super) fn families() -> Vec<InstalledFamily> {
    Vec::new()
  }
}

#[cfg(all(test, any(target_os = "macos", target_os = "windows")))]
mod tests {
  use super::*;
  use crate::engine::app_fonts::{add_app_fonts, AppFamily};

  /// Georgia ships with both systems in a regular and a bold.
  #[test]
  fn lists_each_family_once_a_templates_own_in_place_of_the_systems() {
    let system = system_families();
    add_app_fonts(
      &[AppFamily {
        name: "Georgia".to_owned(),
        weights: vec![900],
        italic: false,
      }],
      &[],
    );
    let families = with_app_families(&system);
    let georgia: Vec<_> = families
      .iter()
      .filter(|family| family.name == "Georgia")
      .collect();
    assert_eq!(georgia.len(), 1);
    // The system's own has a regular, a bold and an italic; the one a
    // template brings is listed as it brings it.
    let installed = system
      .iter()
      .find(|family| family.name == "Georgia")
      .unwrap();
    assert!(installed.weights.contains(&400) && installed.weights.contains(&700));
    assert!(installed.italic);
    assert_eq!(georgia[0].weights, [900]);
    assert!(families.iter().all(|family| !family.name.starts_with('.')));
  }
}
