// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

#import <Metal/Metal.h>

#import "gpu_compositor_macos.h"

/// Caches one decoded background picture under `identifier`. The pixels are
/// tightly packed RGBA owned by the caller and are only read during this
/// call. Registering an identifier that is already cached is a no-op, so a
/// video export does not re-upload its background for every frame. Answers
/// zero when the picture cannot be kept.
int screenwide_gpu_register_background_image(uint32_t identifier,
                                             const uint8_t *rgba,
                                             uint32_t width, uint32_t height);

/// Binds one canvas' uniforms and the background picture they name. The
/// texture slot is always filled, with a transparent 1x1 placeholder when the
/// canvas paints no picture, so every pipeline stays valid. A canvas whose
/// picture is no longer registered is bound with its `has_background_image`
/// cleared, which leaves the mesh or solid colour painting instead: a picture
/// that moved must never break an export.
void screenwide_gpu_bind_canvas(id<MTLComputeCommandEncoder> encoder,
                                id<MTLDevice> device,
                                const ScreenwideCanvas *canvas,
                                NSUInteger buffer_index,
                                NSUInteger texture_index);
