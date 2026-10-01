// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The live overlay's Metal surface, one per display.
//!
//! A surface is a `CAMetalLayer` over the host window's view and nothing else:
//! the annotations it draws come from Rust on every frame, and the drawing itself is
//! the editor's own annotation shader. Nothing is retained between frames, so
//! there is no second copy of the document to keep in step.

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#import <QuartzCore/QuartzCore.h>
#import <objc/runtime.h>

#import "native_overlay_macos.h"
#import "native_overlay_macos_private.h"
#import "native_overlay_macos_shader.h"
#import "../../editor/cursor_export/macos/gpu_compositor/gpu_compositor_macos_annotations.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_curve.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_composite.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_highlight_ink.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_highlight.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_spotlight.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_counter.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_shape.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_draw.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_cursor.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_magnify.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotation_text.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_annotations.h"
#import "../../editor/cursor_export/macos/gpu_compositor/shaders/gpu_compositor_macos_shader_source_types.h"

@interface ScreenwideAnnotateSurface : NSObject
@property(nonatomic, weak) NSView *host;
@property(nonatomic, strong) CAMetalLayer *layer;
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
@property(nonatomic, strong) id<MTLComputePipelineState> pipeline;
@property(nonatomic) uint32_t display;
/// The desktop this display's highlights are recoloured from, which of
/// Rust's revisions it holds, and the same softened for a spotlight's blur,
/// with the revision it was softened from.
@property(nonatomic, strong) id<MTLTexture> underlay;
@property(nonatomic) uint64_t underlayRevision;
@property(nonatomic, strong) id<MTLTexture> softened;
@property(nonatomic) uint64_t softenedRevision;
@end

@implementation ScreenwideAnnotateSurface
@end

static const void *ScreenwideAnnotateSurfaceKey = &ScreenwideAnnotateSurfaceKey;
static NSMutableArray<ScreenwideAnnotateSurface *> *surfaces = nil;
static ScreenwideAnnotateScene sceneCallback = NULL;
static ScreenwideAnnotateUnderlaySource underlayCallback = NULL;

/// The library is this one kernel plus the shared annotation code it calls.
static NSString *shaderSource(void) {
  return GPU_COMPOSITOR_MACOS_SHADER_SOURCE_TYPES
      GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURVE
          GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATIONS
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COUNTER
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_TEXT
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SHAPE
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_DRAW
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURSOR
              GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_MAGNIFY
                  GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COMPOSITE
                      GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT_INK
                      GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT
                          GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SPOTLIGHT
                              SCREENWIDE_ANNOTATE_SHADER_SOURCE;
}

uint32_t screenwide_annotate_shader_check(char *message, uint32_t capacity) {
  id<MTLDevice> device = MTLCreateSystemDefaultDevice();
  if (device == nil)
    return 2;
  NSError *error = nil;
  id<MTLLibrary> library = [device newLibraryWithSource:shaderSource()
                                               options:nil
                                                 error:&error];
  id<MTLFunction> kernel = [library newFunctionWithName:@"annotate_overlay"];
  if (kernel != nil && [device newComputePipelineStateWithFunction:kernel
                                                             error:&error] != nil)
    return 1;
  if (message != NULL && capacity > 0) {
    NSString *reason = error.localizedDescription ?: @"the overlay kernel is missing";
    strlcpy(message, reason.UTF8String, capacity);
  }
  return 0;
}

/// `rgba`, `width` by `height`, in `texture` - made again where it is
/// missing or a different size.
static id<MTLTexture> uploaded(ScreenwideAnnotateSurface *surface, id<MTLTexture> texture,
                               const uint8_t *rgba, uint32_t width, uint32_t height) {
  if (texture == nil || texture.width != width || texture.height != height) {
    MTLTextureDescriptor *descriptor =
        [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                                           width:width
                                                          height:height
                                                       mipmapped:NO];
    descriptor.usage = MTLTextureUsageShaderRead;
    texture = [surface.device newTextureWithDescriptor:descriptor];
  }
  [texture replaceRegion:MTLRegionMake2D(0, 0, width, height)
             mipmapLevel:0
               withBytes:rgba
             bytesPerRow:width * 4u];
  return texture;
}

/// A single clear pixel, bound where there is nothing to read: no highlight
/// or blur is ever drawn over one.
static id<MTLTexture> emptyTexture(ScreenwideAnnotateSurface *surface) {
  MTLTextureDescriptor *empty =
      [MTLTextureDescriptor texture2DDescriptorWithPixelFormat:MTLPixelFormatRGBA8Unorm
                                                         width:1
                                                        height:1
                                                     mipmapped:NO];
  return [surface.device newTextureWithDescriptor:empty];
}

/// Brings this display's underlay, and its softened copy while a spotlight
/// blurs, up to what Rust holds, uploading each only when it has changed.
static void refreshUnderlay(ScreenwideAnnotateSurface *surface) {
  ScreenwideAnnotateUnderlay underlay = {0};
  if (underlayCallback != NULL)
    underlayCallback(surface.display, &underlay);
  BOOL fresh = underlay.rgba != NULL && underlay.width > 0 && underlay.height > 0;
  if (!fresh) {
    surface.underlay = nil;
    surface.softened = nil;
    return;
  }
  if (underlay.revision != surface.underlayRevision || surface.underlay == nil) {
    surface.underlay = uploaded(surface, surface.underlay, underlay.rgba, underlay.width,
                                underlay.height);
    surface.underlayRevision = underlay.revision;
  }
  BOOL soft = underlay.soft_rgba != NULL && underlay.soft_width > 0 && underlay.soft_height > 0;
  if (!soft) {
    surface.softened = nil;
  } else if (underlay.revision != surface.softenedRevision || surface.softened == nil) {
    surface.softened = uploaded(surface, surface.softened, underlay.soft_rgba,
                                underlay.soft_width, underlay.soft_height);
    surface.softenedRevision = underlay.revision;
  }
}

static void drawSurface(ScreenwideAnnotateSurface *surface) {
  if (surface.layer == nil || surface.pipeline == nil || sceneCallback == NULL)
    return;
  CGSize size = surface.layer.drawableSize;
  if (size.width < 1.0 || size.height < 1.0)
    return;
  id<CAMetalDrawable> drawable = [surface.layer nextDrawable];
  if (drawable == nil)
    return;

  ScreenwideAnnotations annotations = {0};
  sceneCallback(surface.display, &annotations);

  // Annotations arrive in this display's layer pixels, so the canvas placement is
  // one drawn pixel per annotation pixel with no offset.
  ScreenwideCanvas canvas = {0};
  canvas.image_width = 1;
  canvas.image_height = 1;

  id<MTLCommandBuffer> buffer = [surface.queue commandBuffer];
  id<MTLComputeCommandEncoder> encoder = [buffer computeCommandEncoder];
  [encoder setComputePipelineState:surface.pipeline];
  [encoder setBytes:&canvas length:sizeof(canvas) atIndex:0];
  uint32_t above_camera = 0;
  [encoder setBytes:&above_camera length:sizeof(above_camera) atIndex:14];
  [encoder setTexture:drawable.texture atIndex:0];
  refreshUnderlay(surface);
  id<MTLTexture> empty = surface.underlay == nil || surface.softened == nil
      ? emptyTexture(surface) : nil;
  [encoder setTexture:surface.underlay ?: empty atIndex:1];
  [encoder setTexture:surface.softened ?: empty atIndex:2];
  screenwide_bind_annotations(encoder, &annotations, &canvas, 1, 1, 1.0f);
  MTLSize threads = MTLSizeMake((NSUInteger)size.width, (NSUInteger)size.height, 1);
  NSUInteger width = surface.pipeline.threadExecutionWidth;
  NSUInteger height = surface.pipeline.maxTotalThreadsPerThreadgroup / MAX(width, 1u);
  [encoder dispatchThreads:threads
      threadsPerThreadgroup:MTLSizeMake(width, MAX(height, 1u), 1)];
  [encoder endEncoding];
  [buffer presentDrawable:drawable];
  [buffer commit];
}

uint32_t screenwide_annotate_attach(void *view_ptr, uint32_t display) {
  NSView *view = (__bridge NSView *)view_ptr;
  if (view == nil || !NSThread.isMainThread)
    return 0;
  ScreenwideAnnotateSurface *surface = [ScreenwideAnnotateSurface new];
  surface.host = view;
  surface.display = display;
  surface.device = MTLCreateSystemDefaultDevice();
  surface.queue = [surface.device newCommandQueue];
  NSError *error = nil;
  id<MTLLibrary> library = [surface.device newLibraryWithSource:shaderSource()
                                                       options:nil
                                                         error:&error];
  id<MTLFunction> kernel = [library newFunctionWithName:@"annotate_overlay"];
  if (surface.device == nil || surface.queue == nil || kernel == nil)
    return 0;
  surface.pipeline = [surface.device newComputePipelineStateWithFunction:kernel
                                                                  error:&error];
  if (surface.pipeline == nil)
    return 0;

  CGFloat scale = view.window.backingScaleFactor > 0 ? view.window.backingScaleFactor : 1.0;
  CAMetalLayer *layer = [CAMetalLayer layer];
  layer.device = surface.device;
  layer.pixelFormat = MTLPixelFormatBGRA8Unorm;
  // The kernel writes the drawable directly, and the desktop shows through
  // every pixel it does not cover.
  layer.framebufferOnly = NO;
  layer.opaque = NO;
  layer.contentsScale = scale;
  layer.frame = view.bounds;
  layer.drawableSize =
      CGSizeMake(view.bounds.size.width * scale, view.bounds.size.height * scale);
  layer.autoresizingMask = kCALayerWidthSizable | kCALayerHeightSizable;
  surface.layer = layer;
  view.wantsLayer = YES;
  [view.layer addSublayer:layer];

  if (surfaces == nil)
    surfaces = [NSMutableArray array];
  [surfaces addObject:surface];
  objc_setAssociatedObject(view, ScreenwideAnnotateSurfaceKey, surface,
                           OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  drawSurface(surface);
  return 1;
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

void screenwide_annotate_redraw(void) {
  if (!NSThread.isMainThread)
    return;
  for (ScreenwideAnnotateSurface *surface in [surfaces copy])
    drawSurface(surface);
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

void screenwide_annotate_install_scene(ScreenwideAnnotateScene scene,
                                       ScreenwideAnnotateUnderlaySource underlay) {
  if (!NSThread.isMainThread)
    return;
  sceneCallback = scene;
  underlayCallback = underlay;
}
