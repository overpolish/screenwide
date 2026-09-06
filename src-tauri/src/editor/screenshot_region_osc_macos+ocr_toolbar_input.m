// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

static ScreenwideOscControlUpdate update_for_phase(
    ScreenwideRegionOSC *surface, NSPoint point, uint32_t phase) {
  if (phase == 1 || phase == 3)
    return screenwide_osc_control_group_hover(
        surface.ocrToolbarControls, point.x, point.y);
  if (phase == 2)
    return screenwide_osc_control_group_down(
        surface.ocrToolbarControls, point.x, point.y);
  if (phase == 4)
    return screenwide_osc_control_group_up(
        surface.ocrToolbarControls, point.x, point.y);
  return screenwide_osc_control_group_clear_hover(
      surface.ocrToolbarControls);
}

static void schedule_close_timeout(ScreenwideRegionOSC *surface,
                                   uint64_t revision) {
  __weak ScreenwideRegionOSC *weak = surface;
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, 2 * NSEC_PER_SEC),
                 dispatch_get_main_queue(), ^{
                   ScreenwideRegionOSC *strong = weak;
                   if (!strong || !strong.ocrToolbarCloseArmed ||
                       strong.ocrToolbarCloseRevision != revision)
                     return;
                   ScreenwideOscConfirmUpdate update =
                       screenwide_osc_confirm_expire(strong.ocrToolbarConfirm);
                   screenwide_region_osc_ocr_toolbar_apply_confirm_update(
                       strong, update);
                 });
}

static BOOL toolbar_input(ScreenwideRegionOSC *surface, NSPoint point,
                          uint32_t phase) {
  if (!surface.ocrToolbarVisible || !surface.ocrToolbarControls)
    return NO;
  ScreenwideOscControlUpdate update =
      update_for_phase(surface, point, phase);
  screenwide_region_osc_ocr_toolbar_apply_update(surface, update);
  if (update.consumed) {
    screenwide_set_region_expected_cursor(NSCursor.pointingHandCursor);
    [NSCursor.pointingHandCursor set];
  }
  if (update.activated == 4) {
    ScreenwideOscConfirmUpdate confirm =
        screenwide_osc_confirm_press(surface.ocrToolbarConfirm);
    screenwide_region_osc_ocr_toolbar_apply_confirm_update(surface, confirm);
    if (confirm.armed) {
      uint64_t revision = ++surface.ocrToolbarCloseRevision;
      schedule_close_timeout(surface, revision);
    } else {
      surface.ocrToolbarCloseRevision += 1;
    }
    if (confirm.confirmed && surface.input && surface.rustContext) {
      NativeOscResult result = {0};
      surface.input(surface.rustContext, 12, 0, 0, 0, &result);
    }
  } else if (update.activated && surface.input && surface.rustContext) {
    NativeOscResult result = {0};
    surface.input(surface.rustContext, 8 + update.activated, 0, 0, 0,
                  &result);
  }
  return update.consumed != 0;
}

BOOL screenwide_region_osc_ocr_control_input(ScreenwideRegionOSC *surface,
                                             NSPoint point, uint32_t phase) {
  return toolbar_input(surface, point, phase) ||
         screenwide_region_osc_ocr_cancel_input(surface, point, phase);
}
