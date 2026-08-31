// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

@interface ScreenwideRegionAppearanceObserverView : NSView
@property(nonatomic, weak) ScreenwideRegionOSC *osc;
@end

@implementation ScreenwideRegionAppearanceObserverView
- (NSView *)hitTest:(NSPoint)point {
  (void)point;
  return nil;
}

- (void)viewDidChangeEffectiveAppearance {
  [super viewDidChangeEffectiveAppearance];
  if (self.osc != nil) {
    screenwide_region_osc_ocr_update_appearance(self.osc);
    self.osc.rulerLabel = nil;
    screenwide_region_osc_ruler_update_appearance(self.osc);
    screenwide_region_osc_draw(self.osc);
  }
}
@end

void screenwide_region_osc_appearance_install(ScreenwideRegionOSC *s) {
  ScreenwideRegionAppearanceObserverView *observer =
      [[ScreenwideRegionAppearanceObserverView alloc]
          initWithFrame:NSMakeRect(0, 0, 1, 1)];
  observer.osc = s;
  s.appearanceObserver = observer;
  [s.host addSubview:observer];
}

void screenwide_region_osc_appearance_teardown(ScreenwideRegionOSC *s) {
  [s.appearanceObserver removeFromSuperview];
  s.appearanceObserver = nil;
}
