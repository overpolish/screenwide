// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

static const CGFloat SCREENWIDE_KEYCAP_HEIGHT = 20.0;
static const CGFloat SCREENWIDE_KEYCAP_MINIMUM_WIDTH = 20.0;
static const CGFloat SCREENWIDE_KEYCAP_PADDING = 4.0;
static const CGFloat SCREENWIDE_KEYCAP_GAP = 4.0;
static const CGFloat SCREENWIDE_KEYCAP_RADIUS = 4.0;
static const CGFloat SCREENWIDE_KEYCAP_FONT_SIZE = 13.0;
static const CGFloat SCREENWIDE_KEYCAP_FILL_ALPHA = 0.10;
static const CGFloat SCREENWIDE_KEYCAP_TEXT_ALPHA = 0.85;

static NSColor *keycap_fill_color(BOOL light) {
  // Resolve bg-fill over the Storybook window backing (#FFFFFF / #252525).
  // One opaque shape keeps its antialiased edge and transparent key gaps clean.
  CGFloat backing = light ? 1.0 : 37.0 / 255.0;
  CGFloat fill = light ? 0.0 : 1.0;
  CGFloat value = backing * (1.0 - SCREENWIDE_KEYCAP_FILL_ALPHA)
      + fill * SCREENWIDE_KEYCAP_FILL_ALPHA;
  return [NSColor colorWithSRGBRed:value green:value blue:value alpha:1.0];
}

static NSColor *keycap_text_color(BOOL light) {
  CGFloat value = light ? 0.0 : 1.0;
  return [NSColor colorWithSRGBRed:value green:value blue:value
                              alpha:SCREENWIDE_KEYCAP_TEXT_ALPHA];
}

static BOOL modifier_icon(uint16_t code) {
  return code == 54 || code == 55 || code == 56 || code == 60 ||
         code == 59 || code == 62;
}

// Lucide Command, ArrowBigUp and ChevronUp geometry from the installed
// lucide-react icons, in their 24-unit SVG coordinate system.
static void draw_modifier_icon(uint16_t code, NSPoint point, CGFloat size) {
  NSBezierPath *path = [NSBezierPath bezierPath];
  if (code == 54 || code == 55) {
    [path moveToPoint:NSMakePoint(15, 6)];
    [path lineToPoint:NSMakePoint(15, 18)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(18, 18) radius:3
        startAngle:180 endAngle:-90 clockwise:YES];
    [path lineToPoint:NSMakePoint(6, 15)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(6, 18) radius:3
        startAngle:-90 endAngle:-360 clockwise:YES];
    [path lineToPoint:NSMakePoint(9, 6)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(6, 6) radius:3
        startAngle:0 endAngle:-270 clockwise:YES];
    [path lineToPoint:NSMakePoint(18, 9)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(18, 6) radius:3
        startAngle:90 endAngle:-180 clockwise:YES];
  } else if (code == 56 || code == 60) {
    [path moveToPoint:NSMakePoint(9, 19)];
    [path curveToPoint:NSMakePoint(10, 20) controlPoint1:NSMakePoint(9, 19.552285)
        controlPoint2:NSMakePoint(9.447715, 20)];
    [path lineToPoint:NSMakePoint(14, 20)];
    [path curveToPoint:NSMakePoint(15, 19) controlPoint1:NSMakePoint(14.552285, 20)
        controlPoint2:NSMakePoint(15, 19.552285)];
    [path lineToPoint:NSMakePoint(15, 13)];
    [path curveToPoint:NSMakePoint(16, 12) controlPoint1:NSMakePoint(15, 12.447715)
        controlPoint2:NSMakePoint(15.447715, 12)];
    [path lineToPoint:NSMakePoint(19.293, 12)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(19.293, 11.293) radius:0.707
        startAngle:90 endAngle:-45 clockwise:YES];
    [path lineToPoint:NSMakePoint(12.707, 3.707)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(12, 4.414) radius:1
        startAngle:-45 endAngle:-135 clockwise:YES];
    [path lineToPoint:NSMakePoint(4.207, 10.793)];
    [path appendBezierPathWithArcWithCenter:NSMakePoint(4.707, 11.293) radius:0.707
        startAngle:-135 endAngle:-270 clockwise:YES];
    [path lineToPoint:NSMakePoint(8, 12)];
    [path curveToPoint:NSMakePoint(9, 13) controlPoint1:NSMakePoint(8.552285, 12)
        controlPoint2:NSMakePoint(9, 12.447715)];
    [path closePath];
  } else {
    [path moveToPoint:NSMakePoint(18, 15)];
    [path lineToPoint:NSMakePoint(12, 9)];
    [path lineToPoint:NSMakePoint(6, 15)];
  }
  NSAffineTransform *transform = [NSAffineTransform transform];
  [transform translateXBy:point.x yBy:point.y + size];
  [transform scaleXBy:size / 24.0 yBy:-size / 24.0];
  [path transformUsingAffineTransform:transform];
  path.lineWidth = 2.0 * size / 24.0;
  path.lineCapStyle = NSLineCapStyleRound;
  path.lineJoinStyle = NSLineJoinStyleRound;
  [path stroke];
}

static NSFont *keycap_font(void) {
  const CGFloat size = SCREENWIDE_KEYCAP_FONT_SIZE;
  NSFont *base = [NSFont fontWithName:@"Inter" size:size];
  if (base == nil) return [NSFont systemFontOfSize:size weight:NSFontWeightRegular];
  NSDictionary *traits = @{
    NSFontTraitsAttribute : @{NSFontWeightTrait : @(NSFontWeightRegular)},
    NSFontFeatureSettingsAttribute : @[ @{
      NSFontFeatureTypeIdentifierKey : @(kNumberSpacingType),
      NSFontFeatureSelectorIdentifierKey : @(kMonospacedNumbersSelector),
    } ],
  };
  NSFontDescriptor *descriptor = [base.fontDescriptor fontDescriptorByAddingAttributes:traits];
  return [NSFont fontWithDescriptor:descriptor size:size] ?: base;
}
