// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <objc/runtime.h>
#include <stdlib.h>

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
  screenwide_region_osc_appearance_teardown(self);
  screenwide_region_osc_input_teardown(self);
  if (self.releaseContext && self.rustContext)
    self.releaseContext(self.rustContext);
}
@end

static const void *ScreenwideRegionOSCKey = &ScreenwideRegionOSCKey;

static uint32_t light_mode(ScreenwideRegionOSC *s) {
  NSString *appearance = [s.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  return [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
}

void screenwide_region_osc_draw(ScreenwideRegionOSC *s) {
  s.drawRevision += 1;
  uint64_t revision = s.drawRevision;
  if (!s.visible || s.layer == nil || s.pipeline == nil) {
    s.drawPending = NO;
    s.layer.hidden = YES;
    return;
  }
  NSSize size = s.host.bounds.size;
  if (size.width <= 0 || size.height <= 0) {
    s.drawPending = NO;
    s.layer.hidden = YES;
    return;
  }
  // Match the export OSC: keep at most one CAMetalDrawable in flight so
  // pointer samples never block AppKit waiting for the display server.
  if (s.drawInFlight) {
    s.drawPending = YES;
    return;
  }
  s.drawInFlight = YES;
  s.drawPending = NO;
  CGFloat scale = s.host.window.backingScaleFactor ?: 1.0;
  s.layer.frame = s.host.bounds;
  s.layer.contentsScale = scale;
  s.layer.drawableSize =
      CGSizeMake(MAX(size.width * scale, 2.0), MAX(size.height * scale, 2.0));

  NSUInteger capacity = 256 + screenwide_region_osc_ocr_vertex_capacity(s);
  ScreenwideRegionOscVertex *vertices =
      calloc(capacity, sizeof(ScreenwideRegionOscVertex));
  if (!vertices) {
    s.drawInFlight = NO;
    return;
  }
  NSUInteger count = 0;
  NSRect canvas = NSMakeRect(0, 0, size.width, size.height);
  if (NSIsEmptyRect(s.region))
    screenwide_region_osc_add_quad(vertices, &count, size, canvas, 6);
  else
    screenwide_region_osc_add_crop_with_handles(
        vertices, &count, size, s.region, canvas, scale, s.showFrame,
        s.showHandles);
  screenwide_region_osc_ocr_add_vertices(s, vertices, &count, size, scale);

  id<CAMetalDrawable> drawable = [s.layer nextDrawable];
  if (drawable == nil) {
    free(vertices);
    s.drawInFlight = NO;
    return;
  }
  id<MTLCommandBuffer> command = [s.queue commandBuffer];
  MTLRenderPassDescriptor *clear =
      [MTLRenderPassDescriptor renderPassDescriptor];
  clear.colorAttachments[0].texture = drawable.texture;
  clear.colorAttachments[0].loadAction = MTLLoadActionClear;
  clear.colorAttachments[0].storeAction = MTLStoreActionStore;
  clear.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLRenderCommandEncoder> clearEncoder =
      [command renderCommandEncoderWithDescriptor:clear];
  [clearEncoder endEncoding];

  BOOL showMagnifier = s.showFrame && s.magnifier.active != 0 &&
                       s.magnifierSource != nil &&
                       s.magnifierPipeline != nil;
  if (showMagnifier) {
    uint32_t dimensions[2] = {s.magnifierSourceWidth,
                              s.magnifierSourceHeight};
    id<MTLComputeCommandEncoder> magnifierEncoder =
        [command computeCommandEncoder];
    screenwide_region_magnifier_encode(
        magnifierEncoder, s.magnifierPipeline, s.magnifierSource,
        drawable.texture, dimensions, s.magnifier);
    [magnifierEncoder endEncoding];
  }

  id<MTLBuffer> buffer =
      [s.device newBufferWithBytes:vertices
                            length:count * sizeof(*vertices)
                           options:MTLResourceStorageModeShared];
  free(vertices);
  MTLRenderPassDescriptor *pass =
      [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionLoad;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  id<MTLRenderCommandEncoder> encoder =
      [command renderCommandEncoderWithDescriptor:pass];
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light_mode(s));
  if (showMagnifier) {
    state.magnifier_box[0] = s.magnifier.box_x;
    state.magnifier_box[1] = s.magnifier.box_y;
    state.magnifier_box[2] = s.magnifier.box_width;
    state.magnifier_box[3] = s.magnifier.box_height;
  }
  screenwide_region_osc_encode(encoder, s.pipeline, buffer, count, state,
                               s.placeholder, s.placeholder);
  [encoder endEncoding];
  [command presentDrawable:drawable];
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    dispatch_async(dispatch_get_main_queue(), ^{
      s.drawInFlight = NO;
      BOOL redrawPending = s.drawPending;
      s.drawPending = NO;
      if (s.visible)
        s.layer.hidden = NO;
      if (redrawPending) {
        screenwide_region_osc_draw(s);
      } else if (s.visible && s.drawRevision == revision) {
        s.layer.hidden = NO;
      }
    });
  }];
  [command commit];
}

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
  s.magnifierPipeline =
      screenwide_region_magnifier_make_pipeline(s.device, library, &error);
  s.placeholder = screenwide_region_osc_make_placeholder(s.device);
  if (!s.pipeline || !s.magnifierPipeline || !s.placeholder) {
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
  s.layer = [CAMetalLayer layer];
  s.layer.device = s.device;
  s.layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
  s.layer.framebufferOnly = NO;
  s.layer.opaque = NO;
  s.layer.frame = view.bounds;
  s.layer.contentsScale = view.window.backingScaleFactor ?: 1;
  s.layer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
  [view.layer addSublayer:s.layer];
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

void screenwide_region_osc_detach(void *view_ptr) {
  NSView *view = (__bridge NSView *)view_ptr;
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (s) {
    screenwide_region_osc_cancel_pointer_claim(s);
    for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(s))
      screenwide_region_osc_cursor_release(surface);
  }
  if (screenwide_region_osc_root(s) == s) {
    for (ScreenwideRegionOSC *peer in s.desktopPeers) {
      peer.input = NULL;
      peer.rustContext = NULL;
    }
    for (NSWindow *window in s.desktopWindows)
      [window close];
    s.desktopPeers = nil;
    s.desktopWindows = nil;
  }
  s.visible = NO;
  s.drawPending = NO;
  s.layer.hidden = YES;
  s.snapshotLayer.hidden = YES;
  s.snapshotLayer.contents = nil;
  screenwide_region_osc_ocr_teardown(s);
  [s.layer removeFromSuperlayer];
  [s.snapshotLayer removeFromSuperlayer];
  objc_setAssociatedObject(view, ScreenwideRegionOSCKey, nil,
                           OBJC_ASSOCIATION_ASSIGN);
}
