// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#import "recording_preview_object_picking_macos.h"

#include <math.h>
#include <objc/runtime.h>

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

@implementation ScreenwideObjectPicking
@end

static char kObjectPickingKey;

/// How far a press must travel, in points, before it paints a line.
static const CGFloat kPaintSlop = 3.0;
/// How far apart the points kept along a painted line are, in points.
static const CGFloat kPaintStep = 4.0;
/// How long a pick on its way waits to arrive before its dot goes.
static const double kPendingSeconds = 4.0;

/// The surface's picking state, made the first time it is asked for. It is
/// held beside the surface, as this category's own, and goes with it.
SCREENWIDE_PREVIEW_PRIVATE ScreenwideObjectPicking *object_picking_state(ScreenwidePreviewSurface *surface) {
  ScreenwideObjectPicking *state = objc_getAssociatedObject(surface, &kObjectPickingKey);
  if (state == nil) {
    state = [ScreenwideObjectPicking new];
    objc_setAssociatedObject(surface, &kObjectPickingKey, state, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  }
  return state;
}

SCREENWIDE_PREVIEW_PRIVATE BOOL object_picking_active(ScreenwidePreviewSurface *surface) {
  return object_picking_state(surface).active;
}

/// Where the camera's whole picture lies in the view, as the selection
/// targets place it: its own pane, the recording itself, or baked into the
/// screen. `NO` while it is laid out nowhere.
SCREENWIDE_PREVIEW_PRIVATE BOOL object_picking_camera_image(ScreenwidePreviewSurface *surface, NSRect *image) {
  uint32_t layer = object_picking_state(surface).cameraLayer;
  for (NSValue *value in surface.selectionTargets) {
    ScreenwidePreviewSelection target;
    [value getValue:&target size:sizeof(target)];
    if (target.layer_id != layer || selection_is_keyboard(target)) continue;
    *image = selection_image_frame_for(surface, target);
    return image->size.width > 0.0 && image->size.height > 0.0;
  }
  return NO;
}

/// Where `point`, on the camera's picture `image`, lies in the view.
SCREENWIDE_PREVIEW_PRIVATE NSPoint object_picking_view_point(NSRect image, ScreenwidePickedPoint point) {
  return NSMakePoint(NSMinX(image) + point.x * image.size.width,
                     NSMinY(image) + point.y * image.size.height);
}

/// The points an object is drawn at: its painted line, or its click.
SCREENWIDE_PREVIEW_PRIVATE const ScreenwidePickedPoint *object_picking_points(ScreenwideObjectPicking *state,
                                                  const ScreenwidePickedObject *object,
                                                  NSUInteger *count) {
  NSUInteger total = state.lines.length / sizeof(ScreenwidePickedPoint);
  if (object->line_points > 0 && object->line_first + object->line_points <= total) {
    *count = object->line_points;
    return (const ScreenwidePickedPoint *)state.lines.bytes + object->line_first;
  }
  *count = 1;
  return (const ScreenwidePickedPoint *)&object->x;
}

/// The point `at` in the view, 0 to 1 across and down the camera's picture
/// `image`, kept inside it.
static ScreenwidePickedPoint picture_point(NSRect image, NSPoint at) {
  return (ScreenwidePickedPoint){
      (float)fmin(1.0, fmax(0.0, (at.x - NSMinX(image)) / image.size.width)),
      (float)fmin(1.0, fmax(0.0, (at.y - NSMinY(image)) / image.size.height))};
}

/// Hands the pick over, shown chosen until it arrives.
static void send_pending(ScreenwideObjectPicking *state, ScreenwidePreviewSurface *surface) {
  state.hasPending = YES;
  state.pendingChosen = YES;
  state.pendingToken += 1;
  state.pendingSentWith = state.objects.length / sizeof(ScreenwidePickedObject);
  redraw_selection(surface);
  if (state.callback != NULL)
    state.callback(state.pendingRemoves ? ScreenwideObjectPressRemove : ScreenwideObjectPressKeep,
                   state.pendingPoints.bytes,
                   (uint32_t)(state.pendingPoints.length / sizeof(ScreenwidePickedPoint)),
                   state.context);
  uint64_t token = state.pendingToken;
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, (int64_t)(kPendingSeconds * NSEC_PER_SEC)),
                 dispatch_get_main_queue(), ^{
                   if (state.pendingToken != token || !state.hasPending) return;
                   state.hasPending = NO;
                   redraw_selection(surface);
                 });
}

/// Follows the press until it is let go: one that travels paints a line
/// through where it went, drawn as it goes; one that does not is a click.
/// Either is then picked.
static void paint_or_click(ScreenwideObjectPicking *state, ScreenwidePreviewInteractionView *view,
                           NSRect image, NSPoint origin, BOOL removes) {
  ScreenwidePreviewSurface *surface = view.surface;
  NSMutableData *points = [NSMutableData data];
  ScreenwidePickedPoint first = picture_point(image, origin);
  [points appendBytes:&first length:sizeof(first)];
  NSPoint last = origin;
  BOOL painting = NO;
  state.pendingRemoves = removes;
  while (YES) {
    NSEvent *next = [view.window
        nextEventMatchingMask:NSEventMaskLeftMouseDragged | NSEventMaskLeftMouseUp];
    NSPoint at = [view convertPoint:next.locationInWindow fromView:nil];
    BOOL up = next.type == NSEventTypeLeftMouseUp;
    if (!painting && hypot(at.x - origin.x, at.y - origin.y) < kPaintSlop) {
      if (up) break;
      continue;
    }
    if (painting && !up && hypot(at.x - last.x, at.y - last.y) < kPaintStep) continue;
    painting = YES;
    last = at;
    ScreenwidePickedPoint point = picture_point(image, at);
    [points appendBytes:&point length:sizeof(point)];
    // The line is drawn as it is painted, as the pick on its way.
    state.pendingPoints = points;
    state.hasPending = YES;
    state.pendingChosen = YES;
    state.pendingToken += 1;
    redraw_selection(surface);
    if (up) break;
  }
  state.pendingPoints = points;
  send_pending(state, surface);
}

SCREENWIDE_PREVIEW_PRIVATE BOOL object_picking_mouse_down(ScreenwidePreviewInteractionView *view,
                                                          NSEvent *event, NSPoint point) {
  ScreenwidePreviewSurface *surface = view.surface;
  ScreenwideObjectPicking *state = object_picking_state(surface);
  if (!state.active) return NO;
  view.selectionDragActive = NO;
  NSRect image;
  BOOL placed = object_picking_camera_image(surface, &image);
  BOOL onPicture = placed && event.buttonNumber == 0 && NSPointInRect(point, image);
  if (!onPicture) {
    // Off the picture a press pans, and a double-click fits the view.
    if (event.clickCount >= 2) apply_editor_fit_basis(surface);
    else view.panning = YES;
    return YES;
  }
  // A double-click is one pick: its second press picks nothing more.
  if (event.clickCount >= 2) return YES;
  // On the picture a click picks the object under it, and a drag paints a
  // line over one, Option either way removing it.
  paint_or_click(state, view, image, point,
                 (event.modifierFlags & NSEventModifierFlagOption) != 0);
  return YES;
}

/// Whether `objects`, with their `lines`, hold the pick on its way: one
/// starting where it starts, with as many points.
static BOOL arrived(ScreenwideObjectPicking *state, NSData *objects, NSData *lines) {
  NSUInteger sent = state.pendingPoints.length / sizeof(ScreenwidePickedPoint);
  if (sent == 0) return NO;
  ScreenwidePickedPoint start = ((const ScreenwidePickedPoint *)state.pendingPoints.bytes)[0];
  const ScreenwidePickedObject *items = objects.bytes;
  NSUInteger total = objects.length / sizeof(ScreenwidePickedObject);
  NSUInteger points = lines.length / sizeof(ScreenwidePickedPoint);
  for (NSUInteger index = 0; index < total; index++) {
    ScreenwidePickedPoint at = {items[index].x, items[index].y};
    NSUInteger count = 1;
    if (items[index].line_points > 0 && items[index].line_first + items[index].line_points <= points) {
      at = ((const ScreenwidePickedPoint *)lines.bytes)[items[index].line_first];
      count = items[index].line_points;
    }
    if (count == sent && fabsf(at.x - start.x) < 1e-4f && fabsf(at.y - start.y) < 1e-4f) return YES;
  }
  return NO;
}

void screenwide_preview_surface_set_object_picking(void *handle, uint32_t active,
                                                   uint32_t camera_layer,
                                                   const ScreenwidePickedObject *objects,
                                                   uint32_t count,
                                                   const ScreenwidePickedPoint *lines,
                                                   uint32_t line_points) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  NSData *copied = [NSData dataWithBytes:(count == 0 ? NULL : objects)
                                  length:count * sizeof(ScreenwidePickedObject)];
  NSData *copiedLines = [NSData dataWithBytes:(line_points == 0 ? NULL : lines)
                                       length:line_points * sizeof(ScreenwidePickedPoint)];
  // Always queued, never run at once from the main thread: the objects sent
  // for a new choice from there must not land ahead of those a still sent
  // from its own thread a moment before, which would then put the old choice
  // back.
  dispatch_async(dispatch_get_main_queue(), ^{
    ScreenwideObjectPicking *state = object_picking_state(surface);
    // The pick has arrived once the objects come back with one more, or with
    // one where it was made; with picking stopped, it is let go.
    if (!active || (state.hasPending && (count > state.pendingSentWith ||
                                         arrived(state, copied, copiedLines)))) {
      state.pendingToken += 1;
      state.hasPending = NO;
    }
    state.active = active != 0;
    state.cameraLayer = camera_layer;
    state.objects = copied;
    state.lines = copiedLines;
    redraw_selection(surface);
  });
}

void screenwide_preview_surface_set_object_pick_callback(
    void *handle, screenwide_preview_object_pick_callback callback, void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    object_picking_state(surface).callback = callback;
    object_picking_state(surface).context = context;
  });
}
