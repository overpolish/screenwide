// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

use objc2::MainThreadMarker;
use objc2_app_kit::{NSAppearanceNameAqua, NSAppearanceNameDarkAqua, NSApplication};
use objc2_foundation::NSArray;

/// Whether the application draws in the light appearance.
pub(crate) fn light_appearance(mtm: MainThreadMarker) -> bool {
  let appearance = NSApplication::sharedApplication(mtm).effectiveAppearance();
  // Safety: the names are AppKit's own constants.
  let names = unsafe { NSArray::from_slice(&[NSAppearanceNameAqua, NSAppearanceNameDarkAqua]) };
  appearance
    .bestMatchFromAppearancesWithNames(&names)
    .is_some_and(|name| &*name == unsafe { NSAppearanceNameAqua })
}
