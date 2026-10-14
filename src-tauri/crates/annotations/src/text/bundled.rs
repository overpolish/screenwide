// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The families the app carries for its caption templates, so a template
//! set in them looks the same on every system. Each is in the binary and
//! read from there by the text engine, before any family the system has of
//! the same name: on macOS through [`screenwide_bundled_font`], on Windows
//! into the engine's own collection.

/// A family the app carries, with the weights it has upright on the CSS
/// scale, lightest first, and whether it has an italic.
#[derive(Clone, Copy, Debug)]
pub struct BundledFamily {
  pub name: &'static str,
  pub weights: &'static [u16],
  pub italic: bool,
}

const EVERY_WEIGHT: &[u16] = &[100, 200, 300, 400, 500, 600, 700, 800, 900];

/// Every family the app carries, by name. Each name is the one its font
/// files give, which the text engines find it by.
pub const BUNDLED_FAMILIES: &[BundledFamily] = &[
  BundledFamily {
    name: "Anton",
    weights: &[400],
    italic: false,
  },
  BundledFamily {
    name: "DM Sans",
    weights: EVERY_WEIGHT,
    italic: false,
  },
  BundledFamily {
    name: "Instrument Serif",
    weights: &[400],
    italic: true,
  },
  BundledFamily {
    name: "Montserrat",
    weights: EVERY_WEIGHT,
    italic: false,
  },
];

/// The font files of [`BUNDLED_FAMILIES`].
pub static BUNDLED_FILES: &[&[u8]] = &[
  include_bytes!("../../../../assets/Anton-Regular.ttf"),
  include_bytes!("../../../../assets/DMSans-VariableFont_opsz,wght.ttf"),
  include_bytes!("../../../../assets/InstrumentSerif-Regular.ttf"),
  include_bytes!("../../../../assets/InstrumentSerif-Italic.ttf"),
  include_bytes!("../../../../assets/Montserrat-VariableFont_wght.ttf"),
];

/// The font file at `index` of [`BUNDLED_FILES`], with its length in
/// `length`, or null past the last: what `annotation_font.m` reads its
/// faces from.
///
/// # Safety
///
/// `length` is null or points to a `u32` the caller owns.
#[cfg(target_os = "macos")]
#[no_mangle]
pub unsafe extern "C" fn screenwide_bundled_font(index: u32, length: *mut u32) -> *const u8 {
  let Some(file) = BUNDLED_FILES.get(index as usize) else {
    return std::ptr::null();
  };
  if !length.is_null() {
    // Safety: the caller passes a `u32` of its own.
    unsafe { *length = file.len() as u32 };
  }
  file.as_ptr()
}
