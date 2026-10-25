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

/// Whether the pointer is over the exempt window where it takes clicks. Its
/// frame is in screen coordinates, as `NSEvent.mouseLocation` is, and the
/// window sits above every other one the tool owns. A window larger than
/// what it draws, as the island's is, lets clicks through everywhere but its
/// parts, so over the rest of its frame the pointer is still the tool's.
static BOOL pointerOverExemptWindow(void) {
  NSWindow *exempt = cursor_guard_exempt_window;
  return exempt != nil && !exempt.ignoresMouseEvents &&
         NSPointInRect(NSEvent.mouseLocation, exempt.frame);
}

BOOL screenwide_cursor_guard_over_exempt_window(void) {
  return pointerOverExemptWindow();
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

/// The window that was key when the pointer came onto the exempt window,
/// which it is given back to when the pointer leaves.
static __weak NSWindow *exempt_handed_from = nil;

/// Lets the exempt window's page follow the pointer while it is over it.
/// WebKit updates hover only in a key window, and pointer moves go to the
/// key window, which is the tool's overlay, so the page would otherwise
/// never see the pointer cross it, and a control it last saw under the
/// pointer would keep its hover. The exempt window takes key status as the
/// pointer comes onto it and gives it back as the pointer leaves; it leaves
/// past a margin round the window's parts, inside the window, so the page
/// sees the pointer go before the overlay is key again. The tool's keys
/// still reach it either way: its event monitor sees every key the
/// application is sent. Main thread only.
void screenwide_cursor_guard_follow_pointer(void) {
  NSWindow *exempt = cursor_guard_exempt_window;
  if (exempt == nil)
    return;
  BOOL over = pointerOverExemptWindow();
  if (over && !exempt.isKeyWindow) {
    exempt_handed_from = NSApp.keyWindow;
    [exempt makeKeyWindow];
  } else if (!over && exempt.isKeyWindow) {
    NSWindow *previous = exempt_handed_from;
    exempt_handed_from = nil;
    [previous makeKeyWindow];
  }
}
