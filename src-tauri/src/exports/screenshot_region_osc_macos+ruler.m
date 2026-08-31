// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <QuartzCore/CATransaction.h>
#import <objc/runtime.h>

static const CFTimeInterval kRulerAnimationDuration = 0.160;
static const CFTimeInterval kRulerHoverPulseDuration =
    kRulerAnimationDuration * 4.0;
static const NSUInteger kRulerAtlasCells = 22;

@interface ScreenwideRulerLabelRenderState : NSObject
@property(nonatomic) BOOL inFlight;
@property(nonatomic) BOOL pending;
@property(nonatomic) NativeRulerMeasurement measurement;
@property(nonatomic) NativeRulerProbe probe;
@property(nonatomic) NativeRulerRadius radius;
@property(nonatomic) uint8_t labelKind;
@property(nonatomic, weak) ScreenwideRegionOSC *surface;
@property(nonatomic, weak) ScreenwideOscMaterialSurfaceView *control;
@end

@implementation ScreenwideRulerLabelRenderState
@end

static const void *ScreenwideRulerLabelRenderStateKey =
    &ScreenwideRulerLabelRenderStateKey;

static ScreenwideRulerLabelRenderState *label_render_state(
    ScreenwideRegionOSC *surface,
    ScreenwideOscMaterialSurfaceView *control) {
  ScreenwideRulerLabelRenderState *state =
      objc_getAssociatedObject(control, ScreenwideRulerLabelRenderStateKey);
  if (!state) {
    state = [ScreenwideRulerLabelRenderState new];
    state.surface = surface;
    state.control = control;
    objc_setAssociatedObject(control, ScreenwideRulerLabelRenderStateKey,
                             state, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  }
  return state;
}

BOOL screenwide_region_osc_ruler_label_hit(
    ScreenwideRegionOSC *surface, NSPoint point,
    ScreenwideRulerLabelHit *hit) {
  if (!surface || !surface.rulerVisible || !hit)
    return NO;
  memset(hit, 0, sizeof(*hit));
  NSPoint appKitPoint =
      NSMakePoint(point.x, NSHeight(surface.host.bounds) - point.y);
  for (ScreenwideOscMaterialSurfaceView *control in
       surface.rulerMeasurementLabelSurfaces) {
    ScreenwideRulerLabelRenderState *state =
        objc_getAssociatedObject(control, ScreenwideRulerLabelRenderStateKey);
    if (state.labelKind == 1 && state.measurement.id != 0 && !control.hidden &&
        NSPointInRect(appKitPoint, control.frame)) {
      hit->id = state.measurement.id;
      hit->kind = 1;
      hit->center = NSMakePoint(NSMidX(control.frame),
                                NSHeight(surface.host.bounds) -
                                    NSMidY(control.frame));
      return YES;
    }
  }
  for (ScreenwideOscMaterialSurfaceView *control in
       surface.rulerProbeLabelSurfaces) {
    ScreenwideRulerLabelRenderState *state =
        objc_getAssociatedObject(control, ScreenwideRulerLabelRenderStateKey);
    if (state.labelKind == 2 && state.probe.id != 0 && !control.hidden &&
        NSPointInRect(appKitPoint, control.frame)) {
      hit->id = state.probe.id;
      hit->kind = 2;
      hit->center = NSMakePoint(NSMidX(control.frame),
                                NSHeight(surface.host.bounds) -
                                    NSMidY(control.frame));
      return YES;
    }
  }
  for (ScreenwideOscMaterialSurfaceView *control in
       surface.rulerGuideGapLabelSurfaces) {
    ScreenwideRulerLabelRenderState *state =
        objc_getAssociatedObject(control, ScreenwideRulerLabelRenderStateKey);
    if (state.labelKind == 3 && state.probe.id != 0 && !control.hidden &&
        NSPointInRect(appKitPoint, control.frame)) {
      hit->id = state.probe.id;
      hit->kind = 3;
      hit->center = NSMakePoint(NSMidX(control.frame),
                                NSHeight(surface.host.bounds) -
                                    NSMidY(control.frame));
      return YES;
    }
  }
  for (ScreenwideOscMaterialSurfaceView *control in
       surface.rulerRadiusLabelSurfaces) {
    ScreenwideRulerLabelRenderState *state =
        objc_getAssociatedObject(control, ScreenwideRulerLabelRenderStateKey);
    if (state.labelKind == 4 && state.radius.id != 0 && !control.hidden &&
        NSPointInRect(appKitPoint, control.frame)) {
      hit->id = state.radius.id;
      hit->kind = 4;
      hit->center = NSMakePoint(NSMidX(control.frame),
                                NSHeight(surface.host.bounds) -
                                    NSMidY(control.frame));
      return YES;
    }
  }
  return NO;
}

static uint32_t light_mode(ScreenwideRegionOSC *surface) {
  NSString *appearance = [surface.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  return [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
}

static ScreenwideOscControlMetrics metrics(void) {
  return screenwide_osc_control_metrics(0, 0);
}

static BOOL point_in_surface(ScreenwideRegionOSC *surface) {
  return surface.rulerPoint.x >= 0.0 && surface.rulerPoint.y >= 0.0 &&
         surface.rulerPoint.x < NSWidth(surface.host.bounds) &&
         surface.rulerPoint.y < NSHeight(surface.host.bounds);
}

static NSPoint latest_pointer_point(ScreenwideRegionOSC *surface) {
  NSWindow *window = surface.host.window;
  if (!window)
    return surface.rulerPoint;
  NSPoint screen = NSEvent.mouseLocation;
  if (!NSPointInRect(screen, window.frame))
    return surface.rulerPoint;
  NSPoint windowPoint = [window convertPointFromScreen:screen];
  NSPoint point = [surface.host convertPoint:windowPoint fromView:nil];
  if (!surface.host.isFlipped)
    point.y = NSHeight(surface.host.bounds) - point.y;
  return point;
}

static NSString *hex_text(ScreenwideRegionOSC *surface) {
  return [NSString stringWithFormat:@"#%02X%02X%02X",
                                    (surface.rulerColor >> 24) & 0xFF,
                                    (surface.rulerColor >> 16) & 0xFF,
                                    (surface.rulerColor >> 8) & 0xFF];
}

static NSUInteger glyph_index(unichar glyph) {
  if (glyph == '#') return 0;
  if (glyph >= '0' && glyph <= '9') return 1 + glyph - '0';
  if (glyph >= 'A' && glyph <= 'F') return 11 + glyph - 'A';
  if (glyph == 0x00D7) return 17;
  if (glyph == ' ') return 18;
  if (glyph == 'p') return 19;
  if (glyph == 'x') return 20;
  if (glyph == 0x2248) return 21;
  return 0;
}

static NSRect glyph_texture_rect(ScreenwideOscTextTexture *atlas,
                                 NSUInteger index) {
  CGFloat cellWidth = 1.0 / kRulerAtlasCells;
  return NSMakeRect(index * cellWidth + atlas.atlasGlyphUOffset, 0.0,
                    atlas.atlasGlyphUWidth, 1.0);
}

static void update_label(ScreenwideRegionOSC *surface, CGFloat scale) {
  uint32_t light = light_mode(surface);
  if (surface.rulerLabel && surface.rulerLabelScale == scale &&
      surface.rulerLabelLightMode == light)
    return;
  ScreenwideOscControlMetrics value = metrics();
  surface.rulerLabel = screenwide_osc_mono_hex_atlas(
      surface.device, scale, light, value.font_size, value.line_height);
  surface.rulerLabelScale = scale;
  surface.rulerLabelLightMode = light;
}

static CGFloat animation_amount(ScreenwideRegionOSC *surface) {
  CFTimeInterval elapsed = CACurrentMediaTime() - surface.rulerAnimationStarted;
  CGFloat progress = MIN(MAX(elapsed / kRulerAnimationDuration, 0.0), 1.0);
  CGFloat eased = 1.0 - pow(1.0 - progress, 3.0);
  CGFloat target = surface.rulerAnimationTarget ? 1.0 : 0.0;
  return surface.rulerAnimationFrom +
         (target - surface.rulerAnimationFrom) * eased;
}

static CGFloat tolerance_animation_amount(ScreenwideRegionOSC *surface) {
  CFTimeInterval elapsed =
      CACurrentMediaTime() - surface.rulerToleranceAnimationStarted;
  CGFloat progress = MIN(MAX(elapsed / kRulerAnimationDuration, 0.0), 1.0);
  CGFloat eased = 1.0 - pow(1.0 - progress, 3.0);
  CGFloat target = surface.rulerToleranceAnimationTarget ? 1.0 : 0.0;
  return surface.rulerToleranceAnimationFrom +
      (target - surface.rulerToleranceAnimationFrom) * eased;
}

static CGFloat ruler_hover_progress(ScreenwideRegionOSC *surface) {
  if (surface.rulerHoveredArtifactKey == 0)
    return 1.0;
  CFTimeInterval elapsed =
      CACurrentMediaTime() - surface.rulerHoverPulseStarted;
  return MIN(MAX(elapsed / kRulerHoverPulseDuration, 0.0), 1.0);
}

static CGFloat ruler_hover_ease(CGFloat progress) {
  return 1.0 - pow(1.0 - progress, 3.0);
}

static CGFloat ruler_hover_width(ScreenwideRegionOSC *surface) {
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  CGFloat progress = ruler_hover_progress(surface);
  if (progress < 0.6) {
    CGFloat eased = ruler_hover_ease(progress / 0.6);
    return spacing.control + (spacing.section - spacing.control) * eased;
  }
  CGFloat eased = ruler_hover_ease((progress - 0.6) / 0.4);
  return spacing.section +
      (spacing.control_inset - spacing.section) * eased;
}

static CGFloat ruler_hover_alpha(ScreenwideRegionOSC *surface) {
  CGFloat progress = ruler_hover_progress(surface);
  if (progress < 0.6) {
    CGFloat eased = ruler_hover_ease(progress / 0.6);
    return 0.50 + (0.16 - 0.50) * eased;
  }
  CGFloat eased = ruler_hover_ease((progress - 0.6) / 0.4);
  return 0.16 + (0.24 - 0.16) * eased;
}

static void render(ScreenwideRegionOSC *surface);
static NSString *tolerance_text(uint8_t mode);
static void render_measurement_labels(ScreenwideRegionOSC *surface);
static void render_probe_label(ScreenwideRegionOSC *surface,
                               ScreenwideOscMaterialSurfaceView *control,
                               NativeRulerProbe probe, uint8_t labelKind);
static void render_probe_labels(ScreenwideRegionOSC *surface);
static void render_guide_gap_labels(ScreenwideRegionOSC *surface);
static void render_radius_labels(ScreenwideRegionOSC *surface);

static NSData *measurement_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_measurements(root.rustContext, NULL, 0);
  NSMutableData *data = [NSMutableData
      dataWithLength:count * sizeof(NativeRulerMeasurement)];
  if (count > 0) {
    size_t written = native_osc_ruler_measurements(
        root.rustContext, data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerMeasurement);
  }
  return data;
}

static NSUInteger measurement_count(ScreenwideRegionOSC *surface) {
  return surface.rulerMeasurements.length / sizeof(NativeRulerMeasurement);
}

static const NativeRulerMeasurement *measurements(
    ScreenwideRegionOSC *surface) {
  return (const NativeRulerMeasurement *)surface.rulerMeasurements.bytes;
}

static NSData *labelled_measurement_data(NSData *data) {
  NSMutableData *labelled = [NSMutableData dataWithData:data];
  NativeRulerMeasurement *items = labelled.mutableBytes;
  NSUInteger count = labelled.length / sizeof(NativeRulerMeasurement);
  for (NSUInteger index = 0; index < count; index++)
    items[index].flags &= 11;
  return labelled;
}

static NSData *probe_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_probes(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerProbe)];
  if (count > 0) {
    size_t written = native_osc_ruler_probes(root.rustContext,
                                             data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerProbe);
  }
  return data;
}

static NSUInteger probe_count(ScreenwideRegionOSC *surface) {
  return surface.rulerProbes.length / sizeof(NativeRulerProbe);
}

static const NativeRulerProbe *probes(ScreenwideRegionOSC *surface) {
  return (const NativeRulerProbe *)surface.rulerProbes.bytes;
}

static NSData *guide_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_guides(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerGuide)];
  if (count > 0) {
    size_t written = native_osc_ruler_guides(root.rustContext,
                                             data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerGuide);
  }
  return data;
}

static NSUInteger guide_count(ScreenwideRegionOSC *surface) {
  return surface.rulerGuides.length / sizeof(NativeRulerGuide);
}

static const NativeRulerGuide *guides(ScreenwideRegionOSC *surface) {
  return (const NativeRulerGuide *)surface.rulerGuides.bytes;
}

static NSData *guide_gap_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_guide_gaps(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerGuideGap)];
  if (count > 0) {
    size_t written = native_osc_ruler_guide_gaps(root.rustContext,
                                                 data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerGuideGap);
  }
  return data;
}

static NSUInteger guide_gap_count(ScreenwideRegionOSC *surface) {
  return surface.rulerGuideGaps.length / sizeof(NativeRulerGuideGap);
}

static const NativeRulerGuideGap *guide_gaps(ScreenwideRegionOSC *surface) {
  return (const NativeRulerGuideGap *)surface.rulerGuideGaps.bytes;
}

static NSData *radius_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_radii(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerRadius)];
  if (count > 0) {
    size_t written = native_osc_ruler_radii(root.rustContext,
                                            data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerRadius);
  }
  return data;
}

static NSUInteger radius_count(ScreenwideRegionOSC *surface) {
  return surface.rulerRadii.length / sizeof(NativeRulerRadius);
}

static const NativeRulerRadius *radii(ScreenwideRegionOSC *surface) {
  return (const NativeRulerRadius *)surface.rulerRadii.bytes;
}

static NSData *labelled_radius_data(NSData *data) {
  NSMutableData *labelled = [NSMutableData dataWithData:data];
  NativeRulerRadius *items = labelled.mutableBytes;
  NSUInteger count = labelled.length / sizeof(NativeRulerRadius);
  for (NSUInteger index = 0; index < count; index++)
    items[index].flags &= 11;
  return labelled;
}

static NSData *centerline_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_centerlines(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerCenterline)];
  if (count > 0) {
    size_t written = native_osc_ruler_centerlines(root.rustContext,
                                                  data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerCenterline);
  }
  return data;
}

static NSUInteger centerline_count(ScreenwideRegionOSC *surface) {
  return surface.rulerCenterlines.length / sizeof(NativeRulerCenterline);
}

static const NativeRulerCenterline *centerlines(
    ScreenwideRegionOSC *surface) {
  return (const NativeRulerCenterline *)surface.rulerCenterlines.bytes;
}

static NSData *inner_object_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_inner_objects(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerInnerObject)];
  if (count > 0) {
    size_t written = native_osc_ruler_inner_objects(root.rustContext,
                                                    data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerInnerObject);
  }
  return data;
}

static NSUInteger inner_object_count(ScreenwideRegionOSC *surface) {
  return surface.rulerInnerObjects.length / sizeof(NativeRulerInnerObject);
}

static const NativeRulerInnerObject *inner_objects(
    ScreenwideRegionOSC *surface) {
  return (const NativeRulerInnerObject *)surface.rulerInnerObjects.bytes;
}

static NSData *labelled_guide_gap_data(NSData *data) {
  NSMutableData *labelled = [NSMutableData data];
  const NativeRulerGuideGap *items = data.bytes;
  NSUInteger count = data.length / sizeof(NativeRulerGuideGap);
  for (NSUInteger index = 0; index < count; index++) {
    NativeRulerGuideGap value = items[index];
    value.flags &= 2;
    [labelled appendBytes:&value length:sizeof(value)];
  }
  return labelled;
}

static NSData *labelled_probe_data(NSData *data) {
  NSMutableData *labelled = [NSMutableData data];
  const NativeRulerProbe *items = data.bytes;
  NSUInteger count = data.length / sizeof(NativeRulerProbe);
  for (NSUInteger index = 0; index < count; index++) {
    BOOL draft = (items[index].flags & 1) != 0;
    BOOL live = (items[index].flags & 4) != 0;
    if (live || (items[index].id == 0 && !draft))
      continue;
    NativeRulerProbe value = items[index];
    value.flags = (draft ? 1 : 0) | (items[index].flags & 8);
    [labelled appendBytes:&value length:sizeof(value)];
  }
  return labelled;
}

static NSUInteger decimal_digit_count(CGFloat value);

static NSString *probe_dimensions_text(ScreenwideRegionOSC *surface) {
  const NativeRulerProbe *items = probes(surface);
  NSUInteger count = probe_count(surface);
  const NativeRulerProbe *horizontal = NULL;
  const NativeRulerProbe *vertical = NULL;
  for (NSUInteger index = 0; index < count; index++) {
    if ((items[index].flags & 4) == 0 ||
        items[index].display_id != surface.displayID)
      continue;
    if (items[index].axis == 1)
      horizontal = &items[index];
    else if (items[index].axis == 2)
      vertical = &items[index];
  }
  if (!horizontal || !vertical)
    return nil;
  NSInteger width =
      MAX((NSInteger)llround(fabs(horizontal->end - horizontal->start)), 0);
  NSInteger height =
      MAX((NSInteger)llround(fabs(vertical->end - vertical->start)), 0);
  int widthDigits = (int)decimal_digit_count(surface.desktopSize.width);
  int heightDigits = (int)decimal_digit_count(surface.desktopSize.height);
  return [NSString stringWithFormat:@"%*ld × %*ld px", widthDigits,
                                    (long)width, heightDigits,
                                    (long)height];
}

static NSUInteger decimal_digit_count(CGFloat value) {
  uint64_t magnitude = (uint64_t)MAX(ceil(fabs(value)), 0.0);
  NSUInteger digits = 1;
  while (magnitude >= 10) {
    magnitude /= 10;
    digits += 1;
  }
  return digits;
}

static NSUInteger reserved_dimensions_length(ScreenwideRegionOSC *surface) {
  // Every desktop peer receives the same union size, so the loupe keeps one
  // width while crossing monitors. Six characters cover " × " and " px".
  return decimal_digit_count(surface.desktopSize.width) +
         decimal_digit_count(surface.desktopSize.height) + 6;
}

static NSData *viewport_data(ScreenwideRegionOSC *root) {
  size_t count = native_osc_ruler_viewports(root.rustContext, NULL, 0);
  NSMutableData *data =
      [NSMutableData dataWithLength:count * sizeof(NativeRulerViewport)];
  if (count > 0) {
    size_t written = native_osc_ruler_viewports(
        root.rustContext, data.mutableBytes, count);
    if (written < count)
      data.length = written * sizeof(NativeRulerViewport);
  }
  return data;
}

static NativeRulerViewport viewport_for_surface(
    ScreenwideRegionOSC *surface, NSData *data) {
  const NativeRulerViewport *items = data.bytes;
  NSUInteger count = data.length / sizeof(NativeRulerViewport);
  for (NSUInteger index = 0; index < count; index++)
    if (items[index].display_id == surface.displayID)
      return items[index];
  NativeRulerViewport identity = {surface.displayID, 0, 1.0, 0.0, 0.0};
  return identity;
}

static NSRect project_measurement(ScreenwideRegionOSC *surface,
                                  NativeRulerMeasurement measurement) {
  CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
  return NSMakeRect(
      (measurement.x - surface.desktopOffset.x -
       surface.rulerViewportOrigin.x) * zoom,
      (measurement.y - surface.desktopOffset.y -
       surface.rulerViewportOrigin.y) * zoom,
      measurement.width * zoom, measurement.height * zoom);
}

static NSRect project_world_rect(ScreenwideRegionOSC *surface, CGFloat x,
                                 CGFloat y, CGFloat width,
                                 CGFloat height) {
  CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
  return NSMakeRect(
      (x - surface.desktopOffset.x - surface.rulerViewportOrigin.x) * zoom,
      (y - surface.desktopOffset.y - surface.rulerViewportOrigin.y) * zoom,
      width * zoom, height * zoom);
}

static void add_center_object_outline(
    ScreenwideRegionOscVertex *vertices, NSUInteger *count, NSSize size,
    NSRect frame, CGFloat scale) {
  CGFloat minX = screenwide_region_osc_snap(NSMinX(frame), scale);
  CGFloat maxX = screenwide_region_osc_snap(NSMaxX(frame), scale);
  CGFloat minY = screenwide_region_osc_snap(NSMinY(frame), scale);
  CGFloat maxY = screenwide_region_osc_snap(NSMaxY(frame), scale);
  CGFloat half = 0.5 / scale;
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(minX - half, minY - half, maxX - minX + half * 2.0,
                 half * 2.0),
      44);
  screenwide_region_osc_add_quad(
      vertices, count, size,
      NSMakeRect(minX - half, maxY - half, maxX - minX + half * 2.0,
                 half * 2.0),
      44);
  CGFloat verticalHeight = MAX(maxY - minY - half * 2.0, 0.0);
  if (verticalHeight > 0.0) {
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(minX - half, minY + half, half * 2.0,
                   verticalHeight),
        44);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(maxX - half, minY + half, half * 2.0,
                   verticalHeight),
        44);
  }
}

static void project_probe(ScreenwideRegionOSC *surface,
                          NativeRulerProbe probe, CGFloat *start,
                          CGFloat *end, CGFloat *position) {
  CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
  if (probe.axis == 1) {
    *start = (probe.start - surface.desktopOffset.x -
              surface.rulerViewportOrigin.x) * zoom;
    *end = (probe.end - surface.desktopOffset.x -
            surface.rulerViewportOrigin.x) * zoom;
    *position = (probe.position - surface.desktopOffset.y -
                 surface.rulerViewportOrigin.y) * zoom;
  } else {
    *start = (probe.start - surface.desktopOffset.y -
              surface.rulerViewportOrigin.y) * zoom;
    *end = (probe.end - surface.desktopOffset.y -
            surface.rulerViewportOrigin.y) * zoom;
    *position = (probe.position - surface.desktopOffset.x -
                 surface.rulerViewportOrigin.x) * zoom;
  }
}

static void schedule_frame(ScreenwideRegionOSC *surface,
                           uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.rulerAnimationRevision != revision) return;
                   render(surface);
                   if (CACurrentMediaTime() - surface.rulerAnimationStarted <
                       kRulerAnimationDuration)
                     schedule_frame(surface, revision);
                 });
}

static void set_copied(ScreenwideRegionOSC *surface, BOOL copied) {
  if (surface.rulerAnimationTarget == copied) return;
  surface.rulerAnimationFrom = animation_amount(surface);
  surface.rulerAnimationStarted = CACurrentMediaTime();
  surface.rulerAnimationTarget = copied;
  uint64_t revision = ++surface.rulerAnimationRevision;
  schedule_frame(surface, revision);
}

static void schedule_tolerance_frame(ScreenwideRegionOSC *surface,
                                     uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (surface.rulerToleranceAnimationRevision != revision)
                     return;
                   render(surface);
                   if (CACurrentMediaTime() -
                           surface.rulerToleranceAnimationStarted <
                       kRulerAnimationDuration)
                     schedule_tolerance_frame(surface, revision);
                 });
}

static void set_tolerance_visible(ScreenwideRegionOSC *surface, BOOL visible,
                                  BOOL restart) {
  if (!restart && surface.rulerToleranceAnimationTarget == visible)
    return;
  surface.rulerToleranceAnimationFrom =
      restart ? 0.0 : tolerance_animation_amount(surface);
  surface.rulerToleranceAnimationStarted = CACurrentMediaTime();
  surface.rulerToleranceAnimationTarget = visible;
  uint64_t revision = ++surface.rulerToleranceAnimationRevision;
  schedule_tolerance_frame(surface, revision);
}

static void layout(ScreenwideRegionOSC *surface, CGFloat width,
                   CGFloat height) {
  NSSize size = surface.host.bounds.size;
  NSPoint point = latest_pointer_point(surface);
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  CGFloat left = point.x + spacing.control_inset;
  CGFloat top = point.y + spacing.control_inset;
  if (left + width > size.width - spacing.control_inset)
    left = point.x - width - spacing.control_inset;
  if (top + height > size.height - spacing.control_inset)
    top = point.y - height - spacing.control_inset;
  left = MAX(spacing.control_inset,
             MIN(left, size.width - width - spacing.control_inset));
  top = MAX(spacing.control_inset,
            MIN(top, size.height - height - spacing.control_inset));

  ScreenwideOscControlSpec spec = {
      0.0, 0.0, width, height, 0, 0, 0, 0, 0};
  screenwide_osc_control_group_layout(surface.rulerControls, &spec, 1);
  ScreenwideOscControlMetrics value = metrics();
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  surface.rulerSurface.frame =
      NSMakeRect(left, size.height - top - height, width, height);
  surface.rulerSurface.layer.cornerRadius = value.radius;
  surface.rulerSurface.contentLayer.cornerRadius = value.radius;
  surface.rulerSurface.contentView.frame = surface.rulerSurface.bounds;
  [CATransaction commit];
}

static void render(ScreenwideRegionOSC *surface) {
  ScreenwideOscMaterialSurfaceView *control = surface.rulerSurface;
  if (!surface.rulerVisible || !surface.rulerTransientChromeVisible ||
      surface.rulerInteractionActive ||
      surface.rulerHoveredArtifactKey != 0 || !point_in_surface(surface)) {
    control.hidden = YES;
    return;
  }
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  update_label(surface, scale);
  if (!surface.rulerLabel || !surface.rulerControls) return;
  ScreenwideOscControlMetrics value = metrics();
  uint32_t light = light_mode(surface);
  CGFloat tolerance = tolerance_animation_amount(surface);
  if ((surface.rulerToleranceVisible || tolerance > 0.001) &&
      (!surface.rulerToleranceLabel ||
       surface.rulerToleranceLabelScale != scale ||
       surface.rulerToleranceLabelLightMode != light)) {
    surface.rulerToleranceLabel = screenwide_osc_mono_text_texture(
        surface.device, tolerance_text(surface.rulerToleranceMode), scale,
        light, value.font_size, value.line_height);
    surface.rulerToleranceLabelScale = scale;
    surface.rulerToleranceLabelLightMode = light;
  }
  CGFloat cellWidth = surface.rulerLabel.atlasGlyphWidth;
  CGFloat labelWidth = cellWidth * 7.0;
  NSString *dimensions = probe_dimensions_text(surface);
  NSString *colour = hex_text(surface);
  control.accessibilityElement = YES;
  control.accessibilityRole = NSAccessibilityStaticTextRole;
  control.accessibilityLabel = @"Ruler readout";
  control.accessibilityValue = dimensions
      ? [NSString stringWithFormat:@"%@, %@", dimensions, colour]
      : colour;
  CGFloat colourWidth = value.icon_size + value.gap + labelWidth;
  NSUInteger dimensionsLength =
      MAX(dimensions.length, reserved_dimensions_length(surface));
  CGFloat dimensionsWidth = cellWidth * dimensionsLength;
  CGFloat width = value.padding_x * 2.0 + MAX(colourWidth, dimensionsWidth);
  CGFloat height = dimensions ? value.height + value.line_height : value.height;
  layout(surface, width, height);
  control.hidden = NO;
  ScreenwideOscControlVisual visual = {0};
  if (screenwide_osc_control_group_visuals(surface.rulerControls,
                                            light_mode(surface) == 0,
                                            &visual, 1) != 1)
    return;
  // Moving the NSVisualEffectView remains synchronous with the latest pointer
  // sample. Coalesce only its small Metal content so nextDrawable never blocks
  // AppKit when pointer events arrive faster than the display refreshes.
  if (surface.rulerDrawInFlight) {
    surface.rulerDrawPending = YES;
    return;
  }
  surface.rulerDrawInFlight = YES;
  surface.rulerDrawPending = NO;
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light);
  memcpy(state.action_fills, visual.fill, sizeof(visual.fill));
  memcpy(state.action_fills + 4, visual.foreground,
         sizeof(visual.foreground));
  state.ruler_sample[0] = ((surface.rulerColor >> 24) & 0xFF) / 255.0;
  state.ruler_sample[1] = ((surface.rulerColor >> 16) & 0xFF) / 255.0;
  state.ruler_sample[2] = ((surface.rulerColor >> 8) & 0xFF) / 255.0;
  state.ruler_sample[3] = (surface.rulerColor & 0xFF) / 255.0;
  CGFloat copied = animation_amount(surface);
  state.ruler_animation[0] = copied;
  state.ruler_animation[3] = tolerance;

  ScreenwideRegionOscVertex vertices[192];
  NSUInteger count = 0;
  NSSize size = NSMakeSize(width, height);
  screenwide_region_osc_add_quad(
      vertices, &count, size, NSMakeRect(0, 0, width, height), 12);
  CGFloat colourTop = dimensions ? value.line_height : 0.0;
  CGFloat iconTop = colourTop + (value.height - value.icon_size) * 0.5;
  NSRect swatch = NSMakeRect(value.padding_x, iconTop, value.icon_size,
                            value.icon_size);
  screenwide_region_osc_add_quad(vertices, &count, size, swatch, 29);

  NSString *text = colour;
  CGFloat textLeft = value.padding_x + value.icon_size + value.gap;
  CGFloat textTop = colourTop +
      (value.height - surface.rulerLabel.size.height) * 0.5;
  for (NSUInteger index = 0; index < text.length; index++) {
    screenwide_region_osc_add_texture_quad(
        vertices, &count, size,
        NSMakeRect(textLeft + cellWidth * index, textTop, cellWidth,
                   surface.rulerLabel.size.height),
        glyph_texture_rect(surface.rulerLabel,
                           glyph_index([text characterAtIndex:index])),
        11);
  }
  if (dimensions) {
    CGFloat dimensionsLeft = (width - cellWidth * dimensions.length) * 0.5;
    CGFloat dimensionsTop =
        (value.height - value.line_height) * 0.5 +
        (value.line_height - surface.rulerLabel.size.height) * 0.5;
    for (NSUInteger index = 0; index < dimensions.length; index++) {
      screenwide_region_osc_add_texture_quad(
          vertices, &count, size,
          NSMakeRect(dimensionsLeft + cellWidth * index, dimensionsTop,
                     cellWidth, surface.rulerLabel.size.height),
          glyph_texture_rect(surface.rulerLabel,
                             glyph_index([dimensions characterAtIndex:index])),
          11);
    }
  }

  // Match CheckOnClick: scale/fade in, then only fade away on expiry.
  CGFloat checkScale = surface.rulerAnimationTarget ? copied : 1.0;
  CGFloat centerX = NSMidX(swatch);
  CGFloat centerY = NSMidY(swatch);
  NSPoint a = NSMakePoint(centerX - 4.0 * checkScale,
                         centerY - 0.5 * checkScale);
  NSPoint b = NSMakePoint(centerX - 1.0 * checkScale,
                         centerY + 2.5 * checkScale);
  NSPoint c = NSMakePoint(centerX + 5.0 * checkScale,
                         centerY - 3.5 * checkScale);
  screenwide_region_osc_add_line(vertices, &count, size, a, b,
                                 2.0 * MAX(checkScale, 0.001), 30);
  screenwide_region_osc_add_line(vertices, &count, size, b, c,
                                 2.0 * MAX(checkScale, 0.001), 30);

  ScreenwideOscTextTexture *toleranceLabel = surface.rulerToleranceLabel;
  if (toleranceLabel && tolerance > 0.001) {
    CGFloat labelScale =
        surface.rulerToleranceAnimationTarget ? tolerance : 1.0;
    NSSize labelSize = NSMakeSize(toleranceLabel.size.width * labelScale,
                                  toleranceLabel.size.height * labelScale);
    screenwide_region_osc_add_texture_quad(
        vertices, &count, size,
        NSMakeRect((width - labelSize.width) * 0.5,
                   (height - labelSize.height) * 0.5,
                   labelSize.width, labelSize.height),
        NSMakeRect(0.0, 0.0, 1.0, 1.0), 37);
  }

  control.contentLayer.contentsScale = scale;
  control.contentLayer.drawableSize =
      CGSizeMake(MAX(width * scale, 2.0), MAX(height * scale, 2.0));
  id<CAMetalDrawable> drawable = [control.contentLayer nextDrawable];
  if (!drawable) {
    surface.rulerDrawInFlight = NO;
    return;
  }
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
  screenwide_region_osc_encode(encoder, surface.pipeline, buffer, count,
                               state, surface.rulerLabel.texture,
                               toleranceLabel ? toleranceLabel.texture
                                              : surface.placeholder);
  [encoder endEncoding];
  [command presentDrawable:drawable];
  [command addCompletedHandler:^(__unused id<MTLCommandBuffer> completed) {
    dispatch_async(dispatch_get_main_queue(), ^{
      surface.rulerDrawInFlight = NO;
      BOOL pending = surface.rulerDrawPending;
      surface.rulerDrawPending = NO;
      if (pending) render(surface);
    });
  }];
  [command commit];
}

static NSString *tolerance_text(uint8_t mode) {
  if (mode == 1) return @"Clear edges";
  if (mode == 3) return @"Subtle edges";
  return @"Balanced";
}

static void schedule_settle_frame(ScreenwideRegionOSC *root) {
  if (root.rulerSettleScheduled || !root.input || !root.rustContext) return;
  root.rulerSettleScheduled = YES;
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   root.rulerSettleScheduled = NO;
                   if (!root.input || !root.rustContext || !root.visible)
                     return;
                   NativeOscResult frame = {0};
                   root.input(root.rustContext, 15, 0, 0, 0, &frame);
                   if (frame.status != 255)
                     screenwide_region_osc_apply_ruler_result(root, frame);
                 });
}

static uint64_t hovered_artifact_key(NSData *measurementsData,
                                     NSData *probesData,
                                     NSData *guidesData,
                                     NSData *gapsData,
                                     NSData *radiiData) {
  const NativeRulerMeasurement *measurementItems = measurementsData.bytes;
  NSUInteger measurementCount =
      measurementsData.length / sizeof(NativeRulerMeasurement);
  for (NSUInteger index = 0; index < measurementCount; index++)
    if ((measurementItems[index].flags & 4) != 0)
      return (measurementItems[index].id << 3) | 1;

  const NativeRulerProbe *probeItems = probesData.bytes;
  NSUInteger probeCount = probesData.length / sizeof(NativeRulerProbe);
  for (NSUInteger index = 0; index < probeCount; index++)
    if ((probeItems[index].flags & 2) != 0)
      return (probeItems[index].id << 3) | 2;
  const NativeRulerGuide *guideItems = guidesData.bytes;
  NSUInteger guideCount = guidesData.length / sizeof(NativeRulerGuide);
  for (NSUInteger index = 0; index < guideCount; index++)
    if ((guideItems[index].flags & 2) != 0)
      return (guideItems[index].id << 3) | 3;
  const NativeRulerGuideGap *gapItems = gapsData.bytes;
  NSUInteger gapCount = gapsData.length / sizeof(NativeRulerGuideGap);
  for (NSUInteger index = 0; index < gapCount; index++)
    if ((gapItems[index].flags & 1) != 0)
      return (gapItems[index].id << 3) | 4;
  const NativeRulerRadius *radiusItems = radiiData.bytes;
  NSUInteger radiusCount = radiiData.length / sizeof(NativeRulerRadius);
  for (NSUInteger index = 0; index < radiusCount; index++)
    if ((radiusItems[index].flags & 4) != 0)
      return (radiusItems[index].id << 3) | 5;
  return 0;
}

static void schedule_hover_pulse_frame(ScreenwideRegionOSC *root,
                                       uint64_t revision) {
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 16 * NSEC_PER_MSEC),
                 dispatch_get_main_queue(), ^{
                   if (root.rulerHoverPulseRevision != revision ||
                       root.rulerHoveredArtifactKey == 0 || !root.visible)
                     return;
                   for (ScreenwideRegionOSC *item in
                        screenwide_region_osc_surfaces(root)) {
                     item.rulerHoveredArtifactKey =
                         root.rulerHoveredArtifactKey;
                     item.rulerHoverPulseStarted =
                         root.rulerHoverPulseStarted;
                     screenwide_region_osc_draw(item);
                   }
                   if (CACurrentMediaTime() - root.rulerHoverPulseStarted <
                       kRulerHoverPulseDuration)
                     schedule_hover_pulse_frame(root, revision);
                 });
}

void screenwide_region_osc_apply_ruler_result(ScreenwideRegionOSC *surface,
                                               NativeOscResult result) {
  if ((result.ruler_flags & 1) == 0) return;
  ScreenwideRegionOSC *root = screenwide_region_osc_root(surface);
  BOOL previousCrosshair = root.rulerCrosshair;
  root.rulerVisible = YES;
  root.rulerCrosshair = (result.ruler_flags & 2) != 0;
  root.rulerCopied = (result.ruler_flags & 4) != 0;
  root.rulerInteractionActive = (result.ruler_flags & 64) != 0;
  root.rulerPoint = NSMakePoint(result.x, result.y);
  root.rulerColor = result.ruler_color;
  BOOL toleranceRequested = (result.ruler_flags & 8) != 0;
  uint8_t toleranceMode = (result.ruler_flags >> 4) & 3;
  BOOL toleranceStarted = toleranceRequested &&
      (!root.rulerToleranceVisible ||
       root.rulerToleranceMode != toleranceMode);
  BOOL toleranceStopped = !toleranceRequested &&
      root.rulerToleranceVisible;
  if (toleranceStarted || toleranceStopped)
    root.rulerToleranceRevision++;
  root.rulerToleranceVisible = toleranceRequested;
  if (toleranceRequested)
    root.rulerToleranceMode = toleranceMode;
  NSData *drawList = measurement_data(root);
  NSData *probeList = probe_data(root);
  NSData *guideList = guide_data(root);
  NSData *guideGapList = guide_gap_data(root);
  NSData *radiusList = radius_data(root);
  NSData *centerlineList = centerline_data(root);
  NSData *innerObjectList = inner_object_data(root);
  NSData *viewports = viewport_data(root);
  uint64_t hoverKey = hovered_artifact_key(drawList, probeList, guideList,
                                           guideGapList, radiusList);
  BOOL hoverChanged = root.rulerHoveredArtifactKey != hoverKey;
  if (hoverChanged) {
    root.rulerHoveredArtifactKey = hoverKey;
    root.rulerHoverPulseStarted = CACurrentMediaTime();
    root.rulerHoverPulseRevision++;
  }
  BOOL animationActive = NO;
  const NativeRulerMeasurement *items = drawList.bytes;
  NSUInteger itemCount = drawList.length / sizeof(NativeRulerMeasurement);
  for (NSUInteger index = 0; index < itemCount; index++)
    animationActive |= (items[index].flags & 2) != 0;
  uint64_t revision = ++root.rulerCopiedRevision;
  for (ScreenwideRegionOSC *item in screenwide_region_osc_surfaces(root)) {
    NativeRulerViewport viewport = viewport_for_surface(item, viewports);
    BOOL viewportChanged = item.rulerViewportZoom != viewport.zoom ||
        item.rulerViewportOrigin.x != viewport.origin_x ||
        item.rulerViewportOrigin.y != viewport.origin_y;
    item.rulerViewportZoom = viewport.zoom;
    item.rulerViewportOrigin =
        NSMakePoint(viewport.origin_x, viewport.origin_y);
    item.rulerVisible = YES;
    item.rulerCrosshair = root.rulerCrosshair;
    item.rulerCopied = root.rulerCopied;
    item.rulerInteractionActive = root.rulerInteractionActive;
    item.rulerPoint = NSMakePoint(result.x - item.desktopOffset.x,
                                 result.y - item.desktopOffset.y);
    item.rulerColor = result.ruler_color;
    if (toleranceStarted ||
        item.rulerToleranceMode != root.rulerToleranceMode)
      item.rulerToleranceLabel = nil;
    item.rulerToleranceMode = root.rulerToleranceMode;
    item.rulerToleranceVisible = root.rulerToleranceVisible;
    set_tolerance_visible(item, root.rulerToleranceVisible,
                          toleranceStarted);
    item.rulerHoveredArtifactKey = hoverKey;
    item.rulerHoverPulseStarted = root.rulerHoverPulseStarted;
    BOOL measurementChanged = viewportChanged ||
        ![item.rulerMeasurements isEqualToData:drawList];
    BOOL measurementLabelsChanged = viewportChanged ||
        ![labelled_measurement_data(item.rulerMeasurements)
            isEqualToData:labelled_measurement_data(drawList)];
    BOOL probeChanged = viewportChanged ||
        ![item.rulerProbes isEqualToData:probeList];
    BOOL guideChanged = viewportChanged ||
        ![item.rulerGuides isEqualToData:guideList];
    BOOL guideGapChanged = viewportChanged ||
        ![item.rulerGuideGaps isEqualToData:guideGapList];
    BOOL guideGapLabelsChanged = viewportChanged ||
        ![labelled_guide_gap_data(item.rulerGuideGaps)
            isEqualToData:labelled_guide_gap_data(guideGapList)];
    BOOL probeLabelsChanged = viewportChanged ||
        ![labelled_probe_data(item.rulerProbes)
            isEqualToData:labelled_probe_data(probeList)];
    BOOL radiusChanged = viewportChanged ||
        ![item.rulerRadii isEqualToData:radiusList];
    BOOL radiusLabelsChanged = viewportChanged ||
        ![labelled_radius_data(item.rulerRadii)
            isEqualToData:labelled_radius_data(radiusList)];
    BOOL centerAidChanged = viewportChanged ||
        ![item.rulerCenterlines isEqualToData:centerlineList] ||
        ![item.rulerInnerObjects isEqualToData:innerObjectList];
    item.rulerMeasurements = drawList;
    item.rulerProbes = probeList;
    item.rulerGuides = guideList;
    item.rulerGuideGaps = guideGapList;
    item.rulerRadii = radiusList;
    item.rulerCenterlines = centerlineList;
    item.rulerInnerObjects = innerObjectList;
    set_copied(item, root.rulerCopied);
    render(item);
    if (previousCrosshair || root.rulerCrosshair || measurementChanged ||
        probeChanged || guideChanged || guideGapChanged || radiusChanged ||
        centerAidChanged)
      screenwide_region_osc_draw(item);
    if (measurementLabelsChanged)
      render_measurement_labels(item);
    if (probeLabelsChanged)
      render_probe_labels(item);
    if (guideGapLabelsChanged)
      render_guide_gap_labels(item);
    if (radiusLabelsChanged)
      render_radius_labels(item);
  }
  if (animationActive)
    schedule_settle_frame(root);
  if (hoverChanged && hoverKey != 0)
    schedule_hover_pulse_frame(root, root.rulerHoverPulseRevision);
  if (toleranceStarted) {
    uint64_t toleranceRevision = root.rulerToleranceRevision;
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 900 * NSEC_PER_MSEC),
                   dispatch_get_main_queue(), ^{
                     if (root.rulerToleranceRevision != toleranceRevision)
                       return;
                     root.rulerToleranceVisible = NO;
                     for (ScreenwideRegionOSC *item in
                          screenwide_region_osc_surfaces(root)) {
                       item.rulerToleranceVisible = NO;
                       set_tolerance_visible(item, NO, NO);
                     }
                   });
  }
  if (root.rulerCopied) {
    dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 900 * NSEC_PER_MSEC),
                   dispatch_get_main_queue(), ^{
                     if (root.rulerCopiedRevision != revision) return;
                     root.rulerCopied = NO;
                     for (ScreenwideRegionOSC *item in
                          screenwide_region_osc_surfaces(root)) {
                       item.rulerCopied = NO;
                       set_copied(item, NO);
                       render(item);
                     }
                   });
  }
}

void screenwide_region_osc_ruler_apply_render_state(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscRenderState *state) {
  if (!state || !surface.rulerControls) return;
  ScreenwideOscControlVisual visual = {0};
  if (screenwide_osc_control_group_visuals(surface.rulerControls,
                                            light_mode(surface) == 0,
                                            &visual, 1) != 1)
    return;
  memcpy(state->action_fills, visual.fill, sizeof(visual.fill));
  memcpy(state->action_fills + 4, visual.foreground,
         sizeof(visual.foreground));
  state->ruler_animation[1] =
      surface.rulerHoveredArtifactKey != 0
          ? ruler_hover_alpha(surface)
          : 0.0;
  state->ruler_animation[2] =
      ruler_hover_width(surface) *
      (surface.host.window.backingScaleFactor ?: 1.0);
}

void screenwide_region_osc_ruler_attach(ScreenwideRegionOSC *surface) {
  surface.rulerTransientChromeVisible = YES;
  surface.rulerControls = screenwide_osc_control_group_create();
  surface.rulerViewportZoom = 1.0;
  surface.rulerViewportOrigin = NSZeroPoint;
  surface.rulerMeasurements = [NSData data];
  surface.rulerMeasurementLabelSurfaces = [NSMutableArray array];
  surface.rulerProbes = [NSData data];
  surface.rulerGuides = [NSData data];
  surface.rulerProbeLabelSurfaces = [NSMutableArray array];
  surface.rulerGuideGaps = [NSData data];
  surface.rulerGuideGapLabelSurfaces = [NSMutableArray array];
  surface.rulerRadii = [NSData data];
  surface.rulerRadiusLabelSurfaces = [NSMutableArray array];
  surface.rulerCenterlines = [NSData data];
  surface.rulerInnerObjects = [NSData data];
  ScreenwideOscControlMetrics value = metrics();
  ScreenwideOscControlSpec spec = {
      0.0, 0.0, 1.0, value.height, 0, 0, 0, 0, 0};
  screenwide_osc_control_group_layout(surface.rulerControls, &spec, 1);
  surface.rulerSurface = screenwide_osc_material_surface(surface.device);
  surface.rulerSurface.accessibilityElement = YES;
  surface.rulerSurface.accessibilityRole = NSAccessibilityStaticTextRole;
  surface.rulerSurface.accessibilityLabel = @"Ruler readout";
  [surface.host addSubview:surface.rulerSurface
                positioned:NSWindowAbove relativeTo:nil];
}

void screenwide_region_osc_ruler_set_transient_chrome(void *view_ptr,
                                                       int visible) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root)
    return;
  BOOL show = visible != 0;
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    surface.rulerTransientChromeVisible = show;
    if (show) {
      render(surface);
    } else {
      surface.rulerDrawPending = NO;
      surface.rulerSurface.hidden = YES;
    }
    screenwide_region_osc_draw(surface);
  }
  [CATransaction commit];
  if (!show)
    [CATransaction flush];
}

void screenwide_region_osc_ruler_teardown(ScreenwideRegionOSC *surface) {
  surface.rulerAnimationRevision += 1;
  surface.rulerToleranceAnimationRevision += 1;
  surface.rulerDrawPending = NO;
  [surface.rulerSurface removeFromSuperview];
  surface.rulerSurface = nil;
  surface.rulerToleranceLabel = nil;
  for (ScreenwideOscMaterialSurfaceView *label in
       surface.rulerMeasurementLabelSurfaces)
    [label removeFromSuperview];
  [surface.rulerMeasurementLabelSurfaces removeAllObjects];
  surface.rulerMeasurementLabelSurfaces = nil;
  surface.rulerMeasurements = nil;
  for (ScreenwideOscMaterialSurfaceView *label in
       surface.rulerProbeLabelSurfaces)
    [label removeFromSuperview];
  [surface.rulerProbeLabelSurfaces removeAllObjects];
  surface.rulerProbeLabelSurfaces = nil;
  surface.rulerProbes = nil;
  surface.rulerGuides = nil;
  for (ScreenwideOscMaterialSurfaceView *label in
       surface.rulerGuideGapLabelSurfaces)
    [label removeFromSuperview];
  [surface.rulerGuideGapLabelSurfaces removeAllObjects];
  surface.rulerGuideGapLabelSurfaces = nil;
  surface.rulerGuideGaps = nil;
  for (ScreenwideOscMaterialSurfaceView *label in
       surface.rulerRadiusLabelSurfaces)
    [label removeFromSuperview];
  [surface.rulerRadiusLabelSurfaces removeAllObjects];
  surface.rulerRadiusLabelSurfaces = nil;
  surface.rulerRadii = nil;
  surface.rulerCenterlines = nil;
  surface.rulerInnerObjects = nil;
  if (surface.rulerControls)
    screenwide_osc_control_group_destroy(surface.rulerControls);
  surface.rulerControls = NULL;
  surface.rulerLabel = nil;
}

void screenwide_region_osc_ruler_update_appearance(
    ScreenwideRegionOSC *surface) {
  surface.rulerLabel = nil;
  surface.rulerToleranceLabel = nil;
  render(surface);
  render_measurement_labels(surface);
  render_probe_labels(surface);
  render_guide_gap_labels(surface);
  render_radius_labels(surface);
}

NSUInteger screenwide_region_osc_ruler_vertex_capacity(
    ScreenwideRegionOSC *surface) {
  NSUInteger crosshair =
      surface.rulerVisible && surface.rulerCrosshair ? 12 : 0;
  return crosshair + measurement_count(surface) * 48 +
      probe_count(surface) * 24 + guide_count(surface) * 12 +
      guide_gap_count(surface) * 24 + radius_count(surface) * 12 +
      centerline_count(surface) * 12 + inner_object_count(surface) * 36;
}

static NSRect visible_world_rect(ScreenwideRegionOSC *surface) {
  CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
  return NSMakeRect(
      surface.desktopOffset.x + surface.rulerViewportOrigin.x,
      surface.desktopOffset.y + surface.rulerViewportOrigin.y,
      NSWidth(surface.host.bounds) / zoom,
      NSHeight(surface.host.bounds) / zoom);
}

static BOOL has_label_anchor(CGFloat x, CGFloat y) {
  return isfinite(x) && isfinite(y);
}

static ScreenwideRegionOSC *label_anchor_surface(
    ScreenwideRegionOSC *surface, NSPoint anchor) {
  for (ScreenwideRegionOSC *item in screenwide_region_osc_surfaces(surface))
    if (NSPointInRect(anchor, visible_world_rect(item)))
      return item;
  return nil;
}

static NSPoint project_label_anchor(ScreenwideRegionOSC *surface,
                                    NSPoint anchor) {
  CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
  return NSMakePoint(
      (anchor.x - surface.desktopOffset.x -
       surface.rulerViewportOrigin.x) * zoom,
      (anchor.y - surface.desktopOffset.y -
       surface.rulerViewportOrigin.y) * zoom);
}

static ScreenwideRegionOSC *measurement_label_surface(
    ScreenwideRegionOSC *surface, NativeRulerMeasurement measurement) {
  if (has_label_anchor(measurement.label_anchor_x,
                       measurement.label_anchor_y))
    return label_anchor_surface(
        surface,
        NSMakePoint(measurement.label_anchor_x,
                    measurement.label_anchor_y));
  NSRect globalMeasurement =
      NSMakeRect(measurement.x, measurement.y,
                 measurement.width, measurement.height);
  ScreenwideRegionOSC *best = nil;
  CGFloat bestArea = 0.0;
  for (ScreenwideRegionOSC *item in screenwide_region_osc_surfaces(surface)) {
    NSRect globalSurface = visible_world_rect(item);
    NSRect overlap = NSIntersectionRect(globalMeasurement, globalSurface);
    CGFloat area = NSWidth(overlap) * NSHeight(overlap);
    if (area > bestArea) {
      best = item;
      bestArea = area;
    }
  }
  return best;
}

static NSString *measurement_text(ScreenwideRegionOSC *surface,
                                  NSRect frame, BOOL reserveWidth) {
  NSInteger width = MAX((NSInteger)llround(NSWidth(frame)), 0);
  NSInteger height = MAX((NSInteger)llround(NSHeight(frame)), 0);
  if (!reserveWidth) {
    if (NSHeight(frame) < 8.0)
      return [NSString stringWithFormat:@"%ld px", (long)width];
    if (NSWidth(frame) < 8.0)
      return [NSString stringWithFormat:@"%ld px", (long)height];
    return [NSString stringWithFormat:@"%ld × %ld px", (long)width,
                                      (long)height];
  }
  int widthDigits = (int)decimal_digit_count(surface.desktopSize.width);
  int heightDigits = (int)decimal_digit_count(surface.desktopSize.height);
  if (NSHeight(frame) < 8.0)
    return [NSString stringWithFormat:@"%*ld px", widthDigits,
                                      (long)width];
  if (NSWidth(frame) < 8.0)
    return [NSString stringWithFormat:@"%*ld px", heightDigits,
                                      (long)height];
  return [NSString stringWithFormat:@"%*ld × %*ld px", widthDigits,
                                    (long)width, heightDigits,
                                    (long)height];
}

static void render_measurement_label(ScreenwideRegionOSC *surface,
                                     ScreenwideOscMaterialSurfaceView *control,
                                     NativeRulerMeasurement measurement) {
  ScreenwideRulerLabelRenderState *renderState =
      label_render_state(surface, control);
  renderState.labelKind = 1;
  renderState.measurement = measurement;
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  update_label(surface, scale);
  if (!surface.rulerLabel) return;
  NSRect global = NSMakeRect(measurement.x, measurement.y,
                             measurement.width, measurement.height);
  NSRect frame = project_measurement(surface, measurement);
  NSString *text = measurement_text(surface, global,
                                    (measurement.flags & 1) != 0);
  control.accessibilityElement = YES;
  control.accessibilityRole = NSAccessibilityStaticTextRole;
  control.accessibilityLabel = @"Measurement";
  control.accessibilityValue = [text stringByTrimmingCharactersInSet:
      NSCharacterSet.whitespaceCharacterSet];
  CGFloat cellWidth = surface.rulerLabel.atlasGlyphWidth;
  ScreenwideOscControlMetrics value = metrics();
  CGFloat width = value.padding_x * 2.0 + cellWidth * text.length;
  CGFloat height = value.height;
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  BOOL horizontal = NSHeight(global) < spacing.control_inset;
  BOOL vertical = NSWidth(global) < spacing.control_inset;
  CGFloat left = NSMidX(frame) - width * 0.5;
  CGFloat top = NSMidY(frame) - height * 0.5;
  BOOL anchored = has_label_anchor(measurement.label_anchor_x,
                                   measurement.label_anchor_y);
  if (anchored) {
    NSPoint anchor = project_label_anchor(
        surface, NSMakePoint(measurement.label_anchor_x,
                             measurement.label_anchor_y));
    left = anchor.x - width * 0.5;
    top = anchor.y - height * 0.5;
  } else if (!horizontal && !vertical &&
      (NSWidth(frame) < width + spacing.control_inset * 2.0 ||
       NSHeight(frame) < height + spacing.control_inset * 2.0))
    top = NSMinY(frame) - height - spacing.control;
  else if (horizontal)
    top = NSMaxY(frame) + spacing.control;
  else if (vertical)
    left = NSMaxX(frame) + spacing.control;
  NSSize host = surface.host.bounds.size;
  left = MAX(spacing.control_inset,
             MIN(left, host.width - width - spacing.control_inset));
  top = MAX(spacing.control_inset,
            MIN(top, host.height - height - spacing.control_inset));

  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  control.frame = NSMakeRect(left, host.height - top - height, width, height);
  control.layer.cornerRadius = value.radius;
  control.contentLayer.cornerRadius = value.radius;
  control.contentView.frame = control.bounds;
  [CATransaction commit];
  control.hidden = NO;

  // The material frame follows the latest pointer sample immediately. Its
  // small Metal content waits until the previous drawable is actually on
  // screen, then redraws only the newest coalesced measurement.
  if (renderState.inFlight) {
    renderState.pending = YES;
    return;
  }
  renderState.inFlight = YES;
  renderState.pending = NO;

  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light_mode(surface));
  screenwide_region_osc_ruler_apply_render_state(surface, &state);
  ScreenwideRegionOscVertex vertices[192];
  NSUInteger count = 0;
  NSSize size = NSMakeSize(width, height);
  screenwide_region_osc_add_quad(
      vertices, &count, size, NSMakeRect(0.0, 0.0, width, height), 12);
  CGFloat textLeft = value.padding_x;
  CGFloat textTop = (height - surface.rulerLabel.size.height) * 0.5;
  for (NSUInteger index = 0; index < text.length; index++) {
    screenwide_region_osc_add_texture_quad(
        vertices, &count, size,
        NSMakeRect(textLeft + cellWidth * index, textTop, cellWidth,
                   surface.rulerLabel.size.height),
        glyph_texture_rect(surface.rulerLabel,
                           glyph_index([text characterAtIndex:index])),
        11);
  }

  control.contentLayer.contentsScale = scale;
  control.contentLayer.drawableSize =
      CGSizeMake(MAX(width * scale, 2.0), MAX(height * scale, 2.0));
  id<CAMetalDrawable> drawable = [control.contentLayer nextDrawable];
  if (!drawable) {
    renderState.inFlight = NO;
    return;
  }
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
  screenwide_region_osc_encode(encoder, surface.pipeline, buffer, count,
                               state, surface.rulerLabel.texture,
                               surface.placeholder);
  [encoder endEncoding];
  __weak ScreenwideRulerLabelRenderState *weakRenderState = renderState;
  [drawable addPresentedHandler:^(__unused id<MTLDrawable> presented) {
    dispatch_async(dispatch_get_main_queue(), ^{
      ScreenwideRulerLabelRenderState *latest = weakRenderState;
      if (!latest)
        return;
      latest.inFlight = NO;
      if (!latest.pending)
        return;
      latest.pending = NO;
      ScreenwideRegionOSC *latestSurface = latest.surface;
      ScreenwideOscMaterialSurfaceView *latestControl = latest.control;
      if (latestSurface && latestControl) {
        if (latest.labelKind != 1)
          render_probe_label(latestSurface, latestControl, latest.probe,
                             latest.labelKind);
        else
          render_measurement_label(latestSurface, latestControl,
                                   latest.measurement);
      }
    });
  }];
  [command presentDrawable:drawable];
  [command commit];
}

static void render_measurement_labels(ScreenwideRegionOSC *surface) {
  NSMutableArray<NSValue *> *owned = [NSMutableArray array];
  const NativeRulerMeasurement *items = measurements(surface);
  NSUInteger count = measurement_count(surface);
  for (NSUInteger index = 0; index < count; index++) {
    if ((items[index].flags & 8) == 0 &&
        measurement_label_surface(surface, items[index]) == surface) {
      [owned addObject:[NSValue valueWithBytes:&items[index]
                                     objCType:@encode(NativeRulerMeasurement)]];
    }
  }
  while (surface.rulerMeasurementLabelSurfaces.count < owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        screenwide_osc_material_surface(surface.device);
    control.accessibilityElement = YES;
    control.accessibilityRole = NSAccessibilityStaticTextRole;
    control.accessibilityLabel = @"Measurement";
    [surface.host addSubview:control
                  positioned:NSWindowBelow relativeTo:surface.rulerSurface];
    [surface.rulerMeasurementLabelSurfaces addObject:control];
  }
  while (surface.rulerMeasurementLabelSurfaces.count > owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.rulerMeasurementLabelSurfaces.lastObject;
    [control removeFromSuperview];
    [surface.rulerMeasurementLabelSurfaces removeLastObject];
  }
  [owned enumerateObjectsUsingBlock:^(NSValue *value, NSUInteger index,
                                      __unused BOOL *stop) {
    NativeRulerMeasurement measurement = {0};
    [value getValue:&measurement size:sizeof(measurement)];
    render_measurement_label(surface,
                             surface.rulerMeasurementLabelSurfaces[index],
                             measurement);
  }];
}

static ScreenwideRegionOSC *probe_label_surface(
    ScreenwideRegionOSC *surface, NativeRulerProbe probe) {
  NSPoint midpoint = has_label_anchor(probe.label_anchor_x,
                                      probe.label_anchor_y)
      ? NSMakePoint(probe.label_anchor_x, probe.label_anchor_y)
      : probe.axis == 1
      ? NSMakePoint((probe.start + probe.end) * 0.5, probe.position)
      : NSMakePoint(probe.position, (probe.start + probe.end) * 0.5);
  return label_anchor_surface(surface, midpoint);
}

static NSString *stamped_probe_text(NativeRulerProbe probe) {
  NSInteger distance =
      MAX((NSInteger)llround(fabs(probe.end - probe.start)), 0);
  return [NSString stringWithFormat:@"%ld px", (long)distance];
}

static NSString *radius_text(NativeRulerRadius radius) {
  NSInteger value = MAX((NSInteger)llround(radius.radius), 0);
  return (radius.flags & 1) != 0
      ? [NSString stringWithFormat:@"≈ %ld px", (long)value]
      : [NSString stringWithFormat:@"%ld px", (long)value];
}

static void render_probe_label(ScreenwideRegionOSC *surface,
                               ScreenwideOscMaterialSurfaceView *control,
                               NativeRulerProbe probe, uint8_t labelKind) {
  ScreenwideRulerLabelRenderState *renderState =
      label_render_state(surface, control);
  renderState.labelKind = labelKind;
  renderState.probe = probe;
  CGFloat scale = surface.host.window.backingScaleFactor ?: 1.0;
  update_label(surface, scale);
  if (!surface.rulerLabel)
    return;
  NSString *text = labelKind == 4 ? radius_text(renderState.radius)
                                  : stamped_probe_text(probe);
  control.accessibilityElement = YES;
  control.accessibilityRole = NSAccessibilityStaticTextRole;
  control.accessibilityLabel = labelKind == 4
      ? @"Corner radius"
      : labelKind == 3 ? @"Guide spacing" : @"Distance";
  control.accessibilityValue = text;
  CGFloat cellWidth = surface.rulerLabel.atlasGlyphWidth;
  ScreenwideOscControlMetrics value = metrics();
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  CGFloat width = value.padding_x * 2.0 + cellWidth * text.length;
  CGFloat height = value.height;
  CGFloat start = 0.0, end = 0.0, position = 0.0;
  project_probe(surface, probe, &start, &end, &position);
  CGFloat left = probe.axis == 1
      ? (start + end - width) * 0.5
      : position - width * 0.5;
  CGFloat top = probe.axis == 1
      ? position - height * 0.5
      : (start + end - height) * 0.5;
  if (labelKind == 4 &&
      !has_label_anchor(probe.label_anchor_x, probe.label_anchor_y)) {
    NativeRulerRadius radius = renderState.radius;
    CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
    BOOL right = radius.corner == 2 || radius.corner == 4;
    BOOL bottom = radius.corner == 3 || radius.corner == 4;
    CGFloat cornerX = radius.x + (right ? radius.width : 0.0);
    CGFloat cornerY = radius.y + (bottom ? radius.height : 0.0);
    cornerX = (cornerX - surface.desktopOffset.x -
               surface.rulerViewportOrigin.x) * zoom;
    cornerY = (cornerY - surface.desktopOffset.y -
               surface.rulerViewportOrigin.y) * zoom;
    left = cornerX + (right ? spacing.control : -width - spacing.control);
    top = cornerY + (bottom ? spacing.control : -height - spacing.control);
  } else if (has_label_anchor(probe.label_anchor_x, probe.label_anchor_y)) {
    NSPoint anchor = project_label_anchor(
        surface, NSMakePoint(probe.label_anchor_x,
                             probe.label_anchor_y));
    left = anchor.x - width * 0.5;
    top = anchor.y - height * 0.5;
  }
  NSSize host = surface.host.bounds.size;
  left = MAX(spacing.control_inset,
             MIN(left, host.width - width - spacing.control_inset));
  top = MAX(spacing.control_inset,
            MIN(top, host.height - height - spacing.control_inset));

  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  control.frame = NSMakeRect(left, host.height - top - height, width, height);
  control.layer.cornerRadius = value.radius;
  control.contentLayer.cornerRadius = value.radius;
  control.contentView.frame = control.bounds;
  [CATransaction commit];
  control.hidden = NO;

  if (renderState.inFlight) {
    renderState.pending = YES;
    return;
  }
  renderState.inFlight = YES;
  renderState.pending = NO;
  ScreenwideRegionOscRenderState state =
      screenwide_region_osc_render_state(light_mode(surface));
  screenwide_region_osc_ruler_apply_render_state(surface, &state);
  ScreenwideRegionOscVertex vertices[96];
  NSUInteger count = 0;
  NSSize size = NSMakeSize(width, height);
  screenwide_region_osc_add_quad(
      vertices, &count, size, NSMakeRect(0.0, 0.0, width, height), 12);
  CGFloat textLeft = value.padding_x;
  CGFloat textTop = (height - surface.rulerLabel.size.height) * 0.5;
  for (NSUInteger index = 0; index < text.length; index++) {
    screenwide_region_osc_add_texture_quad(
        vertices, &count, size,
        NSMakeRect(textLeft + cellWidth * index, textTop, cellWidth,
                   surface.rulerLabel.size.height),
        glyph_texture_rect(surface.rulerLabel,
                           glyph_index([text characterAtIndex:index])),
        11);
  }

  control.contentLayer.contentsScale = scale;
  control.contentLayer.drawableSize =
      CGSizeMake(MAX(width * scale, 2.0), MAX(height * scale, 2.0));
  id<CAMetalDrawable> drawable = [control.contentLayer nextDrawable];
  if (!drawable) {
    renderState.inFlight = NO;
    return;
  }
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
  screenwide_region_osc_encode(encoder, surface.pipeline, buffer, count,
                               state, surface.rulerLabel.texture,
                               surface.placeholder);
  [encoder endEncoding];
  __weak ScreenwideRulerLabelRenderState *weakRenderState = renderState;
  [drawable addPresentedHandler:^(__unused id<MTLDrawable> presented) {
    dispatch_async(dispatch_get_main_queue(), ^{
      ScreenwideRulerLabelRenderState *latest = weakRenderState;
      if (!latest)
        return;
      latest.inFlight = NO;
      if (!latest.pending)
        return;
      latest.pending = NO;
      ScreenwideRegionOSC *latestSurface = latest.surface;
      ScreenwideOscMaterialSurfaceView *latestControl = latest.control;
      if (latestSurface && latestControl) {
        if (latest.labelKind != 1)
          render_probe_label(latestSurface, latestControl, latest.probe,
                             latest.labelKind);
        else
          render_measurement_label(latestSurface, latestControl,
                                   latest.measurement);
      }
    });
  }];
  [command presentDrawable:drawable];
  [command commit];
}

static void render_probe_labels(ScreenwideRegionOSC *surface) {
  NSMutableArray<NSValue *> *owned = [NSMutableArray array];
  const NativeRulerProbe *items = probes(surface);
  NSUInteger count = probe_count(surface);
  for (NSUInteger index = 0; index < count; index++) {
    BOOL draft = (items[index].flags & 1) != 0;
    BOOL labelled = (items[index].flags & 4) == 0 &&
        (items[index].flags & 8) == 0 &&
        (items[index].id != 0 || draft);
    if (labelled && probe_label_surface(surface, items[index]) == surface)
      [owned addObject:[NSValue valueWithBytes:&items[index]
                                     objCType:@encode(NativeRulerProbe)]];
  }
  while (surface.rulerProbeLabelSurfaces.count < owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        screenwide_osc_material_surface(surface.device);
    control.accessibilityElement = YES;
    control.accessibilityRole = NSAccessibilityStaticTextRole;
    control.accessibilityLabel = @"Distance";
    [surface.host addSubview:control
                  positioned:NSWindowBelow relativeTo:surface.rulerSurface];
    [surface.rulerProbeLabelSurfaces addObject:control];
  }
  while (surface.rulerProbeLabelSurfaces.count > owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.rulerProbeLabelSurfaces.lastObject;
    [control removeFromSuperview];
    [surface.rulerProbeLabelSurfaces removeLastObject];
  }
  [owned enumerateObjectsUsingBlock:^(NSValue *value, NSUInteger index,
                                      __unused BOOL *stop) {
    NativeRulerProbe probe = {0};
    [value getValue:&probe size:sizeof(probe)];
    render_probe_label(surface, surface.rulerProbeLabelSurfaces[index], probe,
                       2);
  }];
}

static NativeRulerProbe guide_gap_probe(NativeRulerGuideGap gap) {
  NativeRulerProbe probe = {0};
  probe.id = gap.id;
  probe.display_id = gap.display_id;
  probe.axis = gap.axis;
  probe.flags = (gap.flags & 2) != 0 ? 8 : 0;
  probe.start = gap.start;
  probe.end = gap.end;
  probe.position = gap.position;
  probe.label_anchor_x = gap.label_anchor_x;
  probe.label_anchor_y = gap.label_anchor_y;
  return probe;
}

static void render_guide_gap_labels(ScreenwideRegionOSC *surface) {
  NSMutableArray<NSValue *> *owned = [NSMutableArray array];
  const NativeRulerGuideGap *items = guide_gaps(surface);
  NSUInteger count = guide_gap_count(surface);
  for (NSUInteger index = 0; index < count; index++) {
    if ((items[index].flags & 2) != 0)
      continue;
    NativeRulerProbe probe = guide_gap_probe(items[index]);
    if (probe_label_surface(surface, probe) == surface)
      [owned addObject:[NSValue valueWithBytes:&probe
                                     objCType:@encode(NativeRulerProbe)]];
  }
  while (surface.rulerGuideGapLabelSurfaces.count < owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        screenwide_osc_material_surface(surface.device);
    control.accessibilityElement = YES;
    control.accessibilityRole = NSAccessibilityStaticTextRole;
    control.accessibilityLabel = @"Guide spacing";
    [surface.host addSubview:control
                  positioned:NSWindowBelow relativeTo:surface.rulerSurface];
    [surface.rulerGuideGapLabelSurfaces addObject:control];
  }
  while (surface.rulerGuideGapLabelSurfaces.count > owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.rulerGuideGapLabelSurfaces.lastObject;
    [control removeFromSuperview];
    [surface.rulerGuideGapLabelSurfaces removeLastObject];
  }
  [owned enumerateObjectsUsingBlock:^(NSValue *value, NSUInteger index,
                                      __unused BOOL *stop) {
    NativeRulerProbe probe = {0};
    [value getValue:&probe size:sizeof(probe)];
    render_probe_label(surface,
                       surface.rulerGuideGapLabelSurfaces[index], probe, 3);
  }];
}

static NSPoint radius_center(NativeRulerRadius radius) {
  BOOL right = radius.corner == 2 || radius.corner == 4;
  BOOL bottom = radius.corner == 3 || radius.corner == 4;
  CGFloat cornerX = radius.x + (right ? radius.width : 0.0);
  CGFloat cornerY = radius.y + (bottom ? radius.height : 0.0);
  return NSMakePoint(cornerX + (right ? -radius.radius : radius.radius),
                     cornerY + (bottom ? -radius.radius : radius.radius));
}

static NSPoint radius_arc_midpoint(NativeRulerRadius radius) {
  BOOL right = radius.corner == 2 || radius.corner == 4;
  BOOL bottom = radius.corner == 3 || radius.corner == 4;
  NSPoint center = radius_center(radius);
  CGFloat diagonal = M_SQRT1_2 * radius.radius;
  return NSMakePoint(center.x + (right ? diagonal : -diagonal),
                     center.y + (bottom ? diagonal : -diagonal));
}

static NativeRulerProbe radius_label_probe(NativeRulerRadius radius) {
  NativeRulerProbe probe = {0};
  NSPoint midpoint = radius_arc_midpoint(radius);
  probe.id = radius.id;
  probe.display_id = radius.display_id;
  probe.axis = 1;
  probe.flags = (radius.flags & 8) != 0 ? 8 : 0;
  probe.start = midpoint.x - radius.radius * 0.5;
  probe.end = midpoint.x + radius.radius * 0.5;
  probe.position = midpoint.y;
  probe.label_anchor_x = radius.label_anchor_x;
  probe.label_anchor_y = radius.label_anchor_y;
  return probe;
}

static void render_radius_labels(ScreenwideRegionOSC *surface) {
  NSMutableArray<NSValue *> *owned = [NSMutableArray array];
  const NativeRulerRadius *items = radii(surface);
  NSUInteger count = radius_count(surface);
  for (NSUInteger index = 0; index < count; index++) {
    if ((items[index].flags & 8) != 0)
      continue;
    NativeRulerProbe probe = radius_label_probe(items[index]);
    if (probe_label_surface(surface, probe) == surface)
      [owned addObject:[NSValue valueWithBytes:&items[index]
                                     objCType:@encode(NativeRulerRadius)]];
  }
  while (surface.rulerRadiusLabelSurfaces.count < owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        screenwide_osc_material_surface(surface.device);
    control.accessibilityElement = YES;
    control.accessibilityRole = NSAccessibilityStaticTextRole;
    control.accessibilityLabel = @"Corner radius";
    [surface.host addSubview:control
                  positioned:NSWindowBelow relativeTo:surface.rulerSurface];
    [surface.rulerRadiusLabelSurfaces addObject:control];
  }
  while (surface.rulerRadiusLabelSurfaces.count > owned.count) {
    ScreenwideOscMaterialSurfaceView *control =
        surface.rulerRadiusLabelSurfaces.lastObject;
    [control removeFromSuperview];
    [surface.rulerRadiusLabelSurfaces removeLastObject];
  }
  [owned enumerateObjectsUsingBlock:^(NSValue *value, NSUInteger index,
                                      __unused BOOL *stop) {
    NativeRulerRadius radius = {0};
    [value getValue:&radius size:sizeof(radius)];
    ScreenwideOscMaterialSurfaceView *control =
        surface.rulerRadiusLabelSurfaces[index];
    ScreenwideRulerLabelRenderState *state =
        label_render_state(surface, control);
    state.radius = radius;
    render_probe_label(surface, control, radius_label_probe(radius), 4);
  }];
}

void screenwide_region_osc_ruler_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale) {
  if (!surface.rulerVisible) return;
  NSPoint point = surface.rulerPoint;
  if (surface.rulerCrosshair) {
    CGFloat x = screenwide_region_osc_snap(point.x, scale);
    CGFloat y = screenwide_region_osc_snap(point.y, scale);
    if (x >= 0.0 && x <= size.width)
      screenwide_region_osc_add_line(vertices, count, size,
                                     NSMakePoint(x, 0.0),
                                     NSMakePoint(x, size.height),
                                     1.0 / scale, 28);
    if (y >= 0.0 && y <= size.height)
      screenwide_region_osc_add_line(vertices, count, size,
                                     NSMakePoint(0.0, y),
                                     NSMakePoint(size.width, y),
                                     1.0 / scale, 28);
  }
  const NativeRulerProbe *probeItems = probes(surface);
  NSUInteger probeCount = probe_count(surface);
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  CGFloat hoverWidth = ruler_hover_width(surface);
  for (NSUInteger index = 0; index < probeCount; index++) {
    NativeRulerProbe probe = probeItems[index];
    BOOL live = (probe.flags & 4) != 0;
    if ((live && !surface.rulerTransientChromeVisible) ||
        (live && probe.display_id != surface.displayID))
      continue;
    CGFloat start = 0.0, end = 0.0, position = 0.0;
    project_probe(surface, probe, &start, &end, &position);
    if (start > end) {
      CGFloat swap = start;
      start = end;
      end = swap;
    }
    if ((probe.flags & 2) != 0)
      screenwide_region_osc_add_line(
          vertices, count, size,
          probe.axis == 1 ? NSMakePoint(start, position)
                          : NSMakePoint(position, start),
          probe.axis == 1 ? NSMakePoint(end, position)
                          : NSMakePoint(position, end),
          hoverWidth, 32);
    screenwide_region_osc_add_line(
        vertices, count, size,
        probe.axis == 1 ? NSMakePoint(start, position)
                        : NSMakePoint(position, start),
        probe.axis == 1 ? NSMakePoint(end, position)
                        : NSMakePoint(position, end),
        1.0 / scale, 28);
    CGFloat tick = spacing.control;
    screenwide_region_osc_add_line(
        vertices, count, size,
        probe.axis == 1 ? NSMakePoint(start, position - tick)
                        : NSMakePoint(position - tick, start),
        probe.axis == 1 ? NSMakePoint(start, position + tick)
                        : NSMakePoint(position + tick, start),
        1.0 / scale, 28);
    screenwide_region_osc_add_line(
        vertices, count, size,
        probe.axis == 1 ? NSMakePoint(end, position - tick)
                        : NSMakePoint(position - tick, end),
        probe.axis == 1 ? NSMakePoint(end, position + tick)
                        : NSMakePoint(position + tick, end),
        1.0 / scale, 28);
  }
  const NativeRulerGuide *guideItems = guides(surface);
  NSUInteger guideCount = guide_count(surface);
  for (NSUInteger index = 0; index < guideCount; index++) {
    NativeRulerGuide guide = guideItems[index];
    if (guide.display_id != surface.displayID)
      continue;
    CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
    if (guide.axis == 1) {
      CGFloat x = (guide.position - surface.desktopOffset.x -
                   surface.rulerViewportOrigin.x) * zoom;
      x = screenwide_region_osc_snap(x, scale);
      if (x >= 0.0 && x <= size.width) {
        if ((guide.flags & 2) != 0)
          screenwide_region_osc_add_line(vertices, count, size,
                                         NSMakePoint(x, 0.0),
                                         NSMakePoint(x, size.height),
                                         hoverWidth, 38);
        screenwide_region_osc_add_line(vertices, count, size,
                                       NSMakePoint(x, 0.0),
                                       NSMakePoint(x, size.height),
                                       1.0 / scale, 36);
      }
    } else if (guide.axis == 2) {
      CGFloat y = (guide.position - surface.desktopOffset.y -
                   surface.rulerViewportOrigin.y) * zoom;
      y = screenwide_region_osc_snap(y, scale);
      if (y >= 0.0 && y <= size.height) {
        if ((guide.flags & 2) != 0)
          screenwide_region_osc_add_line(vertices, count, size,
                                         NSMakePoint(0.0, y),
                                         NSMakePoint(size.width, y),
                                         hoverWidth, 38);
        screenwide_region_osc_add_line(vertices, count, size,
                                       NSMakePoint(0.0, y),
                                       NSMakePoint(size.width, y),
                                       1.0 / scale, 36);
      }
    }
  }
  const NativeRulerGuideGap *gapItems = guide_gaps(surface);
  NSUInteger gapCount = guide_gap_count(surface);
  for (NSUInteger index = 0; index < gapCount; index++) {
    NativeRulerGuideGap gap = gapItems[index];
    if (gap.display_id != surface.displayID || (gap.flags & 2) != 0)
      continue;
    NativeRulerProbe probe = guide_gap_probe(gap);
    CGFloat start = 0.0, end = 0.0, position = 0.0;
    project_probe(surface, probe, &start, &end, &position);
    if (start > end) {
      CGFloat swap = start;
      start = end;
      end = swap;
    }
    NSPoint lineStart = gap.axis == 1 ? NSMakePoint(start, position)
                                      : NSMakePoint(position, start);
    NSPoint lineEnd = gap.axis == 1 ? NSMakePoint(end, position)
                                    : NSMakePoint(position, end);
    if ((gap.flags & 1) != 0)
      screenwide_region_osc_add_line(vertices, count, size, lineStart,
                                     lineEnd, hoverWidth, 38);
    screenwide_region_osc_add_line(vertices, count, size, lineStart, lineEnd,
                                   1.0 / scale, 36);
    CGFloat tick = spacing.control;
    screenwide_region_osc_add_line(
        vertices, count, size,
        gap.axis == 1 ? NSMakePoint(start, position - tick)
                      : NSMakePoint(position - tick, start),
        gap.axis == 1 ? NSMakePoint(start, position + tick)
                      : NSMakePoint(position + tick, start),
        1.0 / scale, 36);
    screenwide_region_osc_add_line(
        vertices, count, size,
        gap.axis == 1 ? NSMakePoint(end, position - tick)
                      : NSMakePoint(position - tick, end),
        gap.axis == 1 ? NSMakePoint(end, position + tick)
                      : NSMakePoint(position + tick, end),
        1.0 / scale, 36);
  }
  const NativeRulerRadius *radiusItems = radii(surface);
  NSUInteger radiusCount = radius_count(surface);
  for (NSUInteger index = 0; index < radiusCount; index++) {
    NativeRulerRadius radius = radiusItems[index];
    if (radius.display_id != surface.displayID || radius.radius <= 0.0)
      continue;
    CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
    NSPoint centerWorld = radius_center(radius);
    NSPoint center = NSMakePoint(
        (centerWorld.x - surface.desktopOffset.x -
         surface.rulerViewportOrigin.x) * zoom,
        (centerWorld.y - surface.desktopOffset.y -
         surface.rulerViewportOrigin.y) * zoom);
    center.x = screenwide_region_osc_snap(center.x, scale);
    center.y = screenwide_region_osc_snap(center.y, scale);
    CGFloat value = MAX(round(radius.radius * zoom * scale) / scale,
                        1.0 / scale);
    BOOL lowConfidence = (radius.flags & 1) != 0;
    BOOL hovered = (radius.flags & 4) != 0;
    screenwide_region_osc_add_ruler_arc(
        vertices, count, size, center, value, radius.corner, scale,
        hovered, hoverWidth, lowConfidence);
  }
  const NativeRulerCenterline *centerItems = centerlines(surface);
  NSUInteger centerCount = centerline_count(surface);
  for (NSUInteger index = 0; index < centerCount; index++) {
    NativeRulerCenterline line = centerItems[index];
    NSRect frame = project_world_rect(surface, line.x, line.y, line.width,
                                      line.height);
    CGFloat centerX = screenwide_region_osc_snap(NSMidX(frame), scale);
    CGFloat centerY = screenwide_region_osc_snap(NSMidY(frame), scale);
    screenwide_region_osc_add_line(
        vertices, count, size, NSMakePoint(centerX, NSMinY(frame)),
        NSMakePoint(centerX, NSMaxY(frame)), 1.0 / scale,
        (line.flags & 1) != 0 ? 43 : 42);
    screenwide_region_osc_add_line(
        vertices, count, size, NSMakePoint(NSMinX(frame), centerY),
        NSMakePoint(NSMaxX(frame), centerY), 1.0 / scale,
        (line.flags & 2) != 0 ? 43 : 42);
  }
  const NativeRulerInnerObject *objectItems = inner_objects(surface);
  NSUInteger objectCount = inner_object_count(surface);
  for (NSUInteger index = 0; index < objectCount; index++) {
    NativeRulerInnerObject object = objectItems[index];
    NSRect frame = project_world_rect(surface, object.x, object.y,
                                      object.width, object.height);
    add_center_object_outline(vertices, count, size, frame, scale);
    CGFloat centerX = screenwide_region_osc_snap(NSMidX(frame), scale);
    CGFloat centerY = screenwide_region_osc_snap(NSMidY(frame), scale);
    CGFloat zoom = MAX(surface.rulerViewportZoom, 1.0);
    if ((object.flags & 1) != 0) {
      CGFloat halfTick = MIN(NSHeight(frame), 12.0 * zoom) * 0.5;
      screenwide_region_osc_add_line(
          vertices, count, size,
          NSMakePoint(centerX, centerY - halfTick),
          NSMakePoint(centerX, centerY + halfTick), 2.5 / scale, 43);
    }
    if ((object.flags & 2) != 0) {
      CGFloat halfTick = MIN(NSWidth(frame), 12.0 * zoom) * 0.5;
      screenwide_region_osc_add_line(
          vertices, count, size,
          NSMakePoint(centerX - halfTick, centerY),
          NSMakePoint(centerX + halfTick, centerY), 2.5 / scale, 43);
    }
  }
  const NativeRulerMeasurement *items = measurements(surface);
  NSUInteger measurementCount = measurement_count(surface);
  for (NSUInteger index = 0; index < measurementCount; index++) {
    NSRect frame = project_measurement(surface, items[index]);
    screenwide_region_osc_add_ruler_box(
        vertices, count, size, frame, scale,
        (items[index].flags & 4) != 0, hoverWidth);
  }
}
