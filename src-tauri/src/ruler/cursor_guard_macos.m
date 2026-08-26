// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>
#import <objc/runtime.h>
#include <stdatomic.h>

static IMP original_hidden_until_mouse_moves = NULL;
static atomic_bool range_active = false;

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

void screenwide_set_ruler_cursor_range_active(int active) {
  install_cursor_guard();
  atomic_store_explicit(&range_active, active != 0, memory_order_relaxed);
  if (active) [NSCursor setHiddenUntilMouseMoves:NO];
}
