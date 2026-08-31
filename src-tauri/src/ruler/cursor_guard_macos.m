// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>
#import <objc/runtime.h>
#include <stdatomic.h>

static IMP original_cursor_set = NULL;
static atomic_bool screenshot_crosshair_guard_active = false;
static NSCursor *region_expected_cursor = nil;

static void guarded_cursor_set(id receiver, SEL selector) {
  id applied = receiver;
  if (receiver == [NSCursor arrowCursor]) {
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
