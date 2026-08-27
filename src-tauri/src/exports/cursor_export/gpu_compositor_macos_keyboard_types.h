// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#pragma once

#include <stddef.h>
#include <stdint.h>

#define SCREENWIDE_KEYBOARD_MAX_KEYS 8

typedef struct {
  uint16_t key_code;
  uint16_t _padding;
  uint32_t modifier_mask;
  uint32_t visible;
  float progress;
  float alpha;
  float scale;
  float layout_progress;
  uint32_t slot;
  uint32_t layout_from_mask;
  uint32_t layout_to_mask;
  // Normalized group centre; non-negative is explicit, -1 follows the
  // overlay centre, and at or below -1.5 the key keeps the default.
  float center_x;
  float center_y;
  // Group size relative to the overlay's requested scale; `scale` stays the
  // pure pop-animation scale the motion blur compares to the spring curve.
  float scale_ratio;
} ScreenwideKeyboardKey;

typedef struct {
  uint32_t key_count;
  uint32_t animation;
  uint32_t appearance;
  float scale;
  float progress;
  float maximum_width;
  float requested_scale;
  float center_x;
  float center_y;
  ScreenwideKeyboardKey keys[SCREENWIDE_KEYBOARD_MAX_KEYS];
} ScreenwideKeyboardOverlay;

typedef struct {
  uint32_t x;
  uint32_t width;
  uint32_t visible;
  uint32_t slot;
  float alpha;
  float scale;
  float progress;
  float layout_progress;
  uint32_t layout_from_mask;
  uint32_t layout_to_mask;
  float center_x;
  float center_y;
  float scale_ratio;
} ScreenwideKeyboardKeyUniforms;

typedef struct {
  uint32_t width;
  uint32_t height;
  uint32_t key_count;
  uint32_t animation;
  float scale;
  float layout_progress;
  float maximum_width;
  float requested_scale;
  float center_x;
  float center_y;
  ScreenwideKeyboardKeyUniforms keys[SCREENWIDE_KEYBOARD_MAX_KEYS];
} ScreenwideKeyboardUniforms;

_Static_assert(offsetof(ScreenwideKeyboardUniforms, keys) == 40,
               "Keyboard uniforms must match their Metal layout");
