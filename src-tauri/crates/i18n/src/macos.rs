// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! What macOS needs from the catalog beyond `t!`: the layout direction of
//! the app's own Cocoa chrome.

use objc2_foundation::{NSArgumentDomain, NSMutableDictionary, NSNumber, NSString, NSUserDefaults};

use super::{CATALOG, PSEUDO_BIDI};

/// Lays the app's own Cocoa chrome out right to left under the bidi
/// pseudo-locale, as macOS does for a right-to-left translation: traffic
/// lights and menus mirror. A real translation needs nothing here, since
/// macOS reads the direction from the bundle's `.lproj` folders. The keys go
/// where Xcode's right-to-left pseudolanguage puts them, among the launch
/// arguments: AppKit does not look at registered defaults for them. Set
/// before AppKit starts, and only for this launch.
pub fn apply_layout_direction() {
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
