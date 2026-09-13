// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <objc/runtime.h>

@implementation ScreenwideRegionOSC
- (void)dealloc {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(self);
  if (root == self) {
    for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(self))
      screenwide_region_osc_cursor_release(surface);
    if (self.screenObserver)
      [NSNotificationCenter.defaultCenter removeObserver:self.screenObserver];
    for (ScreenwideRegionOSC *peer in self.desktopPeers) {
      peer.input = NULL;
      peer.rustContext = NULL;
    }
    for (NSWindow *window in self.desktopWindows)
      [window close];
  }
  if (self.appearanceObserver)
    screenwide_region_osc_appearance_teardown(self);
  screenwide_region_osc_input_teardown(self);
  screenwide_region_osc_ruler_teardown(self);
  if (self.releaseContext && self.rustContext)
    self.releaseContext(self.rustContext);
}
@end

static const void *ScreenwideRegionOSCKey = &ScreenwideRegionOSCKey;

void *screenwide_region_osc_attach(void *view_ptr, void *context,
                                   void (*release)(void *),
                                   NativeOscInput input,
                                   NativeOscLayout layout_changed) {
  NSView *view = (__bridge NSView *)view_ptr;
  if (!view) {
    if (release)
      release(context);
    return NULL;
  }
  ScreenwideRegionOSC *s = [ScreenwideRegionOSC new];
  s.host = view;
  s.rustContext = context;
  s.releaseContext = release;
  s.input = input;
  s.layoutChanged = layout_changed;
  s.showFrame = YES;
  s.showHandles = YES;
  s.device = MTLCreateSystemDefaultDevice();
  s.queue = [s.device newCommandQueue];
  NSError *error = nil;
  id<MTLLibrary> library =
      [s.device newLibraryWithSource:screenwide_region_osc_shader_source()
                             options:nil
                               error:&error];
  if (!s.device || !s.queue || !library) {
    s.rustContext = NULL;
    s.releaseContext = NULL;
    if (release)
      release(context);
    return NULL;
  }
  s.pipeline = screenwide_region_osc_make_pipeline(s.device, library, &error);
  s.snapshotPipeline =
      screenwide_region_osc_make_snapshot_pipeline(s.device, library, &error);
  s.magnifierPipeline =
      screenwide_region_magnifier_make_pipeline(s.device, library, &error);
  s.placeholder = screenwide_region_osc_make_placeholder(s.device);
  if (!s.pipeline || !s.snapshotPipeline || !s.magnifierPipeline ||
      !s.placeholder) {
    s.rustContext = NULL;
    s.releaseContext = NULL;
    if (release)
      release(context);
    return NULL;
  }
  view.wantsLayer = YES;
  s.snapshotLayer = [CALayer layer];
  s.snapshotLayer.frame = view.bounds;
  s.snapshotLayer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
  s.snapshotLayer.contentsGravity = kCAGravityResize;
  s.snapshotLayer.hidden = YES;
  [view.layer addSublayer:s.snapshotLayer];
  screenwide_region_osc_ruler_attach(s);
  screenwide_region_osc_ocr_attach(s);
  objc_setAssociatedObject(view, ScreenwideRegionOSCKey, s,
                           OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  screenwide_region_osc_appearance_install(s);
  screenwide_region_osc_input_install(s);
  return (__bridge void *)s;
}

ScreenwideRegionOSC *screenwide_region_osc_for_view(void *view_ptr) {
  NSView *view = (__bridge NSView *)view_ptr;
  return objc_getAssociatedObject(view, ScreenwideRegionOSCKey);
}

ScreenwideRegionOSC *screenwide_region_osc_root(ScreenwideRegionOSC *s) {
  return s.desktopRoot ?: s;
}

NSArray<ScreenwideRegionOSC *> *
screenwide_region_osc_surfaces(ScreenwideRegionOSC *s) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  if (root.desktopPeers.count == 0)
    return @[ root ];
  return [@[ root ] arrayByAddingObjectsFromArray:root.desktopPeers];
}

void screenwide_region_osc_apply_region(ScreenwideRegionOSC *s, NSRect region,
                                        BOOL visible) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  if (!visible)
    screenwide_region_osc_cancel_pointer_claim(root);
  root.desktopRegion = region;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    surface.region = NSOffsetRect(region, -surface.desktopOffset.x,
                                  -surface.desktopOffset.y);
    surface.visible = visible;
    if (!visible)
      screenwide_region_osc_cursor_release(surface);
    screenwide_region_osc_draw(surface);
  }
}

void *screenwide_region_osc_context(void *view_ptr) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  return s ? s.rustContext : NULL;
}

int screenwide_region_osc_set(void *view_ptr, double x, double y, double width,
                              double height, int visible) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return 0;
  screenwide_region_osc_apply_region(s, NSMakeRect(x, y, width, height),
                                     visible != 0);
  return 1;
}

static void detach_surface(ScreenwideRegionOSC *s) {
  s.visible = NO;
  s.drawPending = NO;
  screenwide_region_osc_input_teardown(s);
  screenwide_region_osc_appearance_teardown(s);
  screenwide_region_osc_ruler_teardown(s);
  screenwide_region_osc_ocr_teardown(s);
  s.input = NULL;
  s.layoutChanged = NULL;
  s.layer.hidden = YES;
  s.snapshotLayer.hidden = YES;
  s.snapshotLayer.contents = nil;
  s.snapshotTexture = nil;
  s.magnifierSource = nil;
  [s.layer removeFromSuperlayer];
  [s.snapshotLayer removeFromSuperlayer];
  s.layer = nil;
  s.snapshotLayer = nil;
}

void screenwide_region_osc_detach(void *view_ptr) {
  NSView *view = (__bridge NSView *)view_ptr;
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  screenwide_region_osc_cancel_pointer_claim(s);
  if (screenwide_region_osc_root(s) == s) {
    if (s.screenObserver) {
      [NSNotificationCenter.defaultCenter removeObserver:s.screenObserver];
      s.screenObserver = nil;
    }
    for (ScreenwideRegionOSC *peer in s.desktopPeers) {
      detach_surface(peer);
      peer.rustContext = NULL;
    }
    for (NSWindow *window in s.desktopWindows)
      [window close];
    s.desktopPeers = nil;
    s.desktopWindows = nil;
  }
  detach_surface(s);
  objc_setAssociatedObject(view, ScreenwideRegionOSCKey, nil,
                           OBJC_ASSOCIATION_ASSIGN);
}
