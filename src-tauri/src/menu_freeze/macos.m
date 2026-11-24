// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#import <stdatomic.h>

// Above the menu bar (24) and the status items (25, ordered in front of
// them), under the island (28 to 31) and the capture tools (33), so a tool
// works over the picture and its island stays in front.
static const NSInteger backdropLevel = 25;

// The menu AppKit is tracking, the tray's or one in the menu bar; nil while
// none is open. Touched on the main thread only; `menuOpen` mirrors it for
// the threads a shortcut may arrive on.
static NSMenu *tracking;
static atomic_bool menuOpen;
static NSMutableArray<NSPanel *> *backdrops;

// `changed` is told, on the main thread, each time a menu opens or closes.
void screenwide_menu_freeze_observe(void (*changed)(bool open)) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSNotificationCenter *center = NSNotificationCenter.defaultCenter;
    [center addObserverForName:NSMenuDidBeginTrackingNotification object:nil queue:nil
      usingBlock:^(NSNotification *notification) {
        // A submenu tracked inside the open menu does not replace it.
        if (tracking) return;
        tracking = notification.object;
        atomic_store(&menuOpen, true);
        changed(true);
      }];
    [center addObserverForName:NSMenuDidEndTrackingNotification object:nil queue:nil
      usingBlock:^(NSNotification *notification) {
        if (notification.object != tracking) return;
        tracking = nil;
        atomic_store(&menuOpen, false);
        changed(false);
      }];
  });
}

bool screenwide_menu_freeze_menu_open(void) {
  return atomic_load(&menuOpen);
}

void screenwide_menu_freeze_close_menu(void) {
  [tracking cancelTrackingWithoutAnimation];
}

// Puts one display's frozen picture in front of the desktop. `x`, `y`,
// `width` and `height` are the display's frame in global points from the
// top left; `rgba` is the still, sRGB, `pixelsWide` by `pixelsHigh`.
void screenwide_menu_freeze_show(const uint8_t *rgba, size_t pixelsWide, size_t pixelsHigh,
  double x, double y, double width, double height) {
  @autoreleasepool {
    CFDataRef data = CFDataCreate(NULL, rgba, (CFIndex)(pixelsWide * pixelsHigh * 4));
    CGDataProviderRef provider = CGDataProviderCreateWithCFData(data);
    CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGImageRef image = CGImageCreate(pixelsWide, pixelsHigh, 8, 32, pixelsWide * 4, space,
      kCGImageAlphaNoneSkipLast | kCGBitmapByteOrderDefault, provider, NULL, false,
      kCGRenderingIntentDefault);
    CGColorSpaceRelease(space);
    CGDataProviderRelease(provider);
    CFRelease(data);
    if (!image) return;

    CGFloat primaryHeight = NSScreen.screens.firstObject.frame.size.height;
    NSRect frame = NSMakeRect(x, primaryHeight - y - height, width, height);
    NSPanel *panel = [[NSPanel alloc] initWithContentRect:frame
      styleMask:NSWindowStyleMaskBorderless | NSWindowStyleMaskNonactivatingPanel
      backing:NSBackingStoreBuffered defer:NO];
    panel.releasedWhenClosed = NO;
    panel.level = backdropLevel;
    panel.opaque = YES;
    panel.hasShadow = NO;
    panel.ignoresMouseEvents = YES;
    panel.hidesOnDeactivate = NO;
    panel.animationBehavior = NSWindowAnimationBehaviorNone;
    panel.collectionBehavior = NSWindowCollectionBehaviorCanJoinAllSpaces |
      NSWindowCollectionBehaviorFullScreenAuxiliary | NSWindowCollectionBehaviorStationary;
    NSView *view = panel.contentView;
    view.wantsLayer = YES;
    view.layer.contents = (__bridge id)image;
    view.layer.contentsGravity = kCAGravityResize;
    CGImageRelease(image);
    [panel orderFrontRegardless];
    if (!backdrops) backdrops = [NSMutableArray array];
    [backdrops addObject:panel];
  }
}

void screenwide_menu_freeze_hide(void) {
  for (NSPanel *panel in backdrops) [panel close];
  [backdrops removeAllObjects];
}
