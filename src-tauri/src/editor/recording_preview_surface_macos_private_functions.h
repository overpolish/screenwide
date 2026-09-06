// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_FUNCTIONS_H
#define SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_FUNCTIONS_H

@interface ScreenwidePreviewSurface (OSC)
- (void)redrawSelection;
@end
@interface ScreenwidePreviewSurface (Label)
- (BOOL)updateSelectionLabel:(NSString *)text
                      scale:(CGFloat)scale
                  lightMode:(uint32_t)lightMode
                     action:(BOOL)action;
- (BOOL)updateSelectionSecondaryLabel:(NSString *)text
                               scale:(CGFloat)scale
                           lightMode:(uint32_t)lightMode;
@end
NSRect editor_frame(ScreenwidePreviewSurface *surface, NSRect base);
void selection_action_layout(ScreenwidePreviewSurface *surface);
void selection_action_material_layout(ScreenwidePreviewSurface *surface);
void selection_action_render_surfaces(
    ScreenwidePreviewSurface *surface, CGFloat scale, uint32_t light_mode);
void selection_action_fills(ScreenwidePreviewSurface *surface,
                            uint32_t light_mode, float fills[8]);
BOOL selection_is_keyboard(ScreenwidePreviewSelection selection);
NSRect keyboard_hit_frame(ScreenwidePreviewSurface *surface,
                          ScreenwidePreviewSelection selection);
BOOL keyboard_body_contains(ScreenwidePreviewSurface *surface,
                            ScreenwidePreviewSelection selection,
                            NSPoint point);
void begin_keyboard_transform(ScreenwidePreviewSurface *surface);
void update_keyboard_transform(
    ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection selection,
    double scale);
NSRect editor_frame_with_transform(
    ScreenwidePreviewSurface *surface, NSRect base, double zoom,
    NSPoint pan);
NSRect rebase_workspace_fit(ScreenwidePreviewSurface *surface,
                                   NSRect displayed);
void reflow_recording_workspace_panes(
    ScreenwidePreviewSurface *surface, NSArray<NSValue *> *starts,
    NSUInteger selectedPane, NSRect resized);
void rebase_recording_workspace_fit(
    ScreenwidePreviewSurface *surface, NSArray<NSValue *> *starts,
    double zoom, NSPoint pan);
void redraw_selection(ScreenwidePreviewSurface *surface);
void apply_editor_transform(ScreenwidePreviewSurface *surface);
NSRect selection_display_frame_for(
    ScreenwidePreviewSurface *surface,
    ScreenwidePreviewSelection selection);
NSRect selection_display_frame(ScreenwidePreviewSurface *surface);
NSRect auto_fit_selection_bounds(
    ScreenwidePreviewSurface *surface,
    NSArray<NSValue *> *targets,
    ScreenwidePreviewSelection moved);
BOOL selection_is_frame(ScreenwidePreviewSurface *surface);
BOOL selection_target_at_point(ScreenwidePreviewSurface *surface,
                                      NSPoint point,
                                      ScreenwidePreviewSelection *result);
BOOL shared_selection_hit(ScreenwidePreviewSurface *surface,
                                 NSPoint point,
                                 ScreenwidePreviewSelection *selection,
                                 uint8_t *handle);
uint32_t shared_handle_edges(uint8_t handle);
void emit_selection_gesture(ScreenwidePreviewSurface *surface,
                                   uint32_t phase, uint32_t operation,
                                   uint32_t edges, double scale,
                                   double deltaX, double deltaY);
uint32_t selection_handle_edges(ScreenwidePreviewSurface *surface,
                                       NSPoint point);
BOOL selection_radius_hit(ScreenwidePreviewSurface *surface,
                                 NSPoint point);
double snap_selection_resize(ScreenwidePreviewSurface *surface,
                                    double scale, double anchorX,
                                    double anchorY, double vectorX,
                                    double vectorY, uint32_t edges,
                                    NSRect pane, double minimumScale,
                                    double maximumScale);
void clear_selection_snap_guides(ScreenwidePreviewSurface *surface);
void snap_selection_move(ScreenwidePreviewSurface *surface,
                                double *x, double *y);
void set_selection_cursor(NSCursor *cursor);
void set_selection_move_cursor(void);
void set_selection_cursor_at_point(ScreenwidePreviewSurface *surface,
                                   NSPoint point);
BOOL selection_action_hover(ScreenwidePreviewSurface *surface, NSPoint point);
BOOL selection_action_hit(ScreenwidePreviewSurface *surface, NSPoint point);
BOOL selection_action_clear_hover(ScreenwidePreviewSurface *surface);
BOOL selection_action_begin(ScreenwidePreviewSurface *surface, NSInteger button, NSPoint point);
BOOL selection_action_drag(ScreenwidePreviewSurface *surface, NSPoint point);
BOOL selection_action_end(ScreenwidePreviewSurface *surface, NSPoint point);
ScreenwidePreviewSelection selection_recenter_resize(
    ScreenwidePreviewSelection start, uint32_t edges, double delta_x,
    double delta_y, NSSize pane, double *scale);
double selection_recenter_scale(ScreenwidePreviewSelection start, ScreenwidePreviewSelection resized, uint32_t edges);
void selection_recenter_drag(ScreenwidePreviewSurface *surface,
    ScreenwidePreviewSelection start, uint32_t edges, double delta_x, double delta_y, NSSize pane);
void invalidate_selection_cursor_rects(ScreenwidePreviewSurface *surface);
void set_editor_zoom(ScreenwidePreviewSurface *surface,
                            double zoom, NSPoint anchor);
double maximum_editor_zoom(ScreenwidePreviewSurface *surface);
void clamp_editor_zoom_to_ceiling(ScreenwidePreviewSurface *surface);
void update_crop_magnifier(ScreenwidePreviewSurface *surface,
                                  NSPoint point, uint32_t edges);
void begin_workspace_frame_resize(ScreenwidePreviewSurface *surface);
void update_workspace_frame_resize(
    ScreenwidePreviewSurface *surface, NSRect start, NSRect resized);
BOOL update_workspace_auto_fit_move(
    ScreenwidePreviewSurface *surface, uint32_t selected_layer,
    double move_x, double move_y, NSRect start, NSRect resized);
void end_workspace_frame_resize(
    ScreenwidePreviewSurface *surface, BOOL commit);
void redraw_workspace(ScreenwidePreviewSurface *surface);
#endif
