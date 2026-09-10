// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <QuartzCore/CAAnimation.h>
#import <QuartzCore/CATransaction.h>

static CGColorRef color(const float rgba[4]) {
  return CGColorCreateSRGB(rgba[0], rgba[1], rgba[2], rgba[3]);
}

/// The recognising indicator: an `--spacing-icon-small` ring drawn as a three
/// quarter stroke and spun once a second, the OSC stand-in for AppKit's
/// indeterminate progress spinner. It is a plain CAShapeLayer rather than an
/// NSProgressIndicator so it can carry the label colour exactly and sit inside
/// the pill's own content layer.
static const CGFloat kStatusSpinnerSize = 14.0;
static const CGFloat kStatusSpinnerLineWidth = 1.5;
static const CFTimeInterval kStatusSpinnerPeriod = 1.0;
static NSString *const kStatusSpinnerKey = @"screenwide.ocr.status.spin";

static CAShapeLayer *status_spinner(ScreenwideRegionOSC *surface) {
  if (surface.ocrStatusSpinner || !surface.ocrStatusSurface)
    return surface.ocrStatusSpinner;
  CAShapeLayer *ring = [CAShapeLayer layer];
  ring.fillColor = NULL;
  ring.lineWidth = kStatusSpinnerLineWidth;
  ring.lineCap = kCALineCapRound;
  ring.bounds = CGRectMake(0.0, 0.0, kStatusSpinnerSize, kStatusSpinnerSize);
  ring.anchorPoint = CGPointMake(0.5, 0.5);
  CGFloat center = kStatusSpinnerSize * 0.5;
  CGFloat radius = center - kStatusSpinnerLineWidth * 0.5;
  CGMutablePathRef path = CGPathCreateMutable();
  CGPathAddArc(path, NULL, center, center, radius, M_PI_2, -M_PI, YES);
  ring.path = path;
  CGPathRelease(path);
  [surface.ocrStatusSurface.contentView.layer addSublayer:ring];
  surface.ocrStatusSpinner = ring;
  return ring;
}

static void spin_status_spinner(CAShapeLayer *ring, BOOL running) {
  if (!ring)
    return;
  if (!running) {
    [ring removeAnimationForKey:kStatusSpinnerKey];
    return;
  }
  if ([ring animationForKey:kStatusSpinnerKey])
    return;
  CABasicAnimation *spin =
      [CABasicAnimation animationWithKeyPath:@"transform.rotation.z"];
  spin.fromValue = @0.0;
  spin.toValue = @(-M_PI * 2.0);
  spin.duration = kStatusSpinnerPeriod;
  spin.repeatCount = HUGE_VALF;
  spin.removedOnCompletion = NO;
  [ring addAnimation:spin forKey:kStatusSpinnerKey];
}

void screenwide_region_osc_ocr_update_appearance(
    ScreenwideRegionOSC *surface) {
  NSString *appearance = [surface.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  uint32_t light = [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
  ScreenwideOscOcrPalette palette = screenwide_osc_ocr_palette(light);
  BOOL failed = surface.ocrPhase == 3;
  const float *foreground = failed ? palette.status_error_foreground
                                   : palette.loading_foreground;
  CGColorRef foregroundColor = color(foreground);
  // The material surface is the pill's backing, so the content layer stays
  // clear and unbordered: only the label and the ring carry a colour.
  surface.ocrStatusSurface.contentLayer.backgroundColor = NULL;
  surface.ocrStatusSurface.contentLayer.borderColor = NULL;
  surface.ocrStatusSurface.contentLayer.borderWidth = 0.0;
  surface.ocrStatusLabel.textColor = [NSColor colorWithCGColor:foregroundColor];
  CAShapeLayer *ring = status_spinner(surface);
  ring.strokeColor = foregroundColor;
  ring.hidden = failed;
  spin_status_spinner(ring, !failed && !surface.ocrStatusSurface.hidden);
  CGColorRelease(foregroundColor);
  screenwide_region_osc_ocr_cancel_update_appearance(surface);
  if (surface.ocrToolbarVisible)
    screenwide_region_osc_ocr_toolbar_layout(surface, YES);
  screenwide_region_osc_ocr_toolbar_render(surface);
}

static void layout_status(ScreenwideRegionOSC *surface, NSString *message,
                          BOOL visible) {
  ScreenwideOscMaterialSurfaceView *status = surface.ocrStatusSurface;
  status.hidden = !visible;
  if (!visible) {
    spin_status_spinner(surface.ocrStatusSpinner, NO);
    return;
  }
  surface.ocrStatusLabel.stringValue = message;
  surface.ocrStatusLabel.toolTip = message;
  screenwide_region_osc_ocr_update_appearance(surface);
  ScreenwideOscControlMetrics metrics = screenwide_osc_control_metrics(0, 0);
  ScreenwideOscControlSpacing spacing = screenwide_osc_control_spacing();
  NSSize host = surface.host.bounds.size;
  // The ring only shows while a recognition is running, so an error message
  // starts at the same padding a plain label would.
  BOOL spinning = surface.ocrPhase != 3;
  CGFloat leading = spinning ? kStatusSpinnerSize + spacing.control : 0.0;
  CGFloat height = metrics.height;
  CGFloat inset = spacing.control_inset;
  CGFloat width = MIN(surface.ocrStatusLabel.intrinsicContentSize.width +
                          spacing.section * 2.0 + leading,
                      MAX(host.width - inset * 2.0, 0.0));
  CGFloat top = MIN(MAX(NSMidY(surface.region) - height * 0.5, inset),
                    MAX(host.height - height - inset, inset));
  CGFloat left = MIN(MAX(NSMidX(surface.region) - width * 0.5, inset),
                     MAX(host.width - width - inset, inset));
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  status.frame = NSMakeRect(left, host.height - top - height, width, height);
  status.layer.cornerRadius = metrics.radius;
  status.contentLayer.cornerRadius = metrics.radius;
  status.contentView.frame = status.bounds;
  NSRect content = status.contentView.bounds;
  CAShapeLayer *ring = surface.ocrStatusSpinner;
  ring.contentsScale = surface.host.window.backingScaleFactor ?: 1.0;
  ring.position = CGPointMake(spacing.section + kStatusSpinnerSize * 0.5,
                              NSMidY(content));
  CGFloat labelHeight = surface.ocrStatusLabel.intrinsicContentSize.height;
  surface.ocrStatusLabel.frame = NSMakeRect(
      spacing.section + leading,
      floor((NSHeight(content) - labelHeight) * 0.5),
      MAX(NSWidth(content) - spacing.section * 2.0 - leading, 0.0),
      labelHeight);
  [CATransaction commit];
}

void screenwide_region_osc_ocr_attach(ScreenwideRegionOSC *surface) {
  surface.ocrRects = [NSMutableData data];
  surface.ocrStatusSurface = screenwide_osc_material_surface(surface.device);
  surface.ocrStatusLabel = [NSTextField labelWithString:@""];
  surface.ocrStatusLabel.alignment = NSTextAlignmentCenter;
  surface.ocrStatusLabel.font = screenwide_osc_body_font();
  surface.ocrStatusLabel.lineBreakMode = NSLineBreakByTruncatingTail;
  surface.ocrStatusLabel.maximumNumberOfLines = 1;
  [surface.ocrStatusSurface.contentView addSubview:surface.ocrStatusLabel];
  [surface.host addSubview:surface.ocrStatusSurface
                positioned:NSWindowAbove relativeTo:nil];
  screenwide_region_osc_ocr_cancel_attach(surface);
  screenwide_region_osc_ocr_toolbar_attach(surface);
}

void screenwide_region_osc_ocr_teardown(ScreenwideRegionOSC *surface) {
  [surface.ocrStatusSurface removeFromSuperview];
  surface.ocrStatusSurface = nil;
  surface.ocrStatusLabel = nil;
  [surface.ocrStatusSpinner removeFromSuperlayer];
  surface.ocrStatusSpinner = nil;
  screenwide_region_osc_ocr_cancel_teardown(surface);
  screenwide_region_osc_ocr_toolbar_teardown(surface);
  surface.ocrRects = nil;
  surface.ocrPhase = 0;
}

NSUInteger screenwide_region_osc_ocr_vertex_capacity(
    ScreenwideRegionOSC *surface) {
  NSUInteger rects = surface.ocrRects.length / sizeof(ScreenwideRegionOcrRect);
  return rects * 6 + (surface.ocrPhase == 1 || surface.ocrPhase == 2 ? 24 : 0);
}

void screenwide_region_osc_ocr_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale) {
  if ((surface.ocrPhase == 1 || surface.ocrPhase == 2) &&
      !NSIsEmptyRect(surface.region)) {
    CGFloat half = 1.0 / MAX(scale, 1.0);
    NSRect region = surface.region;
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(NSMinX(region) - half, NSMinY(region) - half,
                   NSWidth(region) + half * 2.0, half * 2.0), 18);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(NSMinX(region) - half, NSMaxY(region) - half,
                   NSWidth(region) + half * 2.0, half * 2.0), 18);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(NSMinX(region) - half, NSMinY(region) - half,
                   half * 2.0, NSHeight(region) + half * 2.0), 18);
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(NSMaxX(region) - half, NSMinY(region) - half,
                   half * 2.0, NSHeight(region) + half * 2.0), 18);
  }
  const ScreenwideRegionOcrRect *rects = surface.ocrRects.bytes;
  NSUInteger length = surface.ocrRects.length / sizeof(*rects);
  for (NSUInteger index = 0; index < length; index++) {
    uint32_t kind = rects[index].kind == 4 ? 20 :
                    rects[index].kind == 3 ? 19 :
                    rects[index].kind == 2 ? 18 : 17;
    screenwide_region_osc_add_quad(
        vertices, count, size,
        NSMakeRect(rects[index].x, rects[index].y,
                   rects[index].width, rects[index].height), kind);
  }
}

int screenwide_region_osc_set_ocr(void *view_ptr, uint32_t phase,
                                  const ScreenwideRegionOcrRect *rects,
                                  size_t count, const char *message) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root || (count > 0 && !rects) || phase > 3)
    return 0;
  NSString *text = message ? [NSString stringWithUTF8String:message] : @"";
  NSArray<ScreenwideRegionOSC *> *surfaces =
      screenwide_region_osc_surfaces(root);
  ScreenwideRegionOSC *target = nil;
  CGFloat bestArea = 0.0;
  for (ScreenwideRegionOSC *surface in surfaces) {
    surface.ocrPhase = phase;
    surface.ocrRects.length = 0;
    for (size_t index = 0; index < count; index++) {
      ScreenwideRegionOcrRect local = rects[index];
      local.x -= surface.desktopOffset.x;
      local.y -= surface.desktopOffset.y;
      NSRect localRect = NSMakeRect(local.x, local.y, local.width, local.height);
      if (NSIntersectsRect(localRect, surface.host.bounds))
        [surface.ocrRects appendBytes:&local length:sizeof(local)];
    }
    NSRect intersection = NSIntersectionRect(
        surface.region, NSMakeRect(0, 0, surface.host.bounds.size.width,
                                   surface.host.bounds.size.height));
    CGFloat area = NSWidth(intersection) * NSHeight(intersection);
    if (area > bestArea) {
      bestArea = area;
      target = surface;
    }
  }
  BOOL showStatus = phase == 1 || phase == 3;
  for (ScreenwideRegionOSC *surface in surfaces) {
    if (phase == 2 && surface.ocrCancelVisible) {
      surface.ocrCancelVisible = NO;
      surface.ocrCancelSurface.hidden = YES;
    }
    if (phase != 2) {
      surface.ocrToolbarCloseArmed = NO;
      surface.ocrToolbarCloseRevision += 1;
    }
    layout_status(surface, text, showStatus && surface == target);
    screenwide_region_osc_ocr_toolbar_layout(
        surface, phase == 2 && surface == target);
    screenwide_region_osc_draw(surface);
  }
  return 1;
}
