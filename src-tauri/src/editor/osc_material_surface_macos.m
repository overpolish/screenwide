// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_material_surface_macos.h"

@implementation ScreenwideOscMaterialSurfaceView
- (NSView *)hitTest:(NSPoint)point {
  (void)point;
  return nil;
}
@end

ScreenwideOscMaterialSurfaceView *
screenwide_osc_material_surface(id<MTLDevice> device) {
  ScreenwideOscMaterialSurfaceView *surface =
      [[ScreenwideOscMaterialSurfaceView alloc] initWithFrame:NSZeroRect];
  surface.material = NSVisualEffectMaterialUnderWindowBackground;
  surface.blendingMode = NSVisualEffectBlendingModeWithinWindow;
  surface.state = NSVisualEffectStateActive;
  surface.wantsLayer = YES;
  surface.layer.masksToBounds = YES;
  surface.hidden = YES;
  surface.contentView = [[NSView alloc] initWithFrame:NSZeroRect];
  surface.contentView.wantsLayer = YES;
  surface.contentLayer = [CAMetalLayer layer];
  surface.contentLayer.device = device;
  surface.contentLayer.pixelFormat = MTLPixelFormatBGRA8Unorm;
  surface.contentLayer.framebufferOnly = YES;
  surface.contentLayer.opaque = NO;
  surface.contentLayer.presentsWithTransaction = NO;
  NSNull *noAction = [NSNull null];
  surface.contentLayer.actions = @{
    @"bounds": noAction,
    @"position": noAction,
    @"hidden": noAction,
    @"opacity": noAction,
    @"contents": noAction,
  };
  surface.contentView.layer = surface.contentLayer;
  [surface addSubview:surface.contentView positioned:NSWindowAbove relativeTo:nil];
  return surface;
}
