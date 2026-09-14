// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once
#import "gpu_compositor_macos_keyboard_shader_source.h"
#import "gpu_compositor_macos_shader_source_types.h"
#import "gpu_compositor_macos_shader_source_annotation_curve.h"
#import "gpu_compositor_macos_shader_source_annotations.h"
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

/// The compositor's Metal library, assembled from its parts. Screenshot
/// annotation tools extend `..._shader_source_annotations.h`: one shape per
/// `AnnotationUniforms.kind`, drawn by both canvas kernels.
__attribute__((visibility("hidden"))) NSString *const shader_source =
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_TYPES
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATION_CURVE
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_ANNOTATIONS
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
    GPU_COMPOSITOR_MACOS_SHADER_SOURCE_CURSOR_OVERLAY_KERNELS;
