// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The `image` crate decodes JPEG and PNG but not HEIC, which is what the
// system stores its own desktop pictures as. ImageIO already has a decoder
// for every format the system knows, so the picture is read through that and
// handed back as the same RGBA8 buffer the rest of the pipeline works in.

#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <stdlib.h>

// Anything past this is a decode nobody asked for: the callers cap what they
// need well below it, and a malformed header must not turn into a gigabyte.
static const size_t kScreenwideMaximumEdge = 16384;

/// The picture at `path`, scaled so its longer edge is at most
/// `max_pixel_size`, as tightly packed RGBA8. The caller owns the buffer and
/// frees it with `screenwide_free_decoded_image`. NULL when the file cannot be
/// read or decoded.
uint8_t *screenwide_decode_image_rgba(const char *path,
                                      uint32_t max_pixel_size,
                                      uint32_t *out_width,
                                      uint32_t *out_height) {
  if (path == NULL || out_width == NULL || out_height == NULL || max_pixel_size == 0) {
    return NULL;
  }
  @autoreleasepool {
    NSString *file = [NSString stringWithUTF8String:path];
    if (file == nil) {
      return NULL;
    }
    NSURL *url = [NSURL fileURLWithPath:file];
    CGImageSourceRef source = CGImageSourceCreateWithURL((__bridge CFURLRef)url, NULL);
    if (source == NULL) {
      return NULL;
    }
    // The thumbnail call is the scaled decode: ImageIO reads only as much of
    // the picture as the cap needs, so a twenty megabyte desktop picture does
    // not become a full size buffer on the way to a swatch.
    NSDictionary *options = @{
      (id)kCGImageSourceCreateThumbnailFromImageAlways : @YES,
      (id)kCGImageSourceCreateThumbnailWithTransform : @YES,
      (id)kCGImageSourceThumbnailMaxPixelSize : @(max_pixel_size),
    };
    CGImageRef image = CGImageSourceCreateThumbnailAtIndex(source, 0, (__bridge CFDictionaryRef)options);
    CFRelease(source);
    if (image == NULL) {
      return NULL;
    }
    size_t width = CGImageGetWidth(image);
    size_t height = CGImageGetHeight(image);
    if (width == 0 || height == 0 || width > kScreenwideMaximumEdge ||
        height > kScreenwideMaximumEdge) {
      CGImageRelease(image);
      return NULL;
    }
    size_t stride = width * 4;
    uint8_t *pixels = calloc(stride * height, 1);
    if (pixels == NULL) {
      CGImageRelease(image);
      return NULL;
    }
    CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
    CGContextRef context =
        CGBitmapContextCreate(pixels, width, height, 8, stride, space,
                              kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
    CGColorSpaceRelease(space);
    if (context == NULL) {
      free(pixels);
      CGImageRelease(image);
      return NULL;
    }
    CGContextDrawImage(context, CGRectMake(0, 0, (CGFloat)width, (CGFloat)height), image);
    CGContextRelease(context);
    CGImageRelease(image);
    *out_width = (uint32_t)width;
    *out_height = (uint32_t)height;
    return pixels;
  }
}

void screenwide_free_decoded_image(uint8_t *pixels) {
  free(pixels);
}
