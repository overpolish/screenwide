// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_FUNCTIONS_H
#define SCREENWIDE_RECORDING_PREVIEW_SURFACE_MACOS_PRIVATE_FUNCTIONS_H

@interface ScreenwidePreviewSurface (OSC)
- (void)redrawSelection;
@end
NSRect editor_frame(ScreenwidePreviewSurface *surface, NSRect base);
NSRect editor_base_bounds(ScreenwidePreviewSurface *surface);
BOOL selection_is_keyboard(ScreenwidePreviewSelection selection);
NSRect keyboard_hit_frame(ScreenwidePreviewSurface *surface,
                          ScreenwidePreviewSelection selection);
BOOL keyboard_body_contains(ScreenwidePreviewSurface *surface,
                            ScreenwidePreviewSelection selection,
                            NSPoint point);
double keyboard_resize_limit(ScreenwidePreviewSelection start,
    double anchorX, double anchorY, double maximum);
void clamp_keyboard_move(ScreenwidePreviewSurface *surface,
    ScreenwidePreviewSelection start, double *x, double *y);
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
void apply_editor_fit_basis(ScreenwidePreviewSurface *surface);
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
NSRect selection_image_frame_for(ScreenwidePreviewSurface *surface,
                                 ScreenwidePreviewSelection selection);
/// The arrow tool's pointer handling. Each returns YES when the press,
/// movement or release belonged to the tool and nothing else should see it.
BOOL annotation_mouse_down(ScreenwidePreviewInteractionView *view,
                           NSPoint point);
BOOL annotation_mouse_dragged(ScreenwidePreviewInteractionView *view,
                              NSPoint point);
BOOL annotation_mouse_up(ScreenwidePreviewInteractionView *view, NSPoint point);
/// The three grips of the chosen arrow, in the same handle shape the
/// selection OSC draws its corners with.
void annotation_add_osc(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                        NSSize size, ScreenwidePreviewSurface *surface,
                        CGFloat scale);
/// What the pointer does over the picture right now, or `None` when no tool
/// is in hand or the surface cannot place a mark.
ScreenwideAnnotationMode annotation_active_mode(
    ScreenwidePreviewSurface *surface);
/// Whether the arrow chrome replaces the layer's own: always while the arrow
/// tool is in hand, and while the select tool holds an arrow.
BOOL annotation_owns_chrome(ScreenwidePreviewSurface *surface);
/// The cursor the arrow chrome asks for under `point`, or nil to leave the
/// choice to the layer.
NSCursor *annotation_cursor(ScreenwidePreviewSurface *surface, NSPoint point);
/// Re-reads which arrow the pointer rests on and, when it changed, starts the
/// hover pulse that grows the halo. `dragging` reports no arrow at all.
void annotation_update_hover(ScreenwidePreviewSurface *surface, NSPoint point,
                             BOOL dragging);
/// The chosen arrow's grip under `point`, or -1.
NSInteger annotation_handle_at_point(ScreenwidePreviewSurface *surface,
                                     NSPoint point);
/// The topmost arrow whose shaft `point` lands on, or -1.
NSInteger annotation_shaft_at_point(ScreenwidePreviewSurface *surface,
                                    NSPoint point);
/// The whole source image's rectangle on screen, in the flipped interaction
/// view's coordinates.
NSRect annotation_image_frame(ScreenwidePreviewSurface *surface);
#endif
