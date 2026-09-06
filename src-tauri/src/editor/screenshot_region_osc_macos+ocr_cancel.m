// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <QuartzCore/CATransaction.h>

static uint32_t light_mode(ScreenwideRegionOSC *surface) {
  NSString *appearance = [surface.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  return [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
}

static ScreenwideOscControlMetrics metrics(void) {
  return screenwide_osc_control_metrics(0, 1);
}

static void update_label(ScreenwideRegionOSC *surface) {
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  uint32_t light = light_mode(surface);
  if (surface.ocrCancelLabel && surface.ocrCancelLabelScale == scale &&
      surface.ocrCancelLabelLightMode == light)
    return;
  ScreenwideOscControlMetrics value = metrics();
  surface.ocrCancelLabel = screenwide_osc_text_texture(
      surface.device, @"Cancel", scale, light, value.font_size,
      value.line_height);
  surface.ocrCancelLabelScale = scale;
  surface.ocrCancelLabelLightMode = light;
}

static void render(ScreenwideRegionOSC *surface) {
  ScreenwideOscMaterialSurfaceView *control = surface.ocrCancelSurface;
  if (!surface.ocrCancelVisible || control.hidden ||
      !surface.ocrCancelControls)
    return;
  ScreenwideOscControlVisual visual = {0};
  uint32_t light = light_mode(surface);
  if (screenwide_osc_control_group_visuals(surface.ocrCancelControls,
                                            light == 0, &visual, 1) != 1)
    return;
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light);
  memcpy(state.action_fills, visual.fill, sizeof(visual.fill));
  memcpy(state.action_fills + 4, visual.foreground,
         sizeof(visual.foreground));
  NSSize size = control.bounds.size;
  ScreenwideRegionOscVertex vertices[48];
  NSUInteger count = 0;
  screenwide_region_osc_add_quad(
      vertices, &count, size, NSMakeRect(0, 0, size.width, size.height), 12);
  ScreenwideOscControlMetrics value = metrics();
  screenwide_region_osc_add_icon(vertices, &count, size, 1, value.padding_x,
                                 (size.height - value.icon_size) * 0.5,
                                 value.icon_size);
  NSSize labelSize = surface.ocrCancelLabel.size;
  screenwide_region_osc_add_quad(
      vertices, &count, size,
      NSMakeRect(value.padding_x + value.icon_size + value.gap,
                 (size.height - labelSize.height) * 0.5,
                 labelSize.width, labelSize.height),
      11);
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  control.contentLayer.contentsScale = scale;
  control.contentLayer.drawableSize =
      CGSizeMake(MAX(size.width * scale, 2.0),
                 MAX(size.height * scale, 2.0));
  id<CAMetalDrawable> drawable = [control.contentLayer nextDrawable];
  if (!drawable)
    return;
  id<MTLBuffer> buffer = [surface.device
      newBufferWithBytes:vertices
                   length:sizeof(ScreenwideRegionOscVertex) * count
                  options:MTLResourceStorageModeShared];
  MTLRenderPassDescriptor *pass =
      [MTLRenderPassDescriptor renderPassDescriptor];
  pass.colorAttachments[0].texture = drawable.texture;
  pass.colorAttachments[0].loadAction = MTLLoadActionClear;
  pass.colorAttachments[0].storeAction = MTLStoreActionStore;
  pass.colorAttachments[0].clearColor = MTLClearColorMake(0, 0, 0, 0);
  id<MTLCommandBuffer> command = [surface.queue commandBuffer];
  id<MTLRenderCommandEncoder> encoder =
      [command renderCommandEncoderWithDescriptor:pass];
  screenwide_region_osc_encode(encoder, surface.pipeline, buffer, count, state,
                               surface.ocrCancelLabel.texture ?:
                                   surface.placeholder,
                               surface.placeholder);
  [encoder endEncoding];
  [command presentDrawable:drawable];
  [command commit];
}

static void schedule_frame(ScreenwideRegionOSC *surface, uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.ocrCancelAnimationRevision != revision)
                     return;
                   render(surface);
                   if (screenwide_osc_control_group_is_animating(
                           surface.ocrCancelControls))
                     schedule_frame(surface, revision);
                 });
}

static void apply_update(ScreenwideRegionOSC *surface,
                         ScreenwideOscControlUpdate update) {
  if (!update.changed)
    return;
  render(surface);
  if (update.animating) {
    uint64_t revision = ++surface.ocrCancelAnimationRevision;
    schedule_frame(surface, revision);
  }
}

static void layout(ScreenwideRegionOSC *surface) {
  update_label(surface);
  ScreenwideOscControlMetrics value = metrics();
  NSSize host = surface.host.bounds.size;
  CGFloat top = 48.0;
  CGFloat width = value.padding_x * 2.0 + value.icon_size + value.gap +
                  surface.ocrCancelLabel.size.width;
  CGFloat left = floor((host.width - width) * 0.5);
  surface.ocrCancelRect = NSMakeRect(left, top, width, value.height);
  ScreenwideOscControlSpec spec = {
      left, top, width, value.height, 0, 0, 1, 0, 1};
  screenwide_osc_control_group_layout(surface.ocrCancelControls, &spec, 1);
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  surface.ocrCancelSurface.frame =
      NSMakeRect(left, host.height - top - value.height, width, value.height);
  surface.ocrCancelSurface.layer.cornerRadius = value.radius;
  surface.ocrCancelSurface.contentLayer.cornerRadius = value.radius;
  surface.ocrCancelSurface.contentView.frame = surface.ocrCancelSurface.bounds;
  [CATransaction commit];
}

BOOL screenwide_region_osc_ocr_cancel_input(ScreenwideRegionOSC *surface,
                                            NSPoint point, uint32_t phase) {
  if (!surface.ocrCancelVisible || !surface.ocrCancelControls)
    return NO;
  ScreenwideOscControlUpdate update = {0};
  if (phase == 1 || phase == 3)
    update = screenwide_osc_control_group_hover(
        surface.ocrCancelControls, point.x, point.y);
  else if (phase == 2)
    update = screenwide_osc_control_group_down(
        surface.ocrCancelControls, point.x, point.y);
  else if (phase == 4)
    update = screenwide_osc_control_group_up(
        surface.ocrCancelControls, point.x, point.y);
  else
    update = screenwide_osc_control_group_clear_hover(
        surface.ocrCancelControls);
  apply_update(surface, update);
  if (update.consumed) {
    screenwide_set_region_expected_cursor(NSCursor.pointingHandCursor);
    [NSCursor.pointingHandCursor set];
  }
  if (update.activated && surface.input && surface.rustContext) {
    NativeOscResult result = {0};
    surface.input(surface.rustContext, 8, 0, 0, 0, &result);
  }
  return update.consumed != 0;
}

void screenwide_region_osc_ocr_set_cancel_visible(void *view_ptr,
                                                   int visible) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(
      screenwide_region_osc_for_view(view_ptr));
  if (!root)
    return;
  for (ScreenwideRegionOSC *surface in
       screenwide_region_osc_surfaces(root)) {
    surface.ocrCancelVisible = visible != 0;
    surface.ocrCancelSurface.hidden = !surface.ocrCancelVisible;
    if (surface.ocrCancelVisible) {
      layout(surface);
      render(surface);
    } else {
      surface.ocrCancelAnimationRevision += 1;
      apply_update(surface, screenwide_osc_control_group_clear_hover(
                                surface.ocrCancelControls));
    }
  }
}

void screenwide_region_osc_ocr_cancel_attach(ScreenwideRegionOSC *surface) {
  surface.ocrCancelControls = screenwide_osc_control_group_create();
  surface.ocrCancelSurface = screenwide_osc_material_surface(surface.device);
  surface.ocrCancelSurface.accessibilityElement = YES;
  surface.ocrCancelSurface.accessibilityRole = NSAccessibilityButtonRole;
  surface.ocrCancelSurface.accessibilityLabel = @"Cancel text recognition";
  [surface.host addSubview:surface.ocrCancelSurface
                positioned:NSWindowAbove relativeTo:nil];
}

void screenwide_region_osc_ocr_cancel_teardown(ScreenwideRegionOSC *surface) {
  [surface.ocrCancelSurface removeFromSuperview];
  surface.ocrCancelSurface = nil;
  if (surface.ocrCancelControls)
    screenwide_osc_control_group_destroy(surface.ocrCancelControls);
  surface.ocrCancelControls = NULL;
  surface.ocrCancelVisible = NO;
  surface.ocrCancelLabel = nil;
}

void screenwide_region_osc_ocr_cancel_update_appearance(
    ScreenwideRegionOSC *surface) {
  if (surface.ocrCancelVisible)
    layout(surface);
  render(surface);
}
