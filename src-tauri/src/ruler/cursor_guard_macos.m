// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>
#import <objc/runtime.h>
#include <stdatomic.h>

static IMP original_hidden_until_mouse_moves = NULL;
static IMP original_cursor_set = NULL;
static atomic_bool range_active = false;
static atomic_bool ruler_cursor_hidden = false;
static atomic_bool screenshot_initial_crosshair_guard_active = false;
static atomic_ulong screenshot_initial_crosshair_guard_generation = 0;

static void guarded_hidden_until_mouse_moves(id receiver, SEL selector,
                                             BOOL flag) {
  if (flag && atomic_load_explicit(&range_active, memory_order_relaxed)) return;
  ((void (*)(id, SEL, BOOL))original_hidden_until_mouse_moves)(receiver,
                                                               selector, flag);
}

static void install_cursor_guard(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    Method method = class_getClassMethod(
        NSCursor.class, @selector(setHiddenUntilMouseMoves:));
    original_hidden_until_mouse_moves = method_setImplementation(
        method, (IMP)guarded_hidden_until_mouse_moves);
  });
}

static void guarded_cursor_set(id receiver, SEL selector) {
  id applied = receiver;
  if (receiver == [NSCursor arrowCursor] &&
      atomic_load_explicit(&screenshot_initial_crosshair_guard_active,
                           memory_order_relaxed)) {
    applied = [NSCursor crosshairCursor];
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

void screenwide_set_ruler_cursor_range_active(int active) {
  install_cursor_guard();
  atomic_store_explicit(&range_active, active != 0, memory_order_relaxed);
  if (active) [NSCursor setHiddenUntilMouseMoves:NO];
}

void screenwide_set_ruler_cursor_visible(int visible) {
  bool hidden = visible == 0;
  bool previous = atomic_exchange_explicit(
      &ruler_cursor_hidden, hidden, memory_order_relaxed);
  if (hidden == previous) return;
  if (hidden) {
    [NSCursor hide];
  } else {
    [NSCursor unhide];
  }
}

void screenwide_arm_screenshot_initial_crosshair_guard(void) {
  install_cursor_set_guard();
  unsigned long generation = atomic_fetch_add_explicit(
                                 &screenshot_initial_crosshair_guard_generation,
                                 1, memory_order_relaxed) +
                             1;
  atomic_store_explicit(&screenshot_initial_crosshair_guard_active, true,
                        memory_order_relaxed);
  dispatch_after(
      dispatch_time(DISPATCH_TIME_NOW, 250 * NSEC_PER_MSEC),
      dispatch_get_main_queue(), ^{
        if (atomic_load_explicit(
                &screenshot_initial_crosshair_guard_generation,
                memory_order_relaxed) != generation)
          return;
        atomic_store_explicit(&screenshot_initial_crosshair_guard_active, false,
                              memory_order_relaxed);
      });
}

void screenwide_disarm_screenshot_initial_crosshair_guard(void) {
  atomic_store_explicit(&screenshot_initial_crosshair_guard_active, false,
                        memory_order_relaxed);
  atomic_fetch_add_explicit(&screenshot_initial_crosshair_guard_generation, 1,
                            memory_order_relaxed);
}
