// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stdint.h>

/// The most keys one shortcut strip holds; the twin of Rust's `MAX_KEYS`.
#define SCREENWIDE_KEYBOARD_MAX_KEYS 8

/// One rasterised shortcut strip; the twin of Rust's `NativeKeyboardRaster`.
typedef struct {
  uint8_t *pixels;
  uint32_t width;
  uint32_t height;
  uint32_t key_count;
  uint32_t key_x[SCREENWIDE_KEYBOARD_MAX_KEYS];
  uint32_t key_width[SCREENWIDE_KEYBOARD_MAX_KEYS];
} ScreenwideKeyboardRaster;

int screenwide_keyboard_raster(const uint16_t *codes, uint32_t count, uint32_t light,
                               double backing_scale, ScreenwideKeyboardRaster *out);
void screenwide_keyboard_raster_free(ScreenwideKeyboardRaster *raster);
