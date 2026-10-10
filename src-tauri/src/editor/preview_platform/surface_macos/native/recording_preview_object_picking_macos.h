// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_OBJECT_PICKING_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_OBJECT_PICKING_MACOS_H

#import <Foundation/Foundation.h>
#include <stdint.h>

/// What a press while picking objects picked: the object at its points, 0
/// to 1 across and down the camera's picture, a click's one or a painted
/// line's several, to keep or to remove.
enum {
  ScreenwideObjectPressKeep = 0,
  ScreenwideObjectPressRemove = 1,
};

/// A point 0 to 1 across and down the camera's picture.
typedef struct {
  float x;
  float y;
} ScreenwidePickedPoint;

typedef void (*screenwide_preview_object_pick_callback)(uint32_t press,
                                                        const ScreenwidePickedPoint *points,
                                                        uint32_t count, void *context);

/// An object picked in the frame the preview shows, 0 to 1 across and down
/// the camera's picture, and where among the lines' points the line painted
/// over it is, of none where it was clicked. The twin of Rust's
/// `PickedObjectDot`.
typedef struct {
  float x;
  float y;
  uint32_t id;
  uint32_t removed;
  uint32_t chosen;
  uint32_t line_first;
  uint32_t line_points;
} ScreenwidePickedObject;

/// What picking objects in the camera holds on a surface.
@interface ScreenwideObjectPicking : NSObject
@property(nonatomic) BOOL active;
/// The layer the camera's picture is, as the selection targets name it.
@property(nonatomic) uint32_t cameraLayer;
@property(nonatomic, strong) NSData *objects;
/// The points of the objects' painted lines, in turn.
@property(nonatomic, strong) NSData *lines;
@property(nonatomic) screenwide_preview_object_pick_callback callback;
@property(nonatomic) void *context;
/// The pick on its way: drawn chosen at once and kept until the objects
/// picked arrive with one more in them than `pendingSentWith`, or for a
/// while without. `pendingPoints` is its click, or the line painted.
@property(nonatomic) BOOL hasPending;
@property(nonatomic) BOOL pendingRemoves;
@property(nonatomic) BOOL pendingChosen;
@property(nonatomic, strong) NSMutableData *pendingPoints;
@property(nonatomic) NSUInteger pendingSentWith;
/// Counts the picks made, so what waits on one is let go once another is.
@property(nonatomic) uint64_t pendingToken;
@end

@class ScreenwidePreviewSurface;

/// The surface's picking state, made the first time it is asked for.
ScreenwideObjectPicking *object_picking_state(ScreenwidePreviewSurface *surface)
    __attribute__((visibility("hidden")));
/// Where the camera's whole picture lies in the view. `NO` while it is laid
/// out nowhere.
BOOL object_picking_camera_image(ScreenwidePreviewSurface *surface, NSRect *image)
    __attribute__((visibility("hidden")));
/// Where `point`, on the camera's picture `image`, lies in the view.
NSPoint object_picking_view_point(NSRect image, ScreenwidePickedPoint point) __attribute__((visibility("hidden")));
/// The points an object is drawn at: its painted line, or its click.
const ScreenwidePickedPoint *object_picking_points(ScreenwideObjectPicking *state,
                                           const ScreenwidePickedObject *object, NSUInteger *count)
    __attribute__((visibility("hidden")));

#endif
