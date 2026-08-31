// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

int screenwide_region_osc_set_magnifier_source(
    void *view_ptr, const uint8_t *rgba, size_t length, uint32_t width,
    uint32_t height) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  size_t expected = (size_t)width * height * 4;
  if (!s || !rgba || width == 0 || height == 0 || length != expected)
    return 0;
  id<MTLBuffer> source =
      [s.device newBufferWithBytes:rgba
                            length:length
                           options:MTLResourceStorageModeShared];
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(s)) {
    surface.magnifierSource = source;
    surface.magnifierSourceWidth = width;
    surface.magnifierSourceHeight = height;
    screenwide_region_osc_draw(surface);
  }
  return source != nil;
}

void screenwide_region_osc_set_input_enabled(void *view_ptr, int enabled) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  ScreenwideRegionOSC *active = nil;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root))
    if (surface.gestureActive) {
      active = surface;
      break;
    }
  if (root.inputEnabled && !enabled && active) {
    NativeOscResult result = {0};
    root.input(root.rustContext, 5, 0, 0, 0, &result);
    screenwide_region_osc_apply_ruler_result(root, result);
    if ((result.ruler_flags & 1) == 0)
      screenwide_region_osc_apply_region(
          root,
          result.has_region
              ? NSMakeRect(result.x, result.y, result.width, result.height)
              : NSZeroRect,
          root.visible);
  }
  if (!enabled)
    screenwide_region_osc_cancel_pointer_claim(root);
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    surface.gestureActive = NO;
    surface.inputEnabled = enabled != 0;
    if (surface != root) {
      surface.host.window.ignoresMouseEvents = !surface.inputEnabled;
      // Loading temporarily disables the peer panels while the React owner is
      // brought forward. Reassert their compositor ordering when ready input
      // returns, otherwise an external display can visually update while its
      // pointer events fall through to the desktop beneath it.
      if (surface.inputEnabled && root.visible && surface.host.window.visible) {
        surface.host.window.level = root.host.window.level;
        [surface.host.window orderFrontRegardless];
      }
    }
    if (!surface.inputEnabled) {
      ScreenwideRegionMagnifier magnifier = surface.magnifier;
      magnifier.active = 0;
      surface.magnifier = magnifier;
      if (surface.cursorHidden) {
        surface.cursorHidden = NO;
        [NSCursor unhide];
      }
      screenwide_region_osc_cursor_release(surface);
    }
    screenwide_region_osc_draw(surface);
  }
}

void screenwide_region_osc_set_exclusion_rect(void *view_ptr, double x,
                                              double y, double width,
                                              double height) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  root.exclusionRect = NSMakeRect(x, y, width, height);
  for (ScreenwideRegionOSC *peer in root.desktopPeers)
    peer.exclusionRect = NSZeroRect;
}

void screenwide_region_osc_set_show_handles(void *view_ptr, int show_handles) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(s))
    surface.showHandles = show_handles != 0;
}

void screenwide_region_osc_set_show_frame(void *view_ptr, int show_frame) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(s)) {
    surface.showFrame = show_frame != 0;
    screenwide_region_osc_draw(surface);
  }
}
