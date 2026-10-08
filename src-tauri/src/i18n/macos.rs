// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What macOS needs from the catalog beyond `t!`: the Objective-C overlays'
//! messages, and the layout direction of the app's own Cocoa chrome.

use objc2_foundation::{NSArgumentDomain, NSMutableDictionary, NSNumber, NSString, NSUserDefaults};

use super::{format, CATALOG, PSEUDO_BIDI};

/// The message `id` for the Objective-C overlays, which read it through
/// `screenwide_osc_localized`. The caller hands the string back to
/// `screenwide_i18n_free`.
///
/// # Safety
/// `id` must point to a NUL-terminated string.
#[no_mangle]
pub unsafe extern "C" fn screenwide_i18n_text(
  id: *const std::ffi::c_char,
) -> *mut std::ffi::c_char {
  let id = unsafe { std::ffi::CStr::from_ptr(id) }.to_string_lossy();
  // A message holds no NUL, so the conversion only fails on a broken file.
  std::ffi::CString::new(format(&id, None))
    .unwrap_or_default()
    .into_raw()
}

/// Frees a string `screenwide_i18n_text` returned.
///
/// # Safety
/// `text` must be null or a pointer `screenwide_i18n_text` returned, freed
/// once.
#[no_mangle]
pub unsafe extern "C" fn screenwide_i18n_free(text: *mut std::ffi::c_char) {
  if !text.is_null() {
    drop(unsafe { std::ffi::CString::from_raw(text) });
  }
}

/// Lays the app's own Cocoa chrome out right to left under the bidi
/// pseudo-locale, as macOS does for a right-to-left translation: traffic
/// lights and menus mirror. A real translation needs nothing here, since
/// macOS reads the direction from the bundle's `.lproj` folders. The keys go
/// where Xcode's right-to-left pseudolanguage puts them, among the launch
/// arguments: AppKit does not look at registered defaults for them. Set
/// before AppKit starts, and only for this launch.
pub(crate) fn apply_layout_direction() {
  if CATALOG.locale.language != PSEUDO_BIDI {
    return;
  }
  let defaults = NSUserDefaults::standardUserDefaults();
  let domain = unsafe { NSArgumentDomain };
  let arguments =
    NSMutableDictionary::dictionaryWithDictionary(&defaults.volatileDomainForName(domain));
  let yes = NSNumber::new_bool(true);
  for key in ["AppleTextDirection", "NSForceRightToLeftWritingDirection"] {
    arguments.insert(&*NSString::from_str(key), &yes);
  }
  // SAFETY: every value is an NSNumber, a property-list type as the domain
  // requires.
  unsafe { defaults.setVolatileDomain_forName(&arguments, domain) };
}
