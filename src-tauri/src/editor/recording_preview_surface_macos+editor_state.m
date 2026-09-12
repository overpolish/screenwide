// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include <math.h>

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

/// Turns the native editor on or off. Off is a full teardown: no transform
/// callback, no interaction view, and the workspace returns to its base rects
/// at 100% with no pan. Use `set_editor_suspended` to take input and chrome
/// away without disturbing the transform.
void screenwide_preview_surface_enable_editor(
    void *handle, screenwide_preview_transform_callback callback,
    void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.editorEnabled = callback != NULL;
    surface.transformCallback = callback;
    surface.transformContext = context;
    // Every layout re-asserts the editor state, so an enable that ignored the
    // suspension would flash the interaction view back over React's overlay on
    // the next resize. Suspension stays orthogonal: only its own setter clears
    // it, this just refuses to undo what it hid.
    surface.interaction.hidden = !surface.editorEnabled || surface.editorSuspended;
    if (!surface.editorEnabled) {
      [surface.interaction releaseCursorControl];
      surface.editorPanX = 0;
      surface.editorPanY = 0;
      surface.editorZoom = 1.0;
      surface.editorPanelFitWidth = 0;
    }
    redraw_selection(surface);
  });
}

/// Suspends the editor for as long as React owns the workarea. Everything that
/// takes input or paints native chrome goes quiet; everything that applies or
/// draws the transform keeps running, and the pan/zoom are never touched, so
/// resuming needs no restore step.
void screenwide_preview_surface_set_editor_suspended(void *handle,
                                                     int suspended) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  const BOOL next = suspended != 0;
  on_main_async(^{
    if (surface.editorSuspended == next) return;
    surface.editorSuspended = next;
    if (next) {
      surface.interaction.hidden = YES;
      // The hidden view keeps no mouse tracking, so hand the window's cursor
      // rects back or the workarea cursor would stay frozen under React.
      [surface.interaction releaseCursorControl];
    } else {
      surface.interaction.hidden = !surface.editorEnabled;
    }
    // Decides the selection layer and the action material together, in both
    // directions: hiding them here, restoring whichever the state calls for.
    redraw_selection(surface);
  });
}

/// Applies whatever the current fit basis is: the panel fit while a tool panel
/// holds one, otherwise 100% centred in the whole viewport. Base geometry stays
/// relative to the full viewport either way, so the basis is the only thing
/// that decides where a reset lands. Publishes the resulting zoom.
void apply_editor_fit_basis(ScreenwidePreviewSurface *surface) {
  NSRect base = editor_base_bounds(surface);
  NSSize viewport = surface.container.bounds.size;
  ScreenwideViewTransform fit = screenwide_workspace_panel_fit(
      viewport.width, viewport.height, base.origin.x, base.origin.y,
      base.size.width, base.size.height, surface.editorPanelFitWidth);
  surface.editorPanX = fit.pan_x;
  surface.editorPanY = fit.pan_y;
  surface.editorZoom = fit.zoom;
  apply_editor_transform(surface);
  if (surface.transformCallback)
    surface.transformCallback(surface.editorZoom * 100.0, surface.transformContext);
}

/// Fit once using native zoom/pan, and take the width as the new basis, so a
/// later double-click returns to the same place.
void screenwide_preview_surface_reset_editor_view(void *handle, double fitWidth) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    if (!surface.editorEnabled || surface.editorSuspended) return;
    // Native owns the transform for the length of a gesture; see
    // `screenwide_preview_surface_set_editor_zoom`.
    if (surface.interaction.selectionDragActive) return;
    surface.editorPanelFitWidth = fitWidth > 0 && isfinite(fitWidth) ? fitWidth : 0;
    apply_editor_fit_basis(surface);
  });
}

/// Moves the basis without moving the view: closing a tool panel leaves the
/// picture where the user left it but hands the next reset back to the full
/// viewport. A width of 0 is that full-viewport basis.
void screenwide_preview_surface_set_editor_fit_basis(void *handle,
                                                     double fitWidth) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.editorPanelFitWidth = fitWidth > 0 && isfinite(fitWidth) ? fitWidth : 0;
  });
}

void screenwide_preview_surface_set_editor_zoom(void *handle,
                                                double zoom_percent) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    if (!surface.editorEnabled) return;
    // React must not drive the transform while its own overlay owns the
    // workarea: the zoom it holds is a snapshot from before the suspension and
    // re-anchoring on it would move the workspace behind the overlay.
    if (surface.editorSuspended) return;
    // A native selection gesture (Frame resize, auto-fit Move) re-fits the
    // transform on every pointer sample and has already reported the exact
    // zoom to React; anything React sends back mid-gesture is that echo
    // rounded to a whole percent, and re-anchoring on it fights the gesture
    // (a visible flicker of the clip). Native owns the transform until
    // mouse-up.
    if (surface.interaction.selectionDragActive) return;
    NSPoint center = NSMakePoint(NSMidX(surface.interaction.bounds),
                                 NSMidY(surface.interaction.bounds));
    set_editor_zoom(surface, zoom_percent / 100.0, center);
  });
}
