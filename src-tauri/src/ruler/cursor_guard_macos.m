// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>
#import <objc/runtime.h>
#include <stdatomic.h>

static IMP original_cursor_set = NULL;
static atomic_bool screenshot_crosshair_guard_active = false;
static NSCursor *region_expected_cursor = nil;
/// A window of the tool's own chrome, where the pointer stays an ordinary
/// arrow: its controls are pressed rather than drawn on. Weak, so the guard
/// never keeps a window alive.
static __weak NSWindow *cursor_guard_exempt_window = nil;

/// Whether the pointer is over the exempt window. Its frame is in screen
/// coordinates, as `NSEvent.mouseLocation` is, and the window sits above
/// every other one the tool owns, so being inside the frame is being over it.
static BOOL pointerOverExemptWindow(void) {
  NSWindow *exempt = cursor_guard_exempt_window;
  return exempt != nil && NSPointInRect(NSEvent.mouseLocation, exempt.frame);
}

static void guarded_cursor_set(id receiver, SEL selector) {
  id applied = receiver;
  if (receiver == [NSCursor arrowCursor] && !pointerOverExemptWindow()) {
    if (region_expected_cursor != nil) {
      applied = region_expected_cursor;
    } else if (atomic_load_explicit(&screenshot_crosshair_guard_active,
                                    memory_order_relaxed)) {
      applied = [NSCursor crosshairCursor];
    }
  }
  ((void (*)(id, SEL))original_cursor_set)(applied, selector);
}

static void install_cursor_set_guard(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    Method method = class_getInstanceMethod(NSCursor.class, @selector(set));
    original_cursor_set =
        method_setImplementation(method, (IMP)guarded_cursor_set);
  });
}

void screenwide_arm_screenshot_initial_crosshair_guard(void) {
  install_cursor_set_guard();
  // WebKit continues requesting the arrow cursor throughout the screenshot
  // session, even with cursor rectangles disabled. Keep the narrow arrow-only
  // substitution active until teardown; explicit move/resize/button cursors
  // are different NSCursor instances and pass through unchanged.
  atomic_store_explicit(&screenshot_crosshair_guard_active, true,
                        memory_order_relaxed);
  region_expected_cursor = [NSCursor crosshairCursor];
}

void screenwide_disarm_screenshot_initial_crosshair_guard(void) {
  atomic_store_explicit(&screenshot_crosshair_guard_active, false,
                        memory_order_relaxed);
  region_expected_cursor = nil;
}

void screenwide_set_region_expected_cursor(NSCursor *cursor) {
  install_cursor_set_guard();
  region_expected_cursor = cursor;
}

/// Names the tool's own chrome window, or clears it with nil. Main thread
/// only, as every cursor decision is.
void screenwide_set_cursor_guard_exempt_window(NSWindow *window) {
  cursor_guard_exempt_window = window;
}
