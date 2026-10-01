// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

//! The live overlay's input: every pointer and key event over its surfaces,
//! routed to Rust and swallowed.

#import <AppKit/AppKit.h>

#import "native_overlay_macos.h"
#import "native_overlay_macos_private.h"

static ScreenwideAnnotatePointer pointerCallback = NULL;
static ScreenwideAnnotateKey keyCallback = NULL;
static id eventMonitor = nil;
/// Whether the tool in hand points with the I-beam rather than the crosshair.
static BOOL textCursor = NO;

void screenwide_annotate_set_text_cursor(uint32_t text) { textCursor = text != 0; }

static uint32_t modifiersOf(NSEvent *event) {
  NSEventModifierFlags flags = event.modifierFlags;
  uint32_t modifiers = 0;
  if ((flags & NSEventModifierFlagCommand) != 0)
    modifiers |= SCREENWIDE_ANNOTATE_MODIFIER_COMMAND;
  if ((flags & NSEventModifierFlagShift) != 0)
    modifiers |= SCREENWIDE_ANNOTATE_MODIFIER_SHIFT;
  if ((flags & NSEventModifierFlagOption) != 0)
    modifiers |= SCREENWIDE_ANNOTATE_MODIFIER_OPTION;
  if ((flags & NSEventModifierFlagControl) != 0)
    modifiers |= SCREENWIDE_ANNOTATE_MODIFIER_CONTROL;
  return modifiers;
}

/// The pointer in the space the cursor sidecar reports in: global desktop
/// points with y running down from the main display's top-left. Taken from the
/// event's CGEvent rather than converted from AppKit's y-up screen
/// coordinates, so an annotation lands where the recording says it did.
static CGPoint pointerLocation(NSEvent *event) {
  CGEventRef cg = event.CGEvent;
  return cg != NULL ? CGEventGetLocation(cg) : CGPointMake(0, 0);
}

/// Every event the overlay is offered is swallowed: there is no pass-through
/// mode, so nothing underneath can be clicked or typed into while it is up.
/// Key-ups and wheels are consumed without being reported; the shortcut and
/// Escape leave by their own global registrations.
///
/// The exception is this application's other windows. An event carrying a
/// window that is not a host belongs to the toolbar - or to a panel it opened
/// - and is handed straight back, so its controls can be pressed and, while
/// it holds key status, typed into.
static NSEvent *handleEvent(NSEvent *event) {
  if (!screenwide_annotate_has_surfaces())
    return event;
  if (event.window != nil && !screenwide_annotate_owns_window(event.window)) {
    // Chrome rather than canvas: the pointer says "press", not "draw". The
    // cursor guard exempts this window, so the arrow survives being set.
    if (event.type == NSEventTypeMouseMoved)
      [NSCursor.arrowCursor set];
    return event;
  }
  switch (event.type) {
  case NSEventTypeLeftMouseDown:
  case NSEventTypeLeftMouseDragged:
  case NSEventTypeLeftMouseUp: {
    if (pointerCallback != NULL) {
      CGPoint point = pointerLocation(event);
      uint32_t phase = event.type == NSEventTypeLeftMouseDown ? 0
                       : event.type == NSEventTypeLeftMouseDragged ? 1
                                                                   : 2;
      pointerCallback(phase, point.x, point.y);
      screenwide_annotate_redraw();
    }
    return nil;
  }
  case NSEventTypeKeyDown: {
    if (keyCallback != NULL && keyCallback(event.keyCode, modifiersOf(event)))
      screenwide_annotate_redraw();
    return nil;
  }
  case NSEventTypeMouseMoved:
    // Back over the canvas, where the tool's pointer says a stroke starts here.
    [(textCursor ? NSCursor.IBeamCursor : NSCursor.crosshairCursor) set];
    return nil;
  default:
    return nil;
  }
}

void screenwide_annotate_install_input(ScreenwideAnnotatePointer pointer,
                                       ScreenwideAnnotateKey key) {
  if (!NSThread.isMainThread)
    return;
  pointerCallback = pointer;
  keyCallback = key;
  if (eventMonitor != nil)
    return;
  NSEventMask mask = NSEventMaskLeftMouseDown | NSEventMaskLeftMouseDragged |
                     NSEventMaskLeftMouseUp | NSEventMaskRightMouseDown |
                     NSEventMaskRightMouseUp | NSEventMaskOtherMouseDown |
                     NSEventMaskOtherMouseUp | NSEventMaskMouseMoved |
                     NSEventMaskScrollWheel | NSEventMaskKeyDown |
                     NSEventMaskKeyUp;
  eventMonitor = [NSEvent
      addLocalMonitorForEventsMatchingMask:mask
                                   handler:^NSEvent *(NSEvent *event) {
                                     return handleEvent(event);
                                   }];
}

void screenwide_annotate_teardown_input(void) {
  if (!NSThread.isMainThread)
    return;
  if (eventMonitor != nil) {
    [NSEvent removeMonitor:eventMonitor];
    eventMonitor = nil;
  }
  // The annotations source stays: hosts left showing still draw from it.
  pointerCallback = NULL;
  keyCallback = NULL;
}

