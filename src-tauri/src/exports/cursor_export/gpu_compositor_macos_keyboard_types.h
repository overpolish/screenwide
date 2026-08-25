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
} ScreenwideKeyboardKey;

typedef struct {
  uint32_t key_count;
  uint32_t animation;
  uint32_t appearance;
  float scale;
  float progress;
  float maximum_width;
  float requested_scale;
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
} ScreenwideKeyboardKeyUniforms;

typedef struct {
  uint32_t width;
  uint32_t height;
  uint32_t key_count;
  uint32_t animation;
  float scale;
  float layout_progress;
  float padding[2];
  ScreenwideKeyboardKeyUniforms keys[SCREENWIDE_KEYBOARD_MAX_KEYS];
} ScreenwideKeyboardUniforms;

_Static_assert(offsetof(ScreenwideKeyboardUniforms, keys) == 32,
               "Keyboard uniforms must match their Metal layout");
