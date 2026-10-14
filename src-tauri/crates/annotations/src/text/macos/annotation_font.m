// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>

#import "annotation_text_box.h"

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

/// The weight annotation type is set at, named and as a variation.
///
/// The bundled Inter is one variable face carrying nine named instances on a
/// `wght` axis, so its semibold has to be asked for by name - a weight
/// *trait* on the descriptor only picks between separate family members, of
/// which there are none, and leaves the type at regular. The variation is
/// the fallback for a system that matched the family but not the instance.
static const uint32_t kScreenwideWeightAxis = 'wght';
static const CGFloat kScreenwideAnnotationWeight = 600.0;

/// Tabular figures, so 1 and 8 take the same room and a counter's number
/// does not shift its weight as it grows.
static NSArray *tabular_figures(void) {
  return @[ @{
    NSFontFeatureTypeIdentifierKey : @(kNumberSpacingType),
    NSFontFeatureSelectorIdentifierKey : @(kMonospacedNumbersSelector),
  } ];
}

NSFont *screenwide_annotation_font(CGFloat size, BOOL tabular) {
  register_inter_font();
  NSArray *features = tabular ? tabular_figures() : @[];
  NSFontDescriptor *named = [NSFontDescriptor fontDescriptorWithFontAttributes:@{
    NSFontFamilyAttribute : @"Inter",
    NSFontFaceAttribute : @"SemiBold",
    NSFontFeatureSettingsAttribute : features,
  }];
  NSFont *font = [NSFont fontWithDescriptor:named size:size];
  if (font != nil &&
      [font.fontName rangeOfString:@"SemiBold"].location != NSNotFound)
    return font;
  NSFont *base = [NSFont fontWithName:@"Inter" size:size];
  if (base == nil)
    return [NSFont systemFontOfSize:size weight:NSFontWeightSemibold];
  NSFontDescriptor *varied = [base.fontDescriptor fontDescriptorByAddingAttributes:@{
    (__bridge NSString *)kCTFontVariationAttribute :
        @{@(kScreenwideWeightAxis) : @(kScreenwideAnnotationWeight)},
    NSFontFeatureSettingsAttribute : features,
  }];
  return [NSFont fontWithDescriptor:varied size:size] ?: base;
}

/// `weight` on the CSS scale as AppKit's weight trait, which runs from -1 to
/// 1 through the named weights' own values.
static CGFloat appkit_weight(uint32_t weight) {
  static const CGFloat steps[] = {-0.8, -0.6, -0.4, 0.0, 0.23, 0.3, 0.4, 0.56, 0.62};
  CGFloat at = MIN(MAX(((CGFloat)weight - 100.0) / 100.0, 0.0), 8.0);
  NSUInteger low = (NSUInteger)floor(at);
  NSUInteger high = MIN(low + 1, (NSUInteger)8);
  return steps[low] + (steps[high] - steps[low]) * (at - (CGFloat)low);
}

/// `font` with its weight axis at `weight`, where it is a variable face: its
/// named members only come at a few weights, and the axis reaches the rest.
static NSFont *with_weight_axis(NSFont *font, uint32_t weight) {
  NSArray *axes = CFBridgingRelease(CTFontCopyVariationAxes((__bridge CTFontRef)font));
  for (NSDictionary *axis in axes) {
    NSNumber *tag = axis[(__bridge NSString *)kCTFontVariationAxisIdentifierKey];
    if (tag.unsignedIntValue != kScreenwideWeightAxis) continue;
    NSFontDescriptor *varied = [font.fontDescriptor fontDescriptorByAddingAttributes:@{
      (__bridge NSString *)kCTFontVariationAttribute : @{@(kScreenwideWeightAxis) : @(weight)},
    }];
    return [NSFont fontWithDescriptor:varied size:font.pointSize] ?: font;
  }
  return font;
}

/// The member of `family` nearest `weight`, upright or italic as asked, as
/// its PostScript name, or nil where the family is not installed. A family's
/// members are separate faces at a few weights, and asking for a weight
/// trait does not choose between them, so each member's own weight is read
/// and the nearest taken; one of the wrong slant only where the family has
/// none of the right one. Kept per family, weight and slant, since every
/// measured line asks.
static NSString *nearest_member(NSString *family, uint32_t weight, BOOL italic) {
  static NSCache<NSString *, NSString *> *chosen;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    chosen = [[NSCache alloc] init];
  });
  NSString *key = [NSString stringWithFormat:@"%@\n%u\n%d", family, weight, italic];
  NSString *known = [chosen objectForKey:key];
  if (known != nil) return known.length > 0 ? known : nil;
  CGFloat wanted = appkit_weight(weight);
  NSString *best = nil;
  CGFloat distance = CGFLOAT_MAX;
  for (NSArray *member in [[NSFontManager sharedFontManager] availableMembersOfFontFamily:family]) {
    NSString *name = member.firstObject;
    NSFont *font = [NSFont fontWithName:name size:12];
    if (font == nil) continue;
    NSDictionary *traits = [font.fontDescriptor objectForKey:NSFontTraitsAttribute];
    CGFloat own = [traits[NSFontWeightTrait] doubleValue];
    BOOL slanted = ([traits[NSFontSymbolicTrait] unsignedIntValue] & NSFontItalicTrait) != 0;
    CGFloat off = fabs(own - wanted) + (slanted != italic ? 10.0 : 0.0);
    if (off < distance) {
      distance = off;
      best = name;
    }
  }
  [chosen setObject:best ?: @"" forKey:key];
  return best;
}

/// Defined in Rust, in `bundled.rs`: the font files the app carries for its
/// caption templates, one at a time, null past the last.
extern const uint8_t *screenwide_bundled_font(uint32_t index, uint32_t *length);

/// Every face in the font files the app carries, with its family, weight
/// trait and slant read once. They are set straight from the binary's own
/// bytes, never registered with the system: Core Text only registers a
/// face it can find again by file.
static NSArray<NSDictionary *> *bundled_faces(void) {
  static NSArray<NSDictionary *> *faces;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSMutableArray<NSDictionary *> *found = [NSMutableArray array];
    uint32_t length = 0;
    for (uint32_t index = 0;; index++) {
      const uint8_t *bytes = screenwide_bundled_font(index, &length);
      if (bytes == NULL) break;
      // The bytes are the binary's own and never freed, so they are read in
      // place rather than copied.
      CFDataRef data = CFDataCreateWithBytesNoCopy(NULL, bytes, length, kCFAllocatorNull);
      NSArray *descriptors = CFBridgingRelease(CTFontManagerCreateFontDescriptorsFromData(data));
      CFRelease(data);
      for (id descriptor in descriptors) {
        CTFontRef font =
            CTFontCreateWithFontDescriptor((__bridge CTFontDescriptorRef)descriptor, 12, NULL);
        NSString *family = CFBridgingRelease(CTFontCopyFamilyName(font));
        NSDictionary *traits = CFBridgingRelease(CTFontCopyTraits(font));
        CFRelease(font);
        uint32_t symbolic = [traits[(__bridge NSString *)kCTFontSymbolicTrait] unsignedIntValue];
        [found addObject:@{
          @"descriptor" : descriptor,
          @"family" : family ?: @"",
          @"weight" : traits[(__bridge NSString *)kCTFontWeightTrait] ?: @0,
          @"italic" : @((symbolic & kCTFontItalicTrait) != 0),
        }];
      }
    }
    faces = found;
  });
  return faces;
}

/// `family` at `weight`, italic where asked and it has one, at the size the
/// cache keeps it at, made from its nearest face and set at the weight
/// exactly where it varies by weight; nil where the app does not carry the
/// family.
static CTFontRef make_bundled_font(NSString *family, uint32_t weight, BOOL italic,
                                   CGFloat size) {
  CGFloat wanted = appkit_weight(weight);
  NSDictionary *best = nil;
  CGFloat distance = CGFLOAT_MAX;
  for (NSDictionary *face in bundled_faces()) {
    if (![face[@"family"] isEqualToString:family]) continue;
    CGFloat off = fabs([face[@"weight"] doubleValue] - wanted) +
                  ([face[@"italic"] boolValue] != italic ? 10.0 : 0.0);
    if (off < distance) {
      distance = off;
      best = face;
    }
  }
  if (best == nil) return NULL;
  CTFontRef font = CTFontCreateWithFontDescriptor(
      (__bridge CTFontDescriptorRef)best[@"descriptor"], size, NULL);
  NSArray *axes = CFBridgingRelease(CTFontCopyVariationAxes(font));
  for (NSDictionary *axis in axes) {
    NSNumber *tag = axis[(__bridge NSString *)kCTFontVariationAxisIdentifierKey];
    if (tag.unsignedIntValue != kScreenwideWeightAxis) continue;
    CTFontDescriptorRef own = CTFontCopyFontDescriptor(font);
    CTFontDescriptorRef varied = CTFontDescriptorCreateCopyWithVariation(
        own, (__bridge CFNumberRef)tag, (CGFloat)weight);
    CTFontRef weighted = CTFontCreateCopyWithAttributes(font, size, NULL, varied);
    CFRelease(varied);
    CFRelease(own);
    if (weighted != NULL) {
      CFRelease(font);
      font = weighted;
    }
    break;
  }
  return font;
}

/// [`make_bundled_font`] at `size`. Making a face from the binary's bytes
/// takes far longer than measuring a word in it, and every measured word
/// asks, so each family, weight and slant is made once and kept, then only
/// copied to the size asked for.
static NSFont *bundled_font(NSString *family, uint32_t weight, BOOL italic, CGFloat size) {
  static NSCache<NSString *, id> *made;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    made = [[NSCache alloc] init];
  });
  NSString *key = [NSString stringWithFormat:@"%@\n%u\n%d", family, weight, italic];
  id known = [made objectForKey:key];
  if (known == nil) {
    CTFontRef font = make_bundled_font(family, weight, italic, 12.0);
    known = font != NULL ? CFBridgingRelease(font) : (id)[NSNull null];
    [made setObject:known forKey:key];
  }
  if (known == [NSNull null]) return nil;
  return CFBridgingRelease(
      CTFontCreateCopyWithAttributes((__bridge CTFontRef)known, size, NULL, NULL));
}

NSFont *screenwide_face_font(NSString *family, uint32_t weight, BOOL italic, CGFloat size) {
  if (family.length == 0) {
    if (weight == (uint32_t)kScreenwideAnnotationWeight && !italic)
      return screenwide_annotation_font(size, NO);
    register_inter_font();
  } else {
    // The app's own copy of a family before the system's, as the font list
    // offers it.
    NSFont *bundled = bundled_font(family, weight, italic, size);
    if (bundled != nil) return bundled;
  }
  NSString *member = nearest_member(family.length > 0 ? family : @"Inter", weight, italic);
  NSFont *font = member != nil ? [NSFont fontWithName:member size:size] : nil;
  if (font == nil) {
    // A family that has gone from this system falls back to Inter, and
    // Inter, where it could not be registered, to the system's face.
    if (family.length > 0) return screenwide_face_font(nil, weight, italic, size);
    return [NSFont systemFontOfSize:size weight:appkit_weight(weight)];
  }
  return with_weight_axis(font, weight);
}

NSFont *screenwide_face_font_bytes(const uint8_t *family, uint32_t length, uint32_t weight,
                                   BOOL italic, CGFloat size) {
  NSString *name = family != NULL && length > 0
                       ? [[NSString alloc] initWithBytes:family
                                                  length:length
                                                encoding:NSUTF8StringEncoding]
                       : nil;
  return screenwide_face_font(name, weight, italic, size);
}
