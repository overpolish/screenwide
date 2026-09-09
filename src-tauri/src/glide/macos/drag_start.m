// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

// Tauri's asynchronous drag request reads NSApp.currentEvent, which may have
// become an AppKit or pressure event since the click. Preserve the actual grip
// for the active press instead of letting that unrelated event move the window.
#import <AppKit/AppKit.h>
#import <objc/runtime.h>

static void (*originalDrag)(id, SEL, NSEvent *);
static NSEvent *lastDown;
static id mouseMonitor;
static id resignObserver;
static void startDrag(NSWindow *window, SEL selector, NSEvent *event) {
  // Never reuse a click from another window or after its button was released.
  // Keep a strong local reference: AppKit can deliver mouse-up during the call.
  NSEvent *down = lastDown;
  if (down && down.windowNumber == window.windowNumber &&
      (NSEvent.pressedMouseButtons & 1) != 0) {
    event = down;
  }
  originalDrag(window, selector, event);
}

void sw_glide_install_drag_start(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    Method method = class_getInstanceMethod(NSWindow.class, @selector(performWindowDragWithEvent:));
    if (!method) return;
    originalDrag = (void (*)(id, SEL, NSEvent *))method_setImplementation(method, (IMP)startDrag);
    mouseMonitor = [NSEvent addLocalMonitorForEventsMatchingMask:
      NSEventMaskLeftMouseDown | NSEventMaskLeftMouseUp handler:^NSEvent *(NSEvent *event) {
      lastDown = event.type == NSEventTypeLeftMouseDown ? event : nil;
      return event;
    }];
    resignObserver = [NSNotificationCenter.defaultCenter
      addObserverForName:NSApplicationDidResignActiveNotification object:NSApp
      queue:nil usingBlock:^(NSNotification *notification) {
        (void)notification;
        lastDown = nil;
      }];
  });
}
