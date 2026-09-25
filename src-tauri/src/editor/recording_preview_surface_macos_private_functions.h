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
/// The snap chrome for the sample on screen: the axis guides a counter
/// landed on, the equal gaps it lined up with, and the element an arrow's
/// tip took. Nothing at all when the last sample snapped to nothing.
void annotation_add_snap_osc(ScreenwideRegionOscVertex *vertices,
                             NSUInteger *count, NSSize size,
                             ScreenwidePreviewSurface *surface, CGFloat scale);
/// Every published annotation, capped at what one layer can carry, or NULL.
const ScreenwidePreviewAnnotation *annotation_items(
    ScreenwidePreviewSurface *surface, NSUInteger *count);
/// What the pointer does over the picture right now, or `None` when no tool
/// is in hand or the surface cannot place an annotation.
ScreenwideAnnotationMode annotation_active_mode(
    ScreenwidePreviewSurface *surface);
/// Whether a tool in hand makes a new annotation on empty picture. The one
/// answer to "does this mode draw at all", the twin of Rust's
/// `annotations::gesture::drawing_kind`.
BOOL annotation_drawing_mode(ScreenwideAnnotationMode mode);
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
/// Re-measures the halo against the picture's current size, a frame after the
/// transform that changed it. The width above the facade is in canvas pixels,
/// so a transform that redraws the picture at another size strands the halo
/// at the size the pointer arrived to until it is measured again.
void annotation_refresh_hover(ScreenwidePreviewSurface *surface);
/// The chosen arrow's grip under `point`, or -1.
NSInteger annotation_handle_at_point(ScreenwidePreviewSurface *surface,
                                     NSPoint point);
/// The topmost arrow whose shaft `point` lands on, or -1.
NSInteger annotation_shaft_at_point(ScreenwidePreviewSurface *surface,
                                    NSPoint point);
/// Whether a text box is being typed into.
BOOL annotation_text_editing(ScreenwidePreviewSurface *surface);
/// Ends the typing, telling Rust the text it ended with. Answers whether that
/// text has anything to read, which is what Rust keeps the box for.
BOOL annotation_text_finish(ScreenwidePreviewSurface *surface);
/// Lays the text view over its box again, at the box's current place and
/// type size, or ends the typing when there is no box left to type into.
void annotation_text_layout(ScreenwidePreviewSurface *surface);
/// A press typing owns: one that ends the typing and does nothing else, and
/// a double-click on a text box, which asks Rust to open it. A press outside
/// the box being typed into ends the typing and, answering NO, carries on as
/// the press it is. YES when the press was typing's.
BOOL annotation_text_press(ScreenwidePreviewInteractionView *view, NSEvent *event);
/// The I-beam over the text being typed into, or nil anywhere else.
NSCursor *annotation_text_cursor(ScreenwidePreviewSurface *surface, NSPoint point);
/// The whole source image's rectangle on screen, in the flipped interaction
/// view's coordinates.
NSRect annotation_image_frame(ScreenwidePreviewSurface *surface);
/// The grips one annotation shows, in display points, and which grip each
/// is. Answers how many, never more than `SCREENWIDE_ANNOTATION_MAX_GRIPS`.
NSUInteger annotation_grips(NSRect image, ScreenwidePreviewAnnotation item,
                            NSPoint *handles, uint32_t *kinds);
/// A redaction's eight box grips, in the layer selection's order, then its
/// radius dot.
NSUInteger annotation_redact_grips(NSRect image, ScreenwidePreviewAnnotation item,
                                   NSPoint *handles, uint32_t *kinds);
/// The layer selection's own box chrome around the chosen redaction. Answers
/// whether the chosen annotation is a redaction, whose box chrome stands in
/// for the grips `annotation_add_osc` would draw.
BOOL annotation_redact_add_osc(ScreenwideRegionOscVertex *vertices, NSUInteger *count,
                               NSSize size, ScreenwidePreviewSurface *surface,
                               CGFloat scale);
#endif
