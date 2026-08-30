// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#import <QuartzCore/CATransaction.h>

static CGColorRef color(const float rgba[4]) {
  return CGColorCreateSRGB(rgba[0], rgba[1], rgba[2], rgba[3]);
}

void screenwide_region_osc_ocr_update_appearance(
    ScreenwideRegionOSC *surface) {
  NSString *appearance = [surface.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  uint32_t light = [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
  ScreenwideOscOcrPalette palette = screenwide_osc_ocr_palette(light);
  const float *fill = surface.ocrPhase == 3 ? palette.status_error_fill
                                            : palette.loading_fill;
  const float *foreground = surface.ocrPhase == 3
                                ? palette.status_error_foreground
                                : palette.loading_foreground;
  const float *outline = surface.ocrPhase == 3 ? palette.error_outline
                                               : palette.primary_outline;
  CGColorRef fillColor = color(fill);
  CGColorRef outlineColor = color(outline);
  CGColorRef foregroundColor = color(foreground);
  surface.ocrStatusSurface.contentLayer.backgroundColor = fillColor;
  surface.ocrStatusSurface.contentLayer.borderColor = outlineColor;
  surface.ocrStatusSurface.contentLayer.borderWidth = 1.0;
  surface.ocrStatusLabel.textColor = [NSColor colorWithCGColor:foregroundColor];
  CGColorRelease(fillColor);
  CGColorRelease(outlineColor);
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
  if (!visible)
    return;
  surface.ocrStatusLabel.stringValue = message;
  surface.ocrStatusLabel.toolTip = message;
  screenwide_region_osc_ocr_update_appearance(surface);
  NSSize host = surface.host.bounds.size;
  CGFloat width = MIN(MAX(surface.ocrStatusLabel.intrinsicContentSize.width + 24.0,
                          128.0),
                      MAX(host.width - 16.0, 0.0));
  CGFloat height = 28.0;
  CGFloat top = MIN(MAX(NSMidY(surface.region) - height * 0.5, 8.0),
                    MAX(host.height - height - 8.0, 8.0));
  CGFloat left = MIN(MAX(NSMidX(surface.region) - width * 0.5, 8.0),
                     MAX(host.width - width - 8.0, 8.0));
  [CATransaction begin];
  [CATransaction setDisableActions:YES];
  status.frame = NSMakeRect(left, host.height - top - height, width, height);
  status.layer.cornerRadius = 8.0;
  status.contentLayer.cornerRadius = 8.0;
  status.contentView.frame = status.bounds;
  surface.ocrStatusLabel.frame = NSInsetRect(status.contentView.bounds, 12.0, 4.0);
  [CATransaction commit];
}

void screenwide_region_osc_ocr_attach(ScreenwideRegionOSC *surface) {
  surface.ocrRects = [NSMutableData data];
  surface.ocrStatusSurface = screenwide_osc_material_surface(surface.device);
  surface.ocrStatusLabel = [NSTextField labelWithString:@""];
  surface.ocrStatusLabel.alignment = NSTextAlignmentCenter;
  surface.ocrStatusLabel.font = [NSFont systemFontOfSize:13.0
                                                  weight:NSFontWeightMedium];
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
