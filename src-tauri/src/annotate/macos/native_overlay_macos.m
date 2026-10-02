// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The live overlay's layers, one per display.
//!
//! A surface is a `CAMetalLayer` over the host window's view and nothing else:
//! Rust draws every frame into it through the shared wgpu device, from the
//! annotations it holds, so there is no second copy of the document here.

#import <AppKit/AppKit.h>
#import <QuartzCore/QuartzCore.h>
#import <objc/runtime.h>

#import "native_overlay_macos.h"
#import "native_overlay_macos_private.h"

@interface ScreenwideAnnotateSurface : NSObject
@property(nonatomic, weak) NSView *host;
@property(nonatomic, strong) CAMetalLayer *layer;
@end

@implementation ScreenwideAnnotateSurface
@end

static const void *ScreenwideAnnotateSurfaceKey = &ScreenwideAnnotateSurfaceKey;
static NSMutableArray<ScreenwideAnnotateSurface *> *surfaces = nil;

void *screenwide_annotate_attach(void *view_ptr) {
  NSView *view = (__bridge NSView *)view_ptr;
  if (view == nil || !NSThread.isMainThread)
    return NULL;
  ScreenwideAnnotateSurface *surface = [ScreenwideAnnotateSurface new];
  surface.host = view;
  CGFloat scale = view.window.backingScaleFactor > 0 ? view.window.backingScaleFactor : 1.0;
  // The renderer's surface sets the drawable size, pixel format and opacity;
  // the desktop shows through every pixel the annotations do not cover.
  CAMetalLayer *layer = [CAMetalLayer layer];
  layer.contentsScale = scale;
  layer.frame = view.bounds;
  layer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
  surface.layer = layer;
  view.wantsLayer = YES;
  [view.layer addSublayer:layer];

  if (surfaces == nil)
    surfaces = [NSMutableArray array];
  [surfaces addObject:surface];
  objc_setAssociatedObject(view, ScreenwideAnnotateSurfaceKey, surface,
                           OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  return (__bridge void *)layer;
}

void screenwide_annotate_detach(void *view_ptr) {
  NSView *view = (__bridge NSView *)view_ptr;
  if (view == nil || !NSThread.isMainThread)
    return;
  ScreenwideAnnotateSurface *surface =
      objc_getAssociatedObject(view, ScreenwideAnnotateSurfaceKey);
  if (surface == nil)
    return;
  [surface.layer removeFromSuperlayer];
  surface.layer = nil;
  [surfaces removeObject:surface];
  objc_setAssociatedObject(view, ScreenwideAnnotateSurfaceKey, nil,
                           OBJC_ASSOCIATION_RETAIN_NONATOMIC);
}

/// Whether an event landed on one of the surfaces the overlay draws on. The
/// monitor sees every event in the process, so this is a pointer compare over
/// the one window per display the overlay owns.
BOOL screenwide_annotate_owns_window(NSWindow *window) {
  for (ScreenwideAnnotateSurface *surface in surfaces)
    if (surface.host.window == window)
      return YES;
  return NO;
}

BOOL screenwide_annotate_has_surfaces(void) { return surfaces.count > 0; }
