// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_SCREENSHOT_REGION_OSC_MACOS_PRIVATE_FUNCTIONS_H
#define SCREENWIDE_SCREENSHOT_REGION_OSC_MACOS_PRIVATE_FUNCTIONS_H

void screenwide_region_osc_draw(ScreenwideRegionOSC *s);
void screenwide_region_osc_add_icon(ScreenwideRegionOscVertex *vertices,
                                    NSUInteger *count, NSSize size,
                                    uint8_t icon, CGFloat left, CGFloat top,
                                    CGFloat icon_size);
void screenwide_region_osc_input_install(ScreenwideRegionOSC *s);
void screenwide_region_osc_input_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_cursor_claim(ScreenwideRegionOSC *s);
void screenwide_region_osc_cursor_release(ScreenwideRegionOSC *s);
void screenwide_region_osc_cancel_pointer_claim(ScreenwideRegionOSC *s);
void screenwide_region_osc_claim_pointer_surface(void *view_ptr);
void screenwide_region_osc_ruler_refresh_pointer(void *view_ptr);
void screenwide_region_osc_appearance_install(ScreenwideRegionOSC *s);
void screenwide_region_osc_appearance_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_apply_ruler_result(ScreenwideRegionOSC *s,
                                               NativeOscResult result);
void screenwide_region_osc_ruler_attach(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_teardown(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_update_appearance(ScreenwideRegionOSC *s);
void screenwide_region_osc_ruler_set_transient_chrome(void *view_ptr,
                                                       int visible);
void screenwide_region_osc_ruler_apply_render_state(
    ScreenwideRegionOSC *s, ScreenwideRegionOscRenderState *state);
BOOL screenwide_region_osc_ruler_label_hit(
    ScreenwideRegionOSC *surface, NSPoint point,
    ScreenwideRulerLabelHit *hit);
NSUInteger screenwide_region_osc_ruler_vertex_capacity(
    ScreenwideRegionOSC *surface);
void screenwide_region_osc_ruler_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale);
void *screenwide_region_osc_attach(void *view_ptr, void *context,
                                   void (*release)(void *),
                                   NativeOscInput input,
                                   NativeOscLayout layout_changed);
ScreenwideRegionOSC *screenwide_region_osc_for_view(void *view_ptr);
ScreenwideRegionOSC *screenwide_region_osc_root(ScreenwideRegionOSC *s);
NSArray<ScreenwideRegionOSC *> *
screenwide_region_osc_surfaces(ScreenwideRegionOSC *s);
void screenwide_region_osc_apply_region(ScreenwideRegionOSC *s, NSRect region,
                                        BOOL visible);
size_t screenwide_region_osc_configure_desktop(
    void *view_ptr, uint32_t anchor_id,
    ScreenwideRegionDesktopDisplay *displays, size_t capacity,
    double *desktop_width, double *desktop_height,
    uint32_t *resolved_anchor_id, int *layout_changed);
void screenwide_region_osc_set_desktop_presented(void *view_ptr,
                                                  int presented);
int screenwide_region_osc_set_snapshot(void *view_ptr, uint32_t display_id,
                                       const uint8_t *rgba, size_t length,
                                       uint32_t width, uint32_t height);
void screenwide_region_osc_set_snapshot_presented(void *view_ptr,
                                                   int presented);
void screenwide_region_osc_set_snapshot_composited(void *view_ptr,
                                                    int composited);
void screenwide_region_osc_ocr_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_update_appearance(ScreenwideRegionOSC *surface);
BOOL screenwide_region_osc_ocr_control_input(ScreenwideRegionOSC *surface,
                                             NSPoint point, uint32_t phase);
BOOL screenwide_region_osc_ocr_cancel_input(ScreenwideRegionOSC *surface,
                                            NSPoint point, uint32_t phase);
void screenwide_region_osc_ocr_cancel_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_cancel_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_cancel_update_appearance(
    ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_attach(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_teardown(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_layout(ScreenwideRegionOSC *surface,
                                              BOOL visible);
void screenwide_region_osc_ocr_toolbar_render(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_toolbar_apply_update(
    ScreenwideRegionOSC *surface, ScreenwideOscControlUpdate update);
void screenwide_region_osc_ocr_toolbar_apply_confirm_update(
    ScreenwideRegionOSC *surface, ScreenwideOscConfirmUpdate update);
uint8_t screenwide_region_osc_ocr_toolbar_icon(
    ScreenwideRegionOSC *surface, NSUInteger index);
void screenwide_region_osc_ocr_set_cancel_visible(void *view_ptr, int visible);
NSUInteger screenwide_region_osc_ocr_vertex_capacity(ScreenwideRegionOSC *surface);
void screenwide_region_osc_ocr_add_vertices(
    ScreenwideRegionOSC *surface, ScreenwideRegionOscVertex *vertices,
    NSUInteger *count, NSSize size, CGFloat scale);
int screenwide_region_osc_set_ocr(void *view_ptr, uint32_t phase,
                                  const ScreenwideRegionOcrRect *rects,
                                  size_t count, const char *message);
size_t screenwide_ocr_toolbar_layout(
    double selection_x, double selection_y, double selection_width,
    double selection_height, double viewport_width, double viewport_height,
    const double *widths, double height, ScreenwideOcrToolbarRect *output,
    size_t capacity);

#endif
