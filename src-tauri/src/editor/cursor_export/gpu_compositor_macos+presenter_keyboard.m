// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#include <math.h>

#import "gpu_compositor_macos.h"
#import "gpu_compositor_macos_presenter_private.h"

static float keyboard_motion_spring(float progress) {
  float t = fminf(fmaxf(progress, 0.0f), 1.0f);
  float phase = 6.0f * t;
  return t >= 1.0f ? 1.0f
      : 1.0f - expf(-5.0f * t) *
          (cosf(phase) + (5.0f / 6.0f) * sinf(phase));
}

static float keyboard_gap(ScreenwideKeyboardUniforms keyboard) {
  float gap = INFINITY;
  uint32_t count = MIN(keyboard.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
  for (uint32_t index = 1; index < count; ++index) {
    float candidate = (float)keyboard.keys[index].x -
        (float)(keyboard.keys[index - 1].x + keyboard.keys[index - 1].width);
    if (candidate > 0.0f) gap = fminf(gap, candidate);
  }
  return isfinite(gap) ? gap : 0.0f;
}

static float keyboard_slot_width(ScreenwideKeyboardUniforms keyboard,
                                 uint32_t slot) {
  float width = 0.0f;
  uint32_t count = MIN(keyboard.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
  for (uint32_t index = 0; index < count; ++index)
    if (keyboard.keys[index].slot == slot)
      width = fmaxf(width, (float)keyboard.keys[index].width);
  return width;
}

static float keyboard_slot_left(ScreenwideKeyboardUniforms keyboard,
                                uint32_t slot, uint32_t mask) {
  uint32_t slots = 0;
  uint32_t count = MIN(keyboard.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
  for (uint32_t index = 0; index < count; ++index)
    slots = MAX(slots, keyboard.keys[index].slot + 1);
  float gap = keyboard_gap(keyboard);
  float total = gap * MAX((int)__builtin_popcount(mask) - 1, 0);
  for (uint32_t candidate = 0; candidate < slots; ++candidate)
    if ((mask & (1u << candidate)) != 0)
      total += keyboard_slot_width(keyboard, candidate);
  float left = ((float)keyboard.width - total) * 0.5f;
  for (uint32_t candidate = 0; candidate < slot; ++candidate)
    if ((mask & (1u << candidate)) != 0)
      left += keyboard_slot_width(keyboard, candidate) + gap;
  return left;
}

static float keyboard_layout_offset(ScreenwideKeyboardUniforms keyboard,
                                    uint32_t index, float full_width) {
  if (keyboard.key_count < 2) return 0.0f;
  ScreenwideKeyboardKeyUniforms key = keyboard.keys[index];
  float slotWidth = keyboard_slot_width(keyboard, key.slot);
  float fromCenter = keyboard_slot_left(
      keyboard, key.slot, key.layout_from_mask) + slotWidth * 0.5f;
  float toCenter = keyboard_slot_left(
      keyboard, key.slot, key.layout_to_mask) + slotWidth * 0.5f;
  float progress = keyboard_motion_spring(key.layout_progress);
  float targetCenter = fromCenter + (toCenter - fromCenter) * progress;
  float sourceOffset = targetCenter -
      ((float)key.x + (float)key.width * 0.5f);
  return sourceOffset * full_width / MAX((float)keyboard.width, 1.0f);
}

int screenwide_gpu_still_presenter_update_workspace_keyboard(
    void *handle, uint32_t pane_index, double center_x, double center_y,
    double scale_ratio) {
  if (handle == NULL || !isfinite(center_x) || !isfinite(center_y) ||
      !isfinite(scale_ratio) || scale_ratio <= 0.0) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  NSArray<NSValue *> *base = presenter.workspaceResizeLayers.count > 0
      ? presenter.workspaceResizeLayers : presenter.workspaceLayers;
  NSMutableArray<NSValue *> *updated =
      [NSMutableArray arrayWithCapacity:base.count];
  BOOL found = NO;
  for (NSValue *value in base) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index == pane_index && layer.keyboard.key_count > 0) {
      found = YES;
      layer.keyboard.center_x = (float)center_x;
      layer.keyboard.center_y = (float)center_y;
      layer.keyboard.requested_scale *= (float)scale_ratio;
      layer.keyboard.scale *= (float)scale_ratio;
      for (uint32_t index = 0;
           index < MIN(layer.keyboard.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
           ++index)
        layer.keyboard.keys[index].scale *= (float)scale_ratio;
    }
    [updated addObject:[NSValue valueWithBytes:&layer
                                       objCType:@encode(ScreenwideWorkspaceLayer)]];
  }
  if (!found) return 0;
  presenter.workspaceResizeApplied = YES;
  presenter.workspaceLayers = updated;
  return 1;
}

int screenwide_gpu_still_presenter_workspace_keyboard_bounds(
    void *handle, uint32_t pane_index, double *x, double *y,
    double *width, double *height) {
  if (handle == NULL || x == NULL || y == NULL || width == NULL ||
      height == NULL) return 0;
  ScreenwideStillPresenter *presenter = (__bridge ScreenwideStillPresenter *)handle;
  for (NSValue *value in presenter.workspaceLayers) {
    ScreenwideWorkspaceLayer layer;
    [value getValue:&layer size:sizeof(layer)];
    if (layer.pane_index != pane_index || layer.keyboard.key_count == 0 ||
        layer.canvas_width == 0 || layer.canvas_height == 0) continue;
    ScreenwideKeyboardArtwork *artwork = screenwide_keyboard_artwork(
        presenter.device, presenter.keyboardArtworks, layer.keyboard,
        layer.canvas_height);
    if (artwork == nil || artwork.uniforms.width == 0 ||
        artwork.uniforms.height == 0) return 0;
    ScreenwideKeyboardUniforms keyboard = artwork.uniforms;
    float requested = keyboard.requested_scale > 0.0f
        ? keyboard.requested_scale : keyboard.scale;
    float effective = requested;
    if (keyboard.maximum_width > 0.0f) {
      float available = (float)layer.canvas_width * (1.0f - 0.055f * 2.0f);
      float unitWidth = (float)layer.canvas_height * (60.0f / 1080.0f) *
          keyboard.maximum_width / 20.0f;
      effective = fminf(requested, available / fmaxf(unitWidth * 1.12f, 0.0001f));
    }
    float rowHeight = (float)layer.canvas_height * (60.0f / 1080.0f) * effective;
    float rowWidth = rowHeight * (float)keyboard.width /
        MAX((float)keyboard.height, 1.0f);
    float defaultX = (float)layer.canvas_width * 0.5f;
    float defaultY =
        (float)layer.canvas_height * (1.0f - 0.055f) - rowHeight * 0.5f;
    float overlayX = keyboard.center_x >= 0.0f
        ? keyboard.center_x * (float)layer.canvas_width : defaultX;
    float overlayY = keyboard.center_y >= 0.0f
        ? keyboard.center_y * (float)layer.canvas_height : defaultY;
    float left = INFINITY, top = INFINITY;
    float right = -INFINITY, bottom = -INFINITY;
    uint32_t count = MIN(keyboard.key_count, SCREENWIDE_KEYBOARD_MAX_KEYS);
    for (uint32_t index = 0; index < count; ++index) {
      ScreenwideKeyboardKeyUniforms key = keyboard.keys[index];
      if (key.visible == 0 || key.alpha <= 0.002f) continue;
      float animationScale = key.scale / fmaxf(requested, 0.001f);
      if (animationScale <= 0.002f) continue;
      // Keys carry their own group centre and size while a differently
      // placed badge is still on screen.
      float ratio = key.scale_ratio > 0.0f ? key.scale_ratio : 1.0f;
      float keyRowHeight = rowHeight * ratio;
      float keyRowWidth = rowWidth * ratio;
      float centerX = key.center_x >= 0.0f
          ? key.center_x * (float)layer.canvas_width
          : (key.center_x > -1.5f ? overlayX : defaultX);
      float centerY = key.center_y >= 0.0f
          ? key.center_y * (float)layer.canvas_height
          : (key.center_y > -1.5f
                 ? overlayY
                 : (float)layer.canvas_height * (1.0f - 0.055f) -
                       keyRowHeight * 0.5f);
      float keyWidth = keyRowHeight * (float)key.width /
          MAX((float)keyboard.height, 1.0f);
      float keyCenterX = centerX - keyRowWidth * 0.5f +
          keyRowWidth * ((float)key.x + (float)key.width * 0.5f) /
              MAX((float)keyboard.width, 1.0f) +
          keyboard_layout_offset(keyboard, index, keyRowWidth);
      float halfWidth = keyWidth * animationScale * 0.5f;
      float halfHeight = keyRowHeight * animationScale * 0.5f;
      left = fminf(left, keyCenterX - halfWidth);
      right = fmaxf(right, keyCenterX + halfWidth);
      top = fminf(top, centerY - halfHeight);
      bottom = fmaxf(bottom, centerY + halfHeight);
    }
    if (!isfinite(left) || right <= left || bottom <= top) return 0;
    *x = left / (double)layer.canvas_width;
    *y = top / (double)layer.canvas_height;
    *width = (right - left) / (double)layer.canvas_width;
    *height = (bottom - top) / (double)layer.canvas_height;
    return 1;
  }
  return 0;
}
