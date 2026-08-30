// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

int screenwide_region_osc_set_snapshot(void *view_ptr, uint32_t display_id,
                                       const uint8_t *rgba, size_t length,
                                       uint32_t width, uint32_t height) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root || !rgba || width == 0 || height == 0 ||
      length != (size_t)width * (size_t)height * 4)
    return 0;
  ScreenwideRegionOSC *target = nil;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root))
    if (surface.displayID == display_id) {
      target = surface;
      break;
    }
  if (!target)
    return 0;

  NSData *pixels = [NSData dataWithBytes:rgba length:length];
  CGDataProviderRef provider =
      CGDataProviderCreateWithCFData((__bridge CFDataRef)pixels);
  CGColorSpaceRef colorSpace = CGColorSpaceCreateWithName(kCGColorSpaceSRGB);
  CGImageRef image = CGImageCreate(
      width, height, 8, 32, (size_t)width * 4, colorSpace,
      kCGBitmapByteOrder32Big | kCGImageAlphaLast, provider, NULL, false,
      kCGRenderingIntentDefault);
  CGColorSpaceRelease(colorSpace);
  CGDataProviderRelease(provider);
  if (!image)
    return 0;
  target.snapshotLayer.contents = (__bridge id)image;
  target.snapshotLayer.hidden = NO;
  CGImageRelease(image);
  return 1;
}

void screenwide_region_osc_set_snapshot_presented(void *view_ptr,
                                                   int presented) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root)
    return;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root))
    surface.snapshotLayer.hidden =
        !presented || surface.snapshotLayer.contents == nil;
}
