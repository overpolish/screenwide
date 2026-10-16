// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>

/// AppKit's member weight, which runs from 0 to 15, on the CSS scale of 100
/// to 900: 5 is regular, 9 bold.
static uint16_t css_weight(NSInteger weight) {
  static const uint16_t steps[] = {100, 100, 200, 300, 300, 400, 500, 500,
                                   600, 700, 800, 900, 900, 900, 900, 900};
  return steps[MAX(0, MIN(weight, (NSInteger)15))];
}

/// Calls `found` with every family installed on the system, by name, the
/// weights it has upright, and whether it has an italic. Families with no
/// upright member are left out: a caption cannot be set in italics alone.
void screenwide_installed_families(void *context,
                                   void (*found)(void *, const uint8_t *, uint32_t,
                                                 const uint16_t *, uint32_t, uint32_t)) {
  @autoreleasepool {
    NSFontManager *manager = [NSFontManager sharedFontManager];
    for (NSString *family in manager.availableFontFamilies) {
      // Families whose names start with a dot are the system's private faces.
      if ([family hasPrefix:@"."]) continue;
      uint16_t weights[16];
      uint32_t count = 0;
      uint32_t italic = 0;
      for (NSArray *member in [manager availableMembersOfFontFamily:family]) {
        if (member.count < 4) continue;
        NSFontTraitMask traits = [member[3] unsignedIntegerValue];
        if ((traits & NSItalicFontMask) != 0) {
          italic = 1;
          continue;
        }
        uint16_t weight = css_weight([member[2] integerValue]);
        BOOL seen = NO;
        for (uint32_t index = 0; index < count; index++) seen |= weights[index] == weight;
        if (!seen && count < 16) weights[count++] = weight;
      }
      if (count == 0) continue;
      const char *name = family.UTF8String;
      found(context, (const uint8_t *)name, (uint32_t)strlen(name), weights, count, italic);
    }
  }
}
