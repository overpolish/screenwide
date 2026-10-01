// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <Metal/Metal.h>
#import <QuartzCore/CAMetalLayer.h>

/// Shared retained native material host for OSC controls and status surfaces.
/// Callers move the view freely and redraw only its small cached content layer.
@interface ScreenwideOscMaterialSurfaceView : NSVisualEffectView
@property(nonatomic, strong) NSView *contentView;
@property(nonatomic, strong) CAMetalLayer *contentLayer;
@property(nonatomic) uint64_t visualKey;
@end

ScreenwideOscMaterialSurfaceView *
screenwide_osc_material_surface(id<MTLDevice> device);
