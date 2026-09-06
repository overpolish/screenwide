// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

BOOL screenwide_region_osc_update_magnifier(
    ScreenwideRegionOSC *surface, NativeOscResult result,
    NSPoint desktop_point, uint32_t phase, uint32_t edges) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(surface);
  BOOL visible = phase == 3 && result.gesture == 3 &&
                 result.has_region != 0 && result.handle != 0;
  ScreenwideRegionOSC *target = nil;
  if (visible) {
    for (ScreenwideRegionOSC *candidate in
         screenwide_region_osc_surfaces(root)) {
      NSRect bounds = NSMakeRect(candidate.desktopOffset.x,
                                 candidate.desktopOffset.y,
                                 candidate.host.bounds.size.width,
                                 candidate.host.bounds.size.height);
      if (NSPointInRect(desktop_point, bounds)) {
        target = candidate;
        break;
      }
    }
  }

  NSRect desktop_frame = NSMakeRect(result.x, result.y,
                                    result.width, result.height);
  NSPoint desktop_anchor = screenwide_region_magnifier_anchor(
      desktop_point, desktop_frame, edges);
  BOOL changed = NO;
  for (ScreenwideRegionOSC *candidate in
       screenwide_region_osc_surfaces(root)) {
    if (candidate != target || candidate.magnifierSource == nil) {
      changed |= candidate.magnifier.active != 0;
      ScreenwideRegionMagnifier cleared = candidate.magnifier;
      cleared.active = 0;
      candidate.magnifier = cleared;
      continue;
    }
    NSPoint anchor = NSMakePoint(
        desktop_anchor.x - candidate.desktopOffset.x,
        desktop_anchor.y - candidate.desktopOffset.y);
    CGFloat scale = candidate.host.window.backingScaleFactor ?: 1.0;
    NSString *appearance = [candidate.host.effectiveAppearance
        bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                             NSAppearanceNameDarkAqua ]];
    uint32_t light_mode =
        [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
    candidate.magnifier = screenwide_region_magnifier_make(
        anchor, scale, edges, light_mode, 0, 0, 0,
        anchor.x / MAX(candidate.host.bounds.size.width, 1.0),
        anchor.y / MAX(candidate.host.bounds.size.height, 1.0), 0, 0, 1, 1);
    changed = YES;
  }
  return changed;
}
