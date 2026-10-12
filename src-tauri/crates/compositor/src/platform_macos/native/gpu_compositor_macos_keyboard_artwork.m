// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "gpu_compositor_macos_keyboard.h"
#import "gpu_compositor_macos_keyboard_artwork_helpers.h"

static void register_inter_font(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSURL *url = [[NSBundle mainBundle]
        URLForResource:@"Inter-VariableFont_opsz,wght"
         withExtension:@"ttf"
          subdirectory:@"fonts"];
    if (url != nil)
      CTFontManagerRegisterFontsForURL((__bridge CFURLRef)url,
                                      kCTFontManagerScopeProcess, NULL);
  });
}

static NSString *key_label(uint16_t code) {
  static NSDictionary<NSNumber *, NSString *> *labels;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    labels = @{
      @0:@"A", @1:@"S", @2:@"D", @3:@"F", @4:@"H", @5:@"G",
      @6:@"Z", @7:@"X", @8:@"C", @9:@"V", @11:@"B", @12:@"Q",
      @13:@"W", @14:@"E", @15:@"R", @16:@"Y", @17:@"T", @18:@"1",
      @19:@"2", @20:@"3", @21:@"4", @22:@"6", @23:@"5", @24:@"=",
      @25:@"9", @26:@"7", @27:@"−", @28:@"8", @29:@"0", @30:@"]",
      @31:@"O", @32:@"U", @33:@"[", @34:@"I", @35:@"P", @36:@"↩",
      @37:@"L", @38:@"J", @39:@"'", @40:@"K", @41:@";", @42:@"\\",
      @43:@",", @44:@"/", @45:@"N", @46:@"M", @47:@".", @48:@"⇥",
      @49:@"Space", @50:@"`", @51:@"⌫", @53:@"Esc", @57:@"⇪", @65:@".",
      @54:@"⌘", @55:@"⌘", @56:@"⇧", @58:@"⌥", @59:@"⌃", @60:@"⇧",
      @61:@"⌥", @62:@"⌃", @63:@"fn",
      @67:@"*", @69:@"+", @71:@"Clear", @75:@"/", @76:@"⌅", @78:@"−",
      @81:@"=", @82:@"0", @83:@"1", @84:@"2", @85:@"3", @86:@"4",
      @87:@"5", @88:@"6", @89:@"7", @91:@"8", @92:@"9", @96:@"F5",
      @97:@"F6", @98:@"F7", @99:@"F3", @100:@"F8", @101:@"F9",
      @103:@"F11", @105:@"F13", @106:@"F16", @107:@"F14", @109:@"F10",
      @111:@"F12", @113:@"F15", @114:@"Help", @115:@"Home", @116:@"Page ↑",
      @117:@"⌦", @118:@"F4", @119:@"End", @120:@"F2", @121:@"Page ↓",
      @122:@"F1", @123:@"←", @124:@"→", @125:@"↓", @126:@"↑",
    };
  });
  return labels[@(code)] ?: [NSString stringWithFormat:@"Key %u", code];
}

/// The strip for `codes`, in the light or dark appearance, at `backingScale`
/// pixels per design point: premultiplied RGBA rows, top row first, and
/// where each key cap starts and how wide it is. The pixels are `malloc`ed
/// and released by `screenwide_keyboard_raster_free`. Returns 0 when nothing
/// could be drawn.
int screenwide_keyboard_raster(const uint16_t *codes, uint32_t count, uint32_t light,
                               double backingScale, ScreenwideKeyboardRaster *out) {
  if (codes == NULL || out == NULL || count == 0 || count > SCREENWIDE_KEYBOARD_MAX_KEYS)
    return 0;
  *out = (ScreenwideKeyboardRaster){0};
  @autoreleasepool {
    // Match the React Keyboard's keycap variant from
    // `src/components/base/keyboard/keyboard.tsx`: `h-5 min-w-5 gap-tight
    // rounded-sm bg-fill px-control font-sans text-body text-content-fg
    // tabular-nums`, i.e. Inter regular at 13/16 with tabular figures and no
    // letter spacing, inside a 20pt-tall cap with 4pt padding and a 4pt radius.
    register_inter_font();
    NSDictionary *attributes = @{
      NSFontAttributeName: keycap_font(),
      NSForegroundColorAttributeName: keycap_text_color(light != 0),
    };
    const CGFloat height = SCREENWIDE_KEYCAP_HEIGHT;
    const CGFloat inset = SCREENWIDE_KEYCAP_PADDING;
    const CGFloat gap = SCREENWIDE_KEYCAP_GAP;
    CGFloat widths[SCREENWIDE_KEYBOARD_MAX_KEYS] = {0};
    CGFloat width = 0.0;
    for (uint32_t index = 0; index < count; ++index) {
      CGFloat contentWidth = modifier_icon(codes[index])
          ? 12.0 : [key_label(codes[index]) sizeWithAttributes:attributes].width;
      widths[index] = MAX(SCREENWIDE_KEYCAP_MINIMUM_WIDTH, ceil(contentWidth) + inset * 2.0);
      width += widths[index];
    }
    width += gap * (count - 1);
    size_t pixelWidth = MAX((size_t)ceil(width * backingScale), (size_t)1);
    size_t pixelHeight = MAX((size_t)ceil(height * backingScale), (size_t)1);
    uint8_t *pixels = calloc(pixelWidth * pixelHeight, 4);
    if (pixels == NULL) return 0;
    CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGContextRef context = CGBitmapContextCreate(
        pixels, pixelWidth, pixelHeight, 8, pixelWidth * 4, space,
        (CGBitmapInfo)kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
    CGColorSpaceRelease(space);
    if (context == NULL) {
      free(pixels);
      return 0;
    }
    // Grayscale antialiasing, as the OSC text rasteriser and the frontend use.
    CGContextSetShouldSmoothFonts(context, false);
    CGContextScaleCTM(context, backingScale, backingScale);
    NSGraphicsContext *graphics = [NSGraphicsContext graphicsContextWithCGContext:context
                                                                          flipped:NO];
    [NSGraphicsContext saveGraphicsState];
    [NSGraphicsContext setCurrentContext:graphics];
    CGFloat x = 0.0;
    NSColor *background = keycap_fill_color(light != 0);
    for (uint32_t index = 0; index < count; ++index) {
      CGFloat keyWidth = widths[index];
      NSRect rect = NSMakeRect(x, 0.0, keyWidth, height);
      [background setFill];
      [[NSBezierPath bezierPathWithRoundedRect:rect
                                       xRadius:SCREENWIDE_KEYCAP_RADIUS
                                       yRadius:SCREENWIDE_KEYCAP_RADIUS] fill];
      if (modifier_icon(codes[index])) {
        [keycap_text_color(light != 0) setStroke];
        draw_modifier_icon(codes[index],
                           NSMakePoint(x + (keyWidth - 12.0) * 0.5, (height - 12.0) * 0.5),
                           12.0);
      } else {
        NSString *label = key_label(codes[index]);
        NSSize text = [label sizeWithAttributes:attributes];
        [label drawAtPoint:NSMakePoint(x + (keyWidth - text.width) / 2.0,
                                      (height - text.height) / 2.0)
            withAttributes:attributes];
      }
      x += keyWidth + gap;
    }
    [NSGraphicsContext restoreGraphicsState];
    CGContextRelease(context);
    out->pixels = pixels;
    out->width = (uint32_t)pixelWidth;
    out->height = (uint32_t)pixelHeight;
    out->key_count = count;
    CGFloat keyX = 0.0;
    for (uint32_t index = 0; index < count; ++index) {
      out->key_x[index] = (uint32_t)llround(keyX * backingScale);
      out->key_width[index] = (uint32_t)llround(widths[index] * backingScale);
      keyX += widths[index] + gap;
    }
    return 1;
  }
}

void screenwide_keyboard_raster_free(ScreenwideKeyboardRaster *raster) {
  if (raster == NULL) return;
  free(raster->pixels);
  raster->pixels = NULL;
}

