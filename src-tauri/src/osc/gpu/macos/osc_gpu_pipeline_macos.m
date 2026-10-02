// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"

ScreenwideRegionOscRenderState screenwide_region_osc_render_state(
    uint32_t light_mode) {
  ScreenwideOscControlPalette palette =
      screenwide_osc_control_palette(light_mode);
  ScreenwideOscOverlayPalette overlay = screenwide_osc_overlay_palette();
  ScreenwideOscOcrPalette ocr = screenwide_osc_ocr_palette(light_mode);
  ScreenwideOscRulerPalette ruler =
      screenwide_osc_ruler_palette(light_mode);
  ScreenwideRegionOscRenderState state = {.light_mode = light_mode};
  memcpy(state.overlay_shade, overlay.shade, sizeof(state.overlay_shade));
  memcpy(state.control_fill, palette.fill, sizeof(state.control_fill));
  memcpy(state.control_outline, palette.outline,
         sizeof(state.control_outline));
  memcpy(state.ocr_colors, &ocr, sizeof(state.ocr_colors));
  memcpy(state.ruler_colors, &ruler, sizeof(state.ruler_colors));
  return state;
}
