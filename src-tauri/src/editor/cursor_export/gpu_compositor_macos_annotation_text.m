// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <CoreText/CoreText.h>
#include <math.h>

#import "gpu_compositor_macos_annotation_text.h"
#import "gpu_compositor_macos_annotation_types.h"

void screenwide_register_inter_font(void) {
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

/// The weight a counter's number is set at, named and as a variation.
///
/// The bundled Inter is one variable face carrying nine named instances on a
/// `wght` axis, so its semibold has to be asked for by name - a weight
/// *trait* on the descriptor only picks between separate family members, of
/// which there are none, and leaves the number at regular. The variation is
/// the fallback for a system that matched the family but not the instance.
static const uint32_t kScreenwideWeightAxis = 'wght';
static const CGFloat kScreenwideCounterWeight = 600.0;

/// Tabular figures, so 1 and 8 take the same room and a number does not
/// shift its weight as it grows.
static NSArray *counter_figures(void) {
  return @[ @{
    NSFontFeatureTypeIdentifierKey : @(kNumberSpacingType),
    NSFontFeatureSelectorIdentifierKey : @(kMonospacedNumbersSelector),
  } ];
}

/// The face a counter's number is set in.
static NSFont *counter_font(CGFloat size) {
  screenwide_register_inter_font();
  NSFontDescriptor *named = [NSFontDescriptor fontDescriptorWithFontAttributes:@{
    NSFontFamilyAttribute : @"Inter",
    NSFontFaceAttribute : @"SemiBold",
    NSFontFeatureSettingsAttribute : counter_figures(),
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
        @{@(kScreenwideWeightAxis) : @(kScreenwideCounterWeight)},
    NSFontFeatureSettingsAttribute : counter_figures(),
  }];
  return [NSFont fontWithDescriptor:varied size:size] ?: base;
}

static NSDictionary *counter_dress(CGFloat size) {
  // White, because the kernel tints the number to whatever reads on the
  // disc's own colour.
  return @{
    NSFontAttributeName : counter_font(size),
    NSForegroundColorAttributeName : [NSColor whiteColor],
  };
}

/// One number as it will be rasterised: everything in atlas pixels, which are
/// [`SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE`] to the drawn pixel.
typedef struct {
  NSDictionary *dress;
  NSString *text;
  CGSize measured;
  CGSize cell;
} ScreenwideCounterText;

/// The dress and the room one number needs, in atlas pixels. A number too
/// wide for its disc is narrowed rather than allowed to touch the edge: three
/// digits in a small disc still read, and the disc stays the size the style
/// asked for.
static ScreenwideCounterText counter_text(const char *value, uint32_t length, CGFloat size) {
  CGFloat diameter = size * 2.0 * SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE;
  size *= SCREENWIDE_COUNTER_TEXT_SUPERSAMPLE;
  ScreenwideCounterText result = { .text = [[NSString alloc] initWithBytes:value length:length encoding:NSUTF8StringEncoding] };
  result.dress = counter_dress(size);
  result.measured = [result.text sizeWithAttributes:result.dress];
  CGFloat limit = diameter * SCREENWIDE_COUNTER_TEXT_WIDTH_SHARE;
  if (result.measured.width > limit && result.measured.width > 0.0) {
    result.dress = counter_dress(size * limit / result.measured.width);
    result.measured = [result.text sizeWithAttributes:result.dress];
  }
  result.cell = CGSizeMake(ceil(result.measured.width) + 2.0, ceil(result.measured.height) + 2.0);
  return result;
}

// The layout half of the atlas, shared with the D3D11 backend through
// `annotations/counter/atlas_ffi.rs`.
typedef struct {
  const char *text;
  uint32_t length;
  float radius;
} ScreenwideCounterNumber;
typedef struct {
  uint32_t index;
  float radius;
} ScreenwideCounterDraw;
typedef struct {
  uint32_t width, height, fresh, draw_count;
} ScreenwideCounterAtlasLayout;
typedef uint32_t (*ScreenwideCounterMeasure)(void *context, uint32_t index, float radius,
                                             uint32_t *width, uint32_t *height);
void *screenwide_counter_atlas_create(void);
void screenwide_counter_atlas_destroy(void *atlas);
uint32_t screenwide_counter_atlas_frame(
    void *atlas, const ScreenwideCounterNumber *numbers, uint32_t count,
    ScreenwideCounterMeasure measure, void *context, ScreenwideAnnotationTextRect *rects,
    ScreenwideCounterDraw *draws, ScreenwideCounterAtlasLayout *out);

/// One thread's atlas: the Rust layout and the pixels it describes. Every
/// thread that composes keeps its own, so the export and the preview neither
/// share a lock nor evict each other's numbers.
@interface ScreenwideCounterAtlas : NSObject
@property(nonatomic) void *layout;
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLBuffer> pixels;
@end

@implementation ScreenwideCounterAtlas
- (instancetype)initWithDevice:(id<MTLDevice>)device {
  if ((self = [super init])) {
    _layout = screenwide_counter_atlas_create();
    _device = device;
  }
  return self;
}
- (void)dealloc {
  screenwide_counter_atlas_destroy(_layout);
}
@end

static ScreenwideCounterAtlas *thread_atlas(id<MTLDevice> device) {
  static NSString *const key = @"ScreenwideCounterAtlas";
  NSMutableDictionary *store = NSThread.currentThread.threadDictionary;
  ScreenwideCounterAtlas *atlas = store[key];
  if (atlas == nil || atlas.device != device) {
    atlas = [[ScreenwideCounterAtlas alloc] initWithDevice:device];
    store[key] = atlas;
  }
  return atlas;
}

static uint32_t measure_counter(void *context, uint32_t index, float radius,
                                uint32_t *width, uint32_t *height) {
  const ScreenwideCounterNumber *numbers = context;
  ScreenwideCounterText text = counter_text(numbers[index].text, numbers[index].length, radius);
  *width = (uint32_t)text.cell.width;
  *height = (uint32_t)text.cell.height;
  return 1;
}

/// Rasterises one number into its rectangle of the atlas, margin included, so
/// whatever the space held before is overwritten.
static void draw_counter(uint8_t *pixels, uint32_t atlas_width,
                         ScreenwideAnnotationTextRect rect,
                         const ScreenwideCounterNumber *number, float radius) {
  ScreenwideCounterText text = counter_text(number->text, number->length, radius);
  size_t width = (size_t)rect.width;
  size_t height = (size_t)rect.height;
  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context = CGBitmapContextCreate(
      NULL, width, height, 8, width * 4, space,
      (CGBitmapInfo)kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (context == NULL) return;
  CGContextClearRect(context, CGRectMake(0, 0, width, height));
  // Grayscale antialiasing, as the keyboard artwork and the OSC text use: the
  // number is tinted by the kernel, so subpixel coverage would be wrong.
  CGContextSetShouldSmoothFonts(context, false);
  NSGraphicsContext *graphics =
      [NSGraphicsContext graphicsContextWithCGContext:context flipped:NO];
  [NSGraphicsContext saveGraphicsState];
  [NSGraphicsContext setCurrentContext:graphics];
  [text.text drawAtPoint:NSMakePoint((text.cell.width - text.measured.width) * 0.5,
                                     (text.cell.height - text.measured.height) * 0.5)
          withAttributes:text.dress];
  [NSGraphicsContext restoreGraphicsState];
  // A bitmap context keeps its top row first, the top-down space the kernels
  // sample the atlas in.
  const uint8_t *source = CGBitmapContextGetData(context);
  size_t row_bytes = CGBitmapContextGetBytesPerRow(context);
  for (size_t row = 0; row < height; row++)
    memcpy(pixels + (((size_t)rect.y + row) * atlas_width + (size_t)rect.x) * 4,
           source + row * row_bytes, width * 4);
  CGContextRelease(context);
}

id<MTLBuffer> screenwide_annotation_text_atlas(
    id<MTLDevice> device, const char *const *values, const uint32_t *lengths,
    const float *sizes, uint32_t count, ScreenwideAnnotationTextRect *rects,
    ScreenwideAnnotationTextUniforms *uniforms) {
  if (rects != NULL && count > 0)
    memset(rects, 0, sizeof(ScreenwideAnnotationTextRect) * count);
  if (uniforms != NULL) *uniforms = (ScreenwideAnnotationTextUniforms){0};
  if (device == nil || values == NULL || lengths == NULL || sizes == NULL ||
      rects == NULL || count == 0)
    return nil;

  NSMutableData *numbers_data =
      [NSMutableData dataWithLength:sizeof(ScreenwideCounterNumber) * count];
  ScreenwideCounterNumber *numbers = numbers_data.mutableBytes;
  for (uint32_t index = 0; index < count; index++)
    numbers[index] = (ScreenwideCounterNumber){
        values[index], values[index] == NULL ? 0 : lengths[index], sizes[index]};
  NSMutableData *draws_data =
      [NSMutableData dataWithLength:sizeof(ScreenwideCounterDraw) * count];
  ScreenwideCounterDraw *draws = draws_data.mutableBytes;
  ScreenwideCounterAtlasLayout layout = {0};
  ScreenwideCounterAtlas *atlas = thread_atlas(device);
  if (!screenwide_counter_atlas_frame(atlas.layout, numbers, count, measure_counter, numbers,
                                      rects, draws, &layout) ||
      layout.width == 0 || layout.height == 0)
    return nil;
  // A fresh layout gets new storage rather than being drawn over: a command
  // buffer still in flight holds the old one and reads the cells it was given.
  // Otherwise the new cells land in space no earlier cell used.
  if (layout.fresh || atlas.pixels == nil) {
    atlas.pixels = [device newBufferWithLength:(NSUInteger)layout.width * layout.height * 4
                                       options:MTLResourceStorageModeShared];
    if (atlas.pixels == nil) return nil;
  }
  for (uint32_t draw = 0; draw < layout.draw_count; draw++) {
    uint32_t index = draws[draw].index;
    draw_counter(atlas.pixels.contents, layout.width, rects[index], &numbers[index],
                 draws[draw].radius);
  }
  if (uniforms != NULL)
    *uniforms = (ScreenwideAnnotationTextUniforms){layout.width, layout.height};
  return atlas.pixels;
}
