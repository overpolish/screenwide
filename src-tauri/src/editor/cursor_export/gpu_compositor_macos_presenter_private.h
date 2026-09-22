// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <CoreVideo/CoreVideo.h>
#import <Metal/Metal.h>

#import "gpu_compositor_macos_keyboard.h"
#import "gpu_compositor_macos_presenter.h"

@class ScreenwideCursorResources;

@interface ScreenwideStillPresenter : NSObject
@property(nonatomic, strong) id<MTLDevice> device;
@property(nonatomic, strong) id<MTLCommandQueue> queue;
@property(nonatomic, strong) id<MTLComputePipelineState> pipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> unpackPipeline;
@property(nonatomic, strong) id<MTLBuffer> source;
@property(nonatomic, strong) id<MTLBuffer> camera;
@property(nonatomic) CVMetalTextureCacheRef textureCache;
@property(nonatomic) uint64_t sourceToken;
@property(nonatomic) uint32_t sourceWidth;
@property(nonatomic) uint32_t sourceHeight;
@property(nonatomic) uint64_t cameraToken;
@property(nonatomic) uint32_t cameraWidth;
@property(nonatomic) uint32_t cameraHeight;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, id<MTLBuffer>> *workspaceSources;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, id<MTLBuffer>> *workspaceCameraSources;
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, NSValue *> *workspaceSourceSizes;
@property(nonatomic, strong) NSMutableArray<NSValue *> *workspaceLayers;
@property(nonatomic, strong) NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *keyboardArtworks;
@property(nonatomic, strong) NSArray<NSValue *> *workspaceResizeLayers;
@property(nonatomic) BOOL workspaceResizeApplied;
/// Copies of the retained layers' annotation lists, which the layers point into.
@property(nonatomic, strong) NSMutableArray<NSMutableData *> *workspaceAnnotationStores;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceClearPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceLayerPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> regionMagnifierPipeline;
@property(nonatomic, strong) ScreenwideCursorResources *cursorResources;
@end

/// Copies one layer's annotations into a store the presenter keeps, returning
/// the view the retained layer holds instead of the caller's.
ScreenwideAnnotations screenwide_presenter_retain_annotations(
    ScreenwideStillPresenter *presenter, const ScreenwideAnnotations *source);
/// Releases the stores no retained layer points into any more.
void screenwide_presenter_prune_annotation_stores(ScreenwideStillPresenter *presenter);
