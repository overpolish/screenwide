// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <QuartzCore/CATransaction.h>

_Static_assert(sizeof(ScreenwideOscIconAtlas) == 32,
               "ScreenwideOscIconAtlas ABI size drift");
_Static_assert(sizeof(ScreenwideOscConfirmSpec) == 8,
               "ScreenwideOscConfirmSpec ABI size drift");
_Static_assert(sizeof(ScreenwideOscConfirmUpdate) == 4,
               "ScreenwideOscConfirmUpdate ABI size drift");
_Static_assert(sizeof(ScreenwideOscConfirmLayer) == 28,
               "ScreenwideOscConfirmLayer ABI size drift");
static uint32_t light_mode(ScreenwideRegionOSC *surface) {
  NSString *appearance = [surface.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  return [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
}

static ScreenwideOscControlMetrics button_metrics(void) {
  return screenwide_osc_control_metrics(0, 0);
}

static ScreenwideOscControlMetrics icon_metrics(void) {
  return screenwide_osc_control_metrics(1, 0);
}

static void update_labels(ScreenwideRegionOSC *surface) {
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  uint32_t light = light_mode(surface);
  if (surface.ocrToolbarLabels && surface.ocrToolbarLabelScale == scale &&
      surface.ocrToolbarLabelLightMode == light)
    return;
  ScreenwideOscControlMetrics metrics = button_metrics();
  surface.ocrToolbarLabels = @[
    screenwide_osc_text_texture(surface.device, @"Copy all", scale, light,
                                metrics.font_size, metrics.line_height),
    screenwide_osc_text_texture(surface.device, @"Copy as paragraph", scale,
                                light, metrics.font_size,
                                metrics.line_height),
  ];
  surface.ocrToolbarLabelScale = scale;
  surface.ocrToolbarLabelLightMode = light;
}

uint8_t screenwide_region_osc_ocr_toolbar_icon(
    ScreenwideRegionOSC *surface, NSUInteger index) {
  (void)surface;
  if (index == 0)
    return 2;
  if (index == 1)
    return 3;
  if (index == 2)
    return 4;
  return 0;
}

static void render_control(ScreenwideRegionOSC *surface, NSUInteger index) {
  if (!surface.ocrToolbarVisible || !surface.ocrToolbarControls ||
      index >= surface.ocrToolbarSurfaces.count)
    return;
  ScreenwideOscMaterialSurfaceView *control =
      surface.ocrToolbarSurfaces[index];
  if (control.hidden)
    return;
  ScreenwideOscControlVisual visuals[4] = {0};
  uint32_t light = light_mode(surface);
  if (screenwide_osc_control_group_visuals(surface.ocrToolbarControls,
                                            light == 0, visuals, 4) != 4)
    return;
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light);
  memcpy(state.action_fills, visuals[index].fill,
         sizeof(visuals[index].fill));
  memcpy(state.action_fills + 4, visuals[index].foreground,
         sizeof(visuals[index].foreground));
  NSSize size = control.bounds.size;
  ScreenwideRegionOscVertex vertices[96];
  NSUInteger count = 0;
  screenwide_region_osc_add_quad(
      vertices, &count, size, NSMakeRect(0, 0, size.width, size.height), 12);
  BOOL button = index < 2;
  ScreenwideOscControlMetrics metrics =
      button ? button_metrics() : icon_metrics();
  CGFloat iconLeft =
      button ? metrics.padding_x : (size.width - metrics.icon_size) * 0.5;
  screenwide_region_osc_add_icon(
      vertices, &count, size,
      screenwide_region_osc_ocr_toolbar_icon(surface, index), iconLeft,
      (size.height - metrics.icon_size) * 0.5, metrics.icon_size);
  ScreenwideOscTextTexture *label = nil;
  if (button) {
    label = surface.ocrToolbarLabels[index];
    screenwide_region_osc_add_quad(
        vertices, &count, size,
        NSMakeRect(metrics.padding_x + metrics.icon_size + metrics.gap,
                   (size.height - label.size.height) * 0.5,
                   label.size.width, label.size.height),
        11);
  }
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
  screenwide_region_osc_encode(
      encoder, surface.pipeline, buffer, count, state,
      label.texture ?: surface.placeholder, surface.placeholder);
  if (index == 3 && surface.ocrToolbarConfirm) {
    ScreenwideOscConfirmLayer layers[2] = {0};
    size_t layerCount = screenwide_osc_confirm_layers(
        surface.ocrToolbarConfirm, light == 0, layers, 2);
    for (size_t layerIndex = 0; layerIndex < layerCount; layerIndex++) {
      ScreenwideOscConfirmLayer layer = layers[layerIndex];
      if (layer.opacity <= 0.002 || layer.scale <= 0.002)
        continue;
      CGFloat iconSize = metrics.icon_size * layer.scale;
      ScreenwideRegionOscVertex iconVertices[6];
      NSUInteger iconCount = 0;
      screenwide_region_osc_add_icon(
          iconVertices, &iconCount, size, layer.icon,
          (size.width - iconSize) * 0.5, (size.height - iconSize) * 0.5,
          iconSize);
      id<MTLBuffer> iconBuffer = [surface.device
          newBufferWithBytes:iconVertices
                       length:sizeof(ScreenwideRegionOscVertex) * iconCount
                      options:MTLResourceStorageModeShared];
      memcpy(state.action_fills + 4, layer.foreground,
             sizeof(layer.foreground));
      state.action_fills[7] *= layer.opacity;
      [encoder setFragmentBytes:state.action_fills
                         length:sizeof(state.action_fills)
                        atIndex:2];
      [encoder setVertexBuffer:iconBuffer offset:0 atIndex:0];
      [encoder drawPrimitives:MTLPrimitiveTypeTriangle
                  vertexStart:0
                  vertexCount:iconCount];
    }
  }
  [encoder endEncoding];
  [command presentDrawable:drawable];
  [command commit];
}

void screenwide_region_osc_ocr_toolbar_render(ScreenwideRegionOSC *surface) {
  for (NSUInteger index = 0; index < 4; index++)
    render_control(surface, index);
}

static void schedule_frame(ScreenwideRegionOSC *surface, uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.ocrToolbarAnimationRevision != revision)
                     return;
                   screenwide_region_osc_ocr_toolbar_render(surface);
                   if (screenwide_osc_control_group_is_animating(
                           surface.ocrToolbarControls) ||
                       screenwide_osc_confirm_is_animating(
                           surface.ocrToolbarConfirm))
                     schedule_frame(surface, revision);
                 });
}

void screenwide_region_osc_ocr_toolbar_apply_confirm_update(
    ScreenwideRegionOSC *surface, ScreenwideOscConfirmUpdate update) {
  surface.ocrToolbarCloseArmed = update.armed != 0;
  surface.ocrToolbarSurfaces[3].accessibilityLabel =
      surface.ocrToolbarCloseArmed ? @"Confirm closing text recognition"
                                   : @"Close text recognition";
  if (!update.changed)
    return;
  screenwide_region_osc_ocr_toolbar_render(surface);
  if (update.animating) {
    uint64_t revision = ++surface.ocrToolbarAnimationRevision;
    schedule_frame(surface, revision);
  }
}

void screenwide_region_osc_ocr_toolbar_apply_update(
    ScreenwideRegionOSC *surface, ScreenwideOscControlUpdate update) {
  if (!update.changed)
    return;
  screenwide_region_osc_ocr_toolbar_render(surface);
  if (update.animating) {
    uint64_t revision = ++surface.ocrToolbarAnimationRevision;
    schedule_frame(surface, revision);
  }
}

void screenwide_region_osc_ocr_toolbar_layout(ScreenwideRegionOSC *surface,
                                              BOOL visible) {
  surface.ocrToolbarVisible = visible;
  if (!visible) {
    surface.ocrToolbarAnimationRevision += 1;
    for (ScreenwideOscMaterialSurfaceView *control in
         surface.ocrToolbarSurfaces)
      control.hidden = YES;
    return;
  }
  update_labels(surface);
  ScreenwideOscControlMetrics button = button_metrics();
  ScreenwideOscControlMetrics icon = icon_metrics();
  double widths[4] = {
      button.padding_x * 2.0 + button.icon_size + button.gap +
          surface.ocrToolbarLabels[0].size.width,
      button.padding_x * 2.0 + button.icon_size + button.gap +
          surface.ocrToolbarLabels[1].size.width,
      icon.height,
      icon.height,
  };
  ScreenwideOcrToolbarRect rects[4] = {0};
  NSSize host = surface.host.bounds.size;
  screenwide_ocr_toolbar_layout(
      NSMinX(surface.region), NSMinY(surface.region),
      NSWidth(surface.region), NSHeight(surface.region), host.width,
      host.height, widths, button.height, rects, 4);
  ScreenwideOscControlSpec specs[4] = {0};
  for (NSUInteger index = 0; index < 4; index++) {
    specs[index] = (ScreenwideOscControlSpec){
        rects[index].x, rects[index].y, rects[index].width,
        rects[index].height, index < 2 ? 0 : 1,
        0, 0, 0,
        screenwide_region_osc_ocr_toolbar_icon(surface, index)};
  }
  screenwide_osc_control_group_layout(surface.ocrToolbarControls, specs, 4);
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  for (NSUInteger index = 0; index < 4; index++) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.ocrToolbarSurfaces[index];
    control.hidden = NO;
    control.frame = NSMakeRect(rects[index].x,
                               host.height - rects[index].y -
                                   rects[index].height,
                               rects[index].width, rects[index].height);
    CGFloat radius = index < 2 ? button.radius : icon.radius;
    control.layer.cornerRadius = radius;
    control.contentLayer.cornerRadius = radius;
    control.contentView.frame = control.bounds;
  }
  surface.ocrToolbarSurfaces[3].accessibilityLabel =
      surface.ocrToolbarCloseArmed ? @"Confirm closing text recognition"
                                   : @"Close text recognition";
  [CATransaction commit];
  screenwide_region_osc_ocr_toolbar_render(surface);
}

void screenwide_region_osc_ocr_toolbar_attach(ScreenwideRegionOSC *surface) {
  surface.ocrToolbarControls = screenwide_osc_control_group_create();
  surface.ocrToolbarConfirm = screenwide_osc_confirm_create(
      (ScreenwideOscConfirmSpec){1, 5, 0, 2, 2000});
  surface.ocrToolbarSurfaces = [NSMutableArray arrayWithCapacity:4];
  for (NSString *label in @[
         @"Copy all", @"Copy as paragraph", @"Recognize another area",
         @"Close text recognition"
       ]) {
    ScreenwideOscMaterialSurfaceView *control =
        screenwide_osc_material_surface(surface.device);
    control.accessibilityElement = YES;
    control.accessibilityRole = NSAccessibilityButtonRole;
    control.accessibilityLabel = label;
    [surface.ocrToolbarSurfaces addObject:control];
    [surface.host addSubview:control positioned:NSWindowAbove relativeTo:nil];
  }
}

void screenwide_region_osc_ocr_toolbar_teardown(ScreenwideRegionOSC *surface) {
  for (ScreenwideOscMaterialSurfaceView *control in
       surface.ocrToolbarSurfaces)
    [control removeFromSuperview];
  surface.ocrToolbarSurfaces = nil;
  if (surface.ocrToolbarControls)
    screenwide_osc_control_group_destroy(surface.ocrToolbarControls);
  surface.ocrToolbarControls = NULL;
  if (surface.ocrToolbarConfirm)
    screenwide_osc_confirm_destroy(surface.ocrToolbarConfirm);
  surface.ocrToolbarConfirm = NULL;
  surface.ocrToolbarLabels = nil;
  surface.ocrToolbarVisible = NO;
  surface.ocrToolbarCloseArmed = NO;
  surface.ocrToolbarCloseRevision += 1;
}
