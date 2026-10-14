// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The font families captions may be set in: those installed on the
//! system, and those the app carries.

use super::bundled::BUNDLED_FAMILIES;

/// A family captions may be set in, the weights it has upright on the CSS
/// scale of 100 to 900, lightest first, and whether it has an italic.
#[derive(Clone, Debug, PartialEq)]
pub struct InstalledFamily {
  pub name: String,
  pub weights: Vec<u16>,
  pub italic: bool,
}

/// Every family installed on the system or carried by the app, by name,
/// each once. A family the app carries is listed as it carries it, though
/// the system has its own copy, since the app's is the one set.
pub fn installed_families() -> Vec<InstalledFamily> {
  let mut families = platform::families();
  families.retain(|family| {
    BUNDLED_FAMILIES
      .iter()
      .all(|bundled| bundled.name != family.name)
  });
  families.extend(BUNDLED_FAMILIES.iter().map(|bundled| InstalledFamily {
    name: bundled.name.to_owned(),
    weights: bundled.weights.to_vec(),
    italic: bundled.italic,
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
    crate::text::type_device::installed_families().unwrap_or_default()
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

  /// Georgia ships with both systems in a regular and a bold.
  #[test]
  fn lists_each_installed_family_once_with_its_upright_weights() {
    let families = installed_families();
    let georgia: Vec<_> = families
      .iter()
      .filter(|family| family.name == "Georgia")
      .collect();
    assert_eq!(georgia.len(), 1);
    assert!(georgia[0].weights.contains(&400) && georgia[0].weights.contains(&700));
    assert!(georgia[0].italic);
    assert!(families.iter().all(|family| !family.name.starts_with('.')));
    // The app's own families are offered whatever the system has.
    let serif = families
      .iter()
      .find(|family| family.name == "Instrument Serif")
      .unwrap();
    assert_eq!((serif.weights.as_slice(), serif.italic), (&[400][..], true));
  }
}
