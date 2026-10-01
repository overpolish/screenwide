// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos_shader_magnifier.h"
#import "osc_gpu_macos_shader_types.h"
#import "osc_gpu_macos_shader_fragment.h"

/// The region OSC's Metal library, assembled from its parts. The preamble and
/// the magnifier kernel come first, then the shared types, then the one
/// fragment stage that draws every kind of OSC geometry.
static NSString *const ScreenwideRegionOscMetalSource =
    SCREENWIDE_REGION_OSC_SHADER_MAGNIFIER
    SCREENWIDE_REGION_OSC_SHADER_TYPES
    SCREENWIDE_REGION_OSC_SHADER_FRAGMENT;
