// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_OSC_CONTROLS_H
#define SCREENWIDE_OSC_CONTROLS_H

#include <stddef.h>
#include <stdint.h>

typedef struct {
  double x;
  double y;
  double width;
  double height;
  uint8_t kind;
  uint8_t color;
  uint8_t size;
  uint8_t disabled;
  uint8_t icon;
} ScreenwideOscControlSpec;

typedef struct {
  uint8_t consumed;
  uint8_t changed;
  uint8_t activated;
  uint8_t animating;
} ScreenwideOscControlUpdate;

typedef struct {
  float fill[4];
  float foreground[4];
} ScreenwideOscControlVisual;

typedef struct {
  double height;
  double radius;
  double padding_x;
  double gap;
  double icon_size;
  double font_size;
  double line_height;
} ScreenwideOscControlMetrics;

typedef struct {
  double tight;
  double control;
  double control_inset;
  double section;
  double window_inset;
} ScreenwideOscControlSpacing;

typedef struct {
  const uint8_t *pixels;
  size_t length;
  uint32_t width;
  uint32_t height;
  uint32_t columns;
} ScreenwideOscIconAtlas;

typedef struct {
  uint8_t idle_icon;
  uint8_t armed_icon;
  uint8_t idle_color;
  uint8_t armed_color;
  uint32_t timeout_ms;
} ScreenwideOscConfirmSpec;

typedef struct {
  uint8_t confirmed;
  uint8_t changed;
  uint8_t animating;
  uint8_t armed;
} ScreenwideOscConfirmUpdate;

typedef struct {
  uint8_t icon;
  uint8_t padding[3];
  float foreground[4];
  float opacity;
  float scale;
} ScreenwideOscConfirmLayer;

void *screenwide_osc_control_group_create(void);
ScreenwideOscControlMetrics screenwide_osc_control_metrics(
    uint8_t kind, uint8_t size);
ScreenwideOscControlSpacing screenwide_osc_control_spacing(void);
void screenwide_osc_control_group_destroy(void *handle);
void screenwide_osc_control_group_layout(
    void *handle, const ScreenwideOscControlSpec *specs, size_t count);
ScreenwideOscControlUpdate screenwide_osc_control_group_hover(
    void *handle, double x, double y);
uint8_t screenwide_osc_control_group_hit(void *handle, double x, double y);
ScreenwideOscControlUpdate screenwide_osc_control_group_down(
    void *handle, double x, double y);
ScreenwideOscControlUpdate screenwide_osc_control_group_up(
    void *handle, double x, double y);
ScreenwideOscControlUpdate screenwide_osc_control_group_clear_hover(
    void *handle);
size_t screenwide_osc_control_group_visuals(
    void *handle, uint8_t dark, ScreenwideOscControlVisual *output,
    size_t capacity);
uint8_t screenwide_osc_control_group_is_animating(void *handle);
ScreenwideOscIconAtlas screenwide_osc_icon_atlas(void);
void *screenwide_osc_confirm_create(ScreenwideOscConfirmSpec spec);
void screenwide_osc_confirm_destroy(void *handle);
ScreenwideOscConfirmUpdate screenwide_osc_confirm_press(void *handle);
ScreenwideOscConfirmUpdate screenwide_osc_confirm_expire(void *handle);
size_t screenwide_osc_confirm_layers(
    void *handle, uint8_t dark, ScreenwideOscConfirmLayer *output,
    size_t capacity);
uint8_t screenwide_osc_confirm_is_animating(void *handle);

#endif
