// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

void screenwide_preview_surface_set_pointer_down_callback(
    void *handle, screenwide_preview_pointer_down_callback callback,
    void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.pointerDownCallback = callback;
    surface.pointerDownContext = context;
  });
}
