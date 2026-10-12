// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

void screenwide_region_osc_add_icon(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    uint8_t icon, CGFloat left, CGFloat top,
                                    CGFloat icon_size) {
  if (icon == 0)
    return;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(left, top, icon_size, icon_size), 21 + icon);
}
