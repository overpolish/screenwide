// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import "../gpu_compositor_macos_keyboard_shader_source.h"
#import "gpu_compositor_macos_shader_source_types.h"
#import "gpu_compositor_macos_shader_source_annotation_curve.h"
#import "gpu_compositor_macos_shader_source_annotations.h"
#import "gpu_compositor_macos_shader_source_annotation_counter.h"
#import "gpu_compositor_macos_shader_source_annotation_text.h"
#import "gpu_compositor_macos_shader_source_annotation_shape.h"
#import "gpu_compositor_macos_shader_source_annotation_draw.h"
#import "gpu_compositor_macos_shader_source_annotation_cursor.h"
#import "gpu_compositor_macos_shader_source_annotation_magnify.h"
#import "gpu_compositor_macos_shader_source_annotation_composite.h"
#import "gpu_compositor_macos_shader_source_annotation_highlight_ink.h"
#import "gpu_compositor_macos_shader_source_annotation_highlight.h"
#import "gpu_compositor_macos_shader_source_annotation_spotlight.h"
#import "gpu_compositor_macos_shader_source_annotation_layers.h"
#import "gpu_compositor_macos_shader_source_annotation_layers_video.h"
#import "gpu_compositor_macos_shader_source_background.h"
#import "gpu_compositor_macos_shader_source_composition.h"
#import "gpu_compositor_macos_shader_source_cursor.h"
#import "gpu_compositor_macos_shader_source_still.h"
#import "gpu_compositor_macos_shader_source_canvas_kernels.h"
#import "gpu_compositor_macos_shader_source_workspace_kernels.h"
#import "gpu_compositor_macos_shader_source_preview_kernels.h"
#import "gpu_compositor_macos_shader_source_camera_kernels.h"
#import "gpu_compositor_macos_shader_source_cursor_kernels.h"
#import "gpu_compositor_macos_shader_source_cursor_overlay_kernels.h"
#import "gpu_compositor_macos_shader_source_redact.h"
#import "gpu_compositor_macos_shader_source_redact_paint.h"
#import "gpu_compositor_macos_shader_source_redact_video.h"

/// The compositor's Metal library, assembled from its parts. An annotation
/// shape brings a source of its own - the arrow's helpers live in
/// `..._annotations.h`, the counter's in `..._annotation_counter.h`, the text
/// `..._annotation_shape.h`, the draw tool's in `..._annotation_draw.h`, the
/// magnifier's in `..._annotation_magnify.h` - and
/// `..._annotation_composite.h` is the one function that branches over
/// `AnnotationUniforms.kind`. A shape's source goes between the two. A
/// redaction is the exception: `..._redact.h` applies it to the source before
/// any canvas pass, and the composite pass only draws its hover halo. A
/// highlight recolours the pixel under it, in `..._annotation_highlight_ink.h`,
/// over the bands `..._annotation_highlight.h` lays, and the spotlights share
/// one shade, in `..._annotation_spotlight.h`, whose blur rides with the
/// redactions. The cursor as the layers see it, blurred by the spotlights and
/// enlarged by a loupe, is in `..._annotation_cursor.h`.
/// `..._annotation_layers.h` walks the document over all of them - what acts
/// on the picture under every mark, and the cursor over the marks, shaded and
/// hidden as if it lay on the picture - and every editor kernel draws through
/// it.
__attribute__((visibility("hidden"))) NSString *const shader_source =
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_TYPES
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURVE
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATIONS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COUNTER
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_TEXT
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SHAPE
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_DRAW
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURSOR
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_MAGNIFY
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_COMPOSITE
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT_INK
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_HIGHLIGHT
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_SPOTLIGHT
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_BACKGROUND
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_COMPOSITION
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_STILL
    SCREENWIDE_KEYBOARD_SHADER_SOURCE
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CANVAS_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_WORKSPACE_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_PREVIEW_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CAMERA_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR_OVERLAY_KERNELS
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT_PAINT
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_REDACT_VIDEO
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_LAYERS_VIDEO;
