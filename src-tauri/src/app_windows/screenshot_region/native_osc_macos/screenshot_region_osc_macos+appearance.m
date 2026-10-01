// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

/// Clears the cached system accent so the next palette read goes back to the
/// platform. Exported by `src/system_accent.rs`.
extern void screenwide_system_accent_invalidate(void);

@interface ScreenwideRegionAppearanceObserverView : NSView
@property(nonatomic, weak) ScreenwideRegionOSC *osc;
- (void)refreshChrome;
@end

@implementation ScreenwideRegionAppearanceObserverView
- (NSView *)hitTest:(NSPoint)point {
  (void)point;
  return nil;
}

// Both the effective appearance and the accent decide colours that are baked
// into cached textures, so either change drops those textures and redraws.
- (void)refreshChrome {
  if (self.osc == nil)
    return;
  screenwide_region_osc_ocr_update_appearance(self.osc);
  self.osc.rulerLabel = nil;
  screenwide_region_osc_ruler_update_appearance(self.osc);
  screenwide_region_osc_draw(self.osc);
}

- (void)viewDidChangeEffectiveAppearance {
  [super viewDidChangeEffectiveAppearance];
  [self refreshChrome];
}

- (void)systemColorsDidChange:(NSNotification *)notification {
  (void)notification;
  // The accent cache is shared with the Rust palettes, and the notification
  // reaches every observer in an unspecified order, so it is dropped here
  // rather than relying on the Rust observer having run first.
  screenwide_system_accent_invalidate();
  [self refreshChrome];
}
@end

void screenwide_region_osc_appearance_install(ScreenwideRegionOSC *s) {
  ScreenwideRegionAppearanceObserverView *observer =
      [[ScreenwideRegionAppearanceObserverView alloc]
          initWithFrame:NSMakeRect(0, 0, 1, 1)];
  observer.osc = s;
  s.appearanceObserver = observer;
  [s.host addSubview:observer];
  [NSNotificationCenter.defaultCenter
      addObserver:observer
         selector:@selector(systemColorsDidChange:)
             name:NSSystemColorsDidChangeNotification
           object:nil];
}

void screenwide_region_osc_appearance_teardown(ScreenwideRegionOSC *s) {
  [NSNotificationCenter.defaultCenter removeObserver:s.appearanceObserver];
  [s.appearanceObserver removeFromSuperview];
  s.appearanceObserver = nil;
}
