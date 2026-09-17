// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_SCREENSHOT_REGION_OSC_MACOS_TYPES_H
#define SCREENWIDE_SCREENSHOT_REGION_OSC_MACOS_TYPES_H

#import <AppKit/AppKit.h>
#include <stddef.h>

typedef struct {
  uint8_t status, gesture, handle, cursor, has_region;
  double x, y, width, height;
  uint32_t ruler_color;
  uint8_t ruler_flags;
  uint8_t ruler_padding[3];
} NativeOscResult;
typedef void (*NativeOscInput)(void *, uint32_t, double, double, uint8_t,
                               NativeOscResult *);
typedef void (*NativeOscLayout)(void *);
typedef struct {
  uint32_t id;
  double x, y, width, height, scale;
} ScreenwideRegionDesktopDisplay;
typedef struct {
  double x, y, width, height;
  uint8_t kind;
  uint8_t padding[7];
} ScreenwideRegionOcrRect;
typedef struct {
  double x, y, width, height;
} ScreenwideOcrToolbarRect;
typedef struct {
  uint64_t id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
  double label_anchor_x, label_anchor_y;
} NativeRulerMeasurement;
typedef struct {
  uint32_t display_id;
  uint32_t padding;
  double zoom;
  double origin_x, origin_y;
} NativeRulerViewport;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double start, end, position;
  double label_anchor_x, label_anchor_y;
} NativeRulerProbe;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double position;
} NativeRulerGuide;
typedef struct {
  uint64_t id;
  uint64_t owner_id;
  uint32_t display_id;
  uint8_t axis;
  uint8_t flags;
  uint8_t padding[2];
  double start, end, position;
  double label_anchor_x, label_anchor_y;
} NativeRulerGuideGap;
typedef struct {
  uint64_t id;
  uint32_t display_id;
  uint8_t corner;
  uint8_t flags;
  uint8_t padding[2];
  double x, y, width, height, radius;
  double label_anchor_x, label_anchor_y;
} NativeRulerRadius;
typedef struct {
  uint64_t id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
} NativeRulerCenterline;
typedef struct {
  uint64_t owner_id;
  double x, y, width, height;
  uint8_t flags;
  uint8_t padding[7];
} NativeRulerInnerObject;
typedef struct {
  uint64_t id;
  uint8_t kind;
  uint8_t padding[7];
  NSPoint center;
} ScreenwideRulerLabelHit;
_Static_assert(sizeof(ScreenwideRegionOcrRect) == 40,
               "ScreenwideRegionOcrRect ABI must match Rust");
_Static_assert(offsetof(ScreenwideRegionOcrRect, kind) == 32,
               "ScreenwideRegionOcrRect.kind ABI must match Rust");
_Static_assert(sizeof(NativeRulerMeasurement) == 64,
               "NativeRulerMeasurement ABI must match Rust");
_Static_assert(offsetof(NativeRulerMeasurement, flags) == 40,
               "NativeRulerMeasurement.flags ABI must match Rust");
_Static_assert(offsetof(NativeRulerMeasurement, label_anchor_x) == 48,
               "NativeRulerMeasurement.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerViewport) == 32,
               "NativeRulerViewport ABI must match Rust");
_Static_assert(offsetof(NativeRulerViewport, zoom) == 8,
               "NativeRulerViewport.zoom ABI must match Rust");
_Static_assert(sizeof(NativeRulerProbe) == 56,
               "NativeRulerProbe ABI must match Rust");
_Static_assert(offsetof(NativeRulerProbe, start) == 16,
               "NativeRulerProbe.start ABI must match Rust");
_Static_assert(offsetof(NativeRulerProbe, label_anchor_x) == 40,
               "NativeRulerProbe.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerGuide) == 24,
               "NativeRulerGuide ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuide, position) == 16,
               "NativeRulerGuide.position ABI must match Rust");
_Static_assert(sizeof(NativeRulerGuideGap) == 64,
               "NativeRulerGuideGap ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuideGap, start) == 24,
               "NativeRulerGuideGap.start ABI must match Rust");
_Static_assert(offsetof(NativeRulerGuideGap, label_anchor_x) == 48,
               "NativeRulerGuideGap.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerRadius) == 72,
               "NativeRulerRadius ABI must match Rust");
_Static_assert(offsetof(NativeRulerRadius, x) == 16,
               "NativeRulerRadius.x ABI must match Rust");
_Static_assert(offsetof(NativeRulerRadius, label_anchor_x) == 56,
               "NativeRulerRadius.label_anchor_x ABI must match Rust");
_Static_assert(sizeof(NativeRulerCenterline) == 48,
               "NativeRulerCenterline ABI must match Rust");
_Static_assert(offsetof(NativeRulerCenterline, flags) == 40,
               "NativeRulerCenterline.flags ABI must match Rust");
_Static_assert(sizeof(NativeRulerInnerObject) == 48,
               "NativeRulerInnerObject ABI must match Rust");
_Static_assert(offsetof(NativeRulerInnerObject, flags) == 40,
               "NativeRulerInnerObject.flags ABI must match Rust");
size_t native_osc_ruler_measurements(void *context,
                                     NativeRulerMeasurement *output,
                                     size_t capacity);
size_t native_osc_ruler_viewports(void *context,
                                  NativeRulerViewport *output,
                                  size_t capacity);
size_t native_osc_ruler_probes(void *context, NativeRulerProbe *output,
                              size_t capacity);
size_t native_osc_ruler_guides(void *context, NativeRulerGuide *output,
                              size_t capacity);
size_t native_osc_ruler_guide_gaps(void *context,
                                  NativeRulerGuideGap *output,
                                  size_t capacity);
size_t native_osc_ruler_radii(void *context, NativeRulerRadius *output,
                             size_t capacity);
size_t native_osc_ruler_centerlines(void *context,
                                    NativeRulerCenterline *output,
                                    size_t capacity);
size_t native_osc_ruler_inner_objects(void *context,
                                      NativeRulerInnerObject *output,
                                      size_t capacity);
int native_osc_ruler_viewport_input(void *context, uint32_t display_id,
                                    uint32_t operation, double anchor_x,
                                    double anchor_y, double delta_x,
                                    double delta_y, NativeOscResult *output);
void native_osc_ruler_label_input(
    void *context, uint32_t operation, uint8_t kind, uint64_t id,
    double pointer_x, double pointer_y, double label_center_x,
    double label_center_y, NativeOscResult *output);
void screenwide_set_region_expected_cursor(NSCursor *cursor);

#endif
