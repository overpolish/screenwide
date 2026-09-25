// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#import <CoreVideo/CoreVideo.h>
#import <Metal/Metal.h>

#import "gpu_compositor_macos_keyboard.h"
#import "gpu_compositor_macos_presenter.h"

@class ScreenwideCursorResources;
@class ScreenwideRedactPipelines;

/// A retained source with its redactions applied: the copy every pass samples
/// in place of `original`, remade only when the original or the redactions
/// change. `pending` until a drawing command buffer has written it.
@interface ScreenwideRedactedSource : NSObject
@property(nonatomic, strong) id<MTLBuffer> original;
@property(nonatomic, strong) NSData *redactions;
@property(nonatomic, strong) id<MTLBuffer> pixels;
@property(nonatomic) BOOL pending;
@end

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
/// The redacted copies of `workspaceSources`, for the tokens whose layers
/// carry redactions.
@property(nonatomic, strong) NSMutableDictionary<NSNumber *, ScreenwideRedactedSource *> *workspaceRedactedSources;
@property(nonatomic, strong) NSMutableArray<NSValue *> *workspaceLayers;
@property(nonatomic, strong) NSMutableDictionary<NSString *, ScreenwideKeyboardArtwork *> *keyboardArtworks;
@property(nonatomic, strong) NSArray<NSValue *> *workspaceResizeLayers;
@property(nonatomic) BOOL workspaceResizeApplied;
/// Copies of the retained layers' annotation lists, which the layers point into.
@property(nonatomic, strong) NSMutableArray<NSMutableData *> *workspaceAnnotationStores;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceClearPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> workspaceLayerPipeline;
@property(nonatomic, strong) id<MTLComputePipelineState> regionMagnifierPipeline;
@property(nonatomic, strong) ScreenwideRedactPipelines *redactPipelines;
@property(nonatomic, strong) ScreenwideCursorResources *cursorResources;
@end

/// Copies one layer's annotations into a store the presenter keeps, returning
/// the view the retained layer holds instead of the caller's.
ScreenwideAnnotations screenwide_presenter_retain_annotations(
    ScreenwideStillPresenter *presenter, const ScreenwideAnnotations *source);
/// Releases the stores no retained layer points into any more.
void screenwide_presenter_prune_annotation_stores(ScreenwideStillPresenter *presenter);

/// Notes that `layer`'s redactions are due on its retained `source`, where
/// they changed. A layer without redactions drops its redacted copy.
void screenwide_presenter_redact_source(ScreenwideStillPresenter *presenter,
                                        id<MTLBuffer> source,
                                        const ScreenwideWorkspaceLayer *layer);
/// Writes every redacted copy that is due, at the head of `command`, which
/// must be the command buffer that goes on to sample them.
void screenwide_presenter_apply_redactions(ScreenwideStillPresenter *presenter,
                                           id<MTLCommandBuffer> command);
/// The buffer every pass samples for `token`: its redacted copy where it has
/// one, and the retained source otherwise. A draw applies the due redactions
/// before it samples this.
id<MTLBuffer> screenwide_presenter_visible_source(ScreenwideStillPresenter *presenter,
                                                  uint64_t token);
