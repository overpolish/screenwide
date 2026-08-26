// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "gpu_compositor_macos_keyboard.h"

@implementation ScreenwideKeyboardArtwork
@end

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

typedef struct {
  uint16_t keyCode;
  ScreenwideKeyboardKey state;
} ScreenwidePreparedKey;

typedef struct {
  uint32_t count;
  ScreenwidePreparedKey keys[SCREENWIDE_KEYBOARD_MAX_KEYS];
} ScreenwidePreparedShortcut;

static BOOL is_modifier_key(uint16_t code) {
  return code == 54 || code == 55 || code == 56 || code == 58 ||
      code == 59 || code == 60 || code == 61 || code == 62 || code == 63;
}

static void append_prepared(ScreenwidePreparedShortcut *result, uint16_t code,
                            ScreenwideKeyboardKey state) {
  if (result->count >= SCREENWIDE_KEYBOARD_MAX_KEYS) return;
  result->keys[result->count++] = (ScreenwidePreparedKey){code, state};
}

static ScreenwidePreparedShortcut prepared_shortcut(
    ScreenwideKeyboardOverlay overlay) {
  ScreenwidePreparedShortcut result = {0};
  uint32_t count = MIN(overlay.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
  for (uint32_t index = 0; index < count; ++index) {
    ScreenwideKeyboardKey state = overlay.keys[index];
    // Version-one sidecars stored a modifier mask on the final key. Expand it
    // here so old recordings keep the grouped KBD appearance.
    if (count == 1 && !is_modifier_key(state.key_code)) {
      const uint16_t modifierCodes[] = {55, 59, 58, 56, 63};
      for (uint32_t bit = 0; bit < 5; ++bit)
        if ((state.modifier_mask & (1u << bit)) != 0)
          append_prepared(&result, modifierCodes[bit], state);
    }
    append_prepared(&result, state.key_code, state);
  }
  return result;
}

static void update_uniforms(ScreenwideKeyboardUniforms *uniforms,
                            ScreenwideKeyboardOverlay overlay,
                            ScreenwidePreparedShortcut prepared) {
  uniforms->key_count = prepared.count;
  uniforms->animation = overlay.animation;
  uniforms->scale = overlay.scale;
  uniforms->layout_progress = overlay.progress;
  uniforms->maximum_width = overlay.maximum_width;
  uniforms->requested_scale = overlay.requested_scale;
  uniforms->center_x = overlay.center_x;
  uniforms->center_y = overlay.center_y;
  for (uint32_t index = 0; index < prepared.count; ++index) {
    ScreenwideKeyboardKey state = prepared.keys[index].state;
    uniforms->keys[index].visible = state.visible;
    uniforms->keys[index].alpha = state.alpha;
    uniforms->keys[index].scale = state.scale;
    uniforms->keys[index].progress = state.progress;
    uniforms->keys[index].layout_progress = state.layout_progress;
    uniforms->keys[index].slot = state.slot;
    uniforms->keys[index].layout_from_mask = state.layout_from_mask;
    uniforms->keys[index].layout_to_mask = state.layout_to_mask;
  }
}

static CGFloat keyboard_backing_scale(uint32_t outputHeight,
                                      ScreenwideKeyboardOverlay overlay) {
  // The spring peaks just below 1.073. Cover its complete scale excursion so
  // animation never enlarges the cached artwork beyond its source pixels.
  const CGFloat maximumAnimatedScale = 1.08;
  const CGFloat designHeight = 20.0;
  CGFloat requestedScale = overlay.requested_scale > 0.0
      ? overlay.requested_scale : overlay.scale;
  CGFloat outputPixels = (CGFloat)outputHeight * (60.0 / 1080.0) *
      MAX(requestedScale, 0.0) * maximumAnimatedScale;
  return MIN(MAX(ceil(outputPixels / designHeight), 12.0), 64.0);
}

ScreenwideKeyboardArtwork *screenwide_keyboard_artwork(
    id<MTLDevice> device,
    NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *cache,
    ScreenwideKeyboardOverlay overlay, uint32_t outputHeight) {
  if (device == nil || cache == nil || overlay.key_count == 0) return nil;
  ScreenwidePreparedShortcut prepared = prepared_shortcut(overlay);
  if (prepared.count == 0) return nil;
  CGFloat backingScale = keyboard_backing_scale(outputHeight, overlay);
  NSMutableString *key =
      [NSMutableString stringWithFormat:@"%u|%.0f|", overlay.appearance,
                                        backingScale];
  NSMutableArray<NSString *> *labels = [NSMutableArray arrayWithCapacity:prepared.count];
  for (uint32_t index = 0; index < prepared.count; ++index) {
    [key appendFormat:@"%u:", prepared.keys[index].keyCode];
    [labels addObject:key_label(prepared.keys[index].keyCode)];
  }
  ScreenwideKeyboardArtwork *known = cache[key];
  if (known != nil) {
    ScreenwideKeyboardUniforms uniforms = known.uniforms;
    update_uniforms(&uniforms, overlay, prepared);
    known.uniforms = uniforms;
    return known;
  }

  // Match the React Keyboard's default variant: text-sm/5, px-1,
  // rounded-sm, tracking-wider, Inter, bg-neutral and text-muted. Artwork is
  // rasterised once at the density its output canvas and animation require;
  // every subsequent frame remains a GPU-only composition.
  register_inter_font();
  NSFont *font = [NSFont fontWithName:@"Inter" size:14.0]
      ?: [NSFont systemFontOfSize:14.0 weight:NSFontWeightRegular];
  BOOL light = overlay.appearance == 1;
  NSDictionary *attributes = @{
    NSFontAttributeName: font,
    NSKernAttributeName: @0.7,
    NSForegroundColorAttributeName:
        light ? [NSColor colorWithSRGBRed:0.251 green:0.251 blue:0.251 alpha:1.0]
              : [NSColor colorWithSRGBRed:0.639 green:0.639 blue:0.639 alpha:1.0],
  };
  const CGFloat height = 20.0, inset = 4.0, gap = 4.0;
  NSMutableArray<NSNumber *> *widths = [NSMutableArray arrayWithCapacity:labels.count];
  CGFloat width = 0.0;
  for (NSString *label in labels) {
    CGFloat keyWidth = ceil([label sizeWithAttributes:attributes].width) + inset * 2.0;
    [widths addObject:@(keyWidth)];
    width += keyWidth;
  }
  width += gap * MAX((NSInteger)labels.count - 1, 0);
  NSUInteger pixelWidth = MAX((NSUInteger)ceil(width * backingScale), 1u);
  NSUInteger pixelHeight = MAX((NSUInteger)ceil(height * backingScale), 1u);
  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, pixelWidth, pixelHeight, 8, pixelWidth * 4, space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (context == NULL) return nil;
  CGContextScaleCTM(context, backingScale, backingScale);
  NSGraphicsContext *graphics = [NSGraphicsContext graphicsContextWithCGContext:context
                                                                        flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  CGFloat x = 0.0;
  NSColor *background = light
      ? [NSColor colorWithSRGBRed:0.898 green:0.898 blue:0.898 alpha:1.0]
      : [NSColor colorWithSRGBRed:0.251 green:0.251 blue:0.251 alpha:1.0];
  for (NSUInteger index = 0; index < labels.count; ++index) {
    CGFloat keyWidth = widths[index].doubleValue;
    NSRect rect = NSMakeRect(x, 0.0, keyWidth, height);
    [background setFill];
    [[NSBezierPath bezierPathWithRoundedRect:rect xRadius:4.0 yRadius:4.0] fill];
    NSString *label = labels[index];
    NSSize text = [label sizeWithAttributes:attributes];
    [label drawAtPoint:NSMakePoint(x + (keyWidth - text.width) / 2.0,
                                  (height - text.height) / 2.0)
        withAttributes:attributes];
    x += keyWidth + gap;
  }
  [NSGraphicsContext restoreGraphicsState];
  id<MTLBuffer> pixels = [device newBufferWithBytes:CGBitmapContextGetData(context)
      length:pixelWidth * pixelHeight * 4 options:MTLResourceStorageModeShared];
  CGContextRelease(context);
  if (pixels == nil) return nil;
  ScreenwideKeyboardArtwork *artwork = [ScreenwideKeyboardArtwork new];
  artwork.pixels = pixels;
  ScreenwideKeyboardUniforms uniforms = {0};
  uniforms.width = (uint32_t)pixelWidth;
  uniforms.height = (uint32_t)pixelHeight;
  CGFloat keyX = 0.0;
  for (uint32_t index = 0; index < prepared.count; ++index) {
    uniforms.keys[index].x = (uint32_t)llround(keyX * backingScale);
    uniforms.keys[index].width =
        (uint32_t)llround(widths[index].doubleValue * backingScale);
    keyX += widths[index].doubleValue + gap;
  }
  update_uniforms(&uniforms, overlay, prepared);
  artwork.uniforms = uniforms;
  const NSUInteger cacheLimit = 64u * 1024u * 1024u;
  __block NSUInteger cacheBytes = 0;
  [cache enumerateKeysAndObjectsUsingBlock:
      ^(__unused NSString *cacheKey, ScreenwideKeyboardArtwork *cached,
        __unused BOOL *stop) { cacheBytes += cached.pixels.length; }];
  if (cache.count >= 64 ||
      (cacheBytes > 0 && cacheBytes + pixels.length > cacheLimit))
    [cache removeAllObjects];
  cache[key] = artwork;
  return artwork;
}
