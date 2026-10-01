// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// The modern system wallpapers are movies rather than pictures: the folder
// under `.wallpapers` holds a `.mov` and nothing a picker can show. A still
// off the front of the movie is what a tile and a canvas both want, so the
// frame is copied once through AVFoundation and written as a PNG the rest of
// the pipeline reads like any other picture.

#import <AVFoundation/AVFoundation.h>
#import <CoreGraphics/CoreGraphics.h>
#import <Foundation/Foundation.h>
#import <ImageIO/ImageIO.h>
#import <UniformTypeIdentifiers/UniformTypeIdentifiers.h>

// A wallpaper still is a background, not an export: this is the largest edge
// a canvas ever samples one at, and it keeps a 4K movie frame from becoming a
// thirty megabyte PNG on the way to a swatch.
static const CGFloat kScreenwideStillMaximumEdge = 1024.0;

/// The frame as plain 8-bit sRGB.
///
/// A wallpaper movie is graded wide, so the frame comes off the reader as
/// 16-bit and in its own colour space, which a PNG carries at four times the
/// size for detail a swatch cannot show. Drawing it once into an sRGB bitmap
/// is also what keeps the written file to what every decoder downstream
/// already reads. NULL leaves the caller with the frame it had.
static CGImageRef ScreenwideStillAsDisplayImage(CGImageRef frame) CF_RETURNS_RETAINED {
  size_t width = CGImageGetWidth(frame);
  size_t height = CGImageGetHeight(frame);
  if (width == 0 || height == 0) {
    return NULL;
  }
  CGColorSpaceRef space = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGContextRef context =
      CGBitmapContextCreate(NULL, width, height, 8, width * 4, space,
                            kCGImageAlphaPremultipliedLast | kCGBitmapByteOrder32Big);
  CGColorSpaceRelease(space);
  if (context == NULL) {
    return NULL;
  }
  CGContextDrawImage(context, CGRectMake(0, 0, (CGFloat)width, (CGFloat)height), frame);
  CGImageRef flattened = CGBitmapContextCreateImage(context);
  CGContextRelease(context);
  return flattened;
}

/// Writes the first frame of the movie at `video_path` to `destination_path`
/// as a PNG. Answers 1 when the file is there afterwards, 0 otherwise.
int screenwide_video_still_png(const char *video_path, const char *destination_path) {
  if (video_path == NULL || destination_path == NULL) {
    return 0;
  }
  @autoreleasepool {
    NSString *source = [NSString stringWithUTF8String:video_path];
    NSString *destination = [NSString stringWithUTF8String:destination_path];
    if (source == nil || destination == nil) {
      return 0;
    }
    AVURLAsset *asset = [AVURLAsset URLAssetWithURL:[NSURL fileURLWithPath:source] options:nil];
    if (asset == nil) {
      return 0;
    }
    AVAssetImageGenerator *generator = [AVAssetImageGenerator assetImageGeneratorWithAsset:asset];
    generator.appliesPreferredTrackTransform = YES;
    generator.maximumSize = CGSizeMake(kScreenwideStillMaximumEdge, kScreenwideStillMaximumEdge);
    // The nearest keyframe is the point: an exact frame would make the reader
    // decode forward from one, and a wallpaper's opening frame is a
    // background either way.
    generator.requestedTimeToleranceBefore = kCMTimePositiveInfinity;
    generator.requestedTimeToleranceAfter = kCMTimePositiveInfinity;

    NSError *error = nil;
// The asynchronous replacement arrived in macOS 15, and this runs on a
// blocking task that has nothing else to do while a single frame is read.
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
    CGImageRef frame = [generator copyCGImageAtTime:kCMTimeZero actualTime:NULL error:&error];
#pragma clang diagnostic pop
    if (frame == NULL) {
      return 0;
    }
    CGImageRef flattened = ScreenwideStillAsDisplayImage(frame);
    if (flattened != NULL) {
      CGImageRelease(frame);
      frame = flattened;
    }
    NSURL *file = [NSURL fileURLWithPath:destination];
    CGImageDestinationRef writer = CGImageDestinationCreateWithURL(
        (__bridge CFURLRef)file, (__bridge CFStringRef)UTTypePNG.identifier, 1, NULL);
    if (writer == NULL) {
      CGImageRelease(frame);
      return 0;
    }
    CGImageDestinationAddImage(writer, frame, NULL);
    bool written = CGImageDestinationFinalize(writer);
    CFRelease(writer);
    CGImageRelease(frame);
    return written ? 1 : 0;
  }
}
