// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dispatch/dispatch.h>
#include <stdint.h>

static uint64_t activeGeneration = 0;
static NSRunningApplication *previousApplication = nil;

static NSCursor *cursorForIcon(uint8_t icon) {
  switch (icon) {
  case 1: return NSCursor.crosshairCursor;
  case 2: return NSCursor.openHandCursor;
  case 3: return NSCursor.closedHandCursor;
  case 7: return NSCursor.arrowCursor;
  case 8: return NSCursor.IBeamCursor;
  case 9: return NSCursor.pointingHandCursor;
  default: return nil;
  }
}

static void applyCursorForGeneration(uint64_t generation, uint8_t icon) {
  NSCursor *cursor = cursorForIcon(icon);
  if (activeGeneration == generation && cursor != nil)
    [cursor set];
}

int screenwide_cursor_session_acquire(uint64_t generation, uint8_t icon) {
  if (!NSThread.isMainThread || activeGeneration != 0 || generation == 0 ||
      cursorForIcon(icon) == nil)
    return 0;

  NSRunningApplication *frontmost =
      NSWorkspace.sharedWorkspace.frontmostApplication;
  if (frontmost.processIdentifier != NSProcessInfo.processInfo.processIdentifier)
    previousApplication = frontmost;
  else
    previousApplication = nil;

  activeGeneration = generation;
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
  [NSApp activateIgnoringOtherApps:YES];
#pragma clang diagnostic pop
  applyCursorForGeneration(generation, icon);

  // Activation is committed by AppKit at the end of this main-loop turn.
  // Reapply only if this exact lease still owns presentation by then.
  dispatch_async(dispatch_get_main_queue(), ^{
    applyCursorForGeneration(generation, icon);
  });
  return 1;
}

int screenwide_cursor_session_update(uint64_t generation, uint8_t icon) {
  if (!NSThread.isMainThread || activeGeneration != generation ||
      cursorForIcon(icon) == nil)
    return 0;
  applyCursorForGeneration(generation, icon);
  return 1;
}

int screenwide_cursor_session_transfer(uint64_t from, uint64_t to,
                                       uint8_t icon) {
  if (!NSThread.isMainThread || activeGeneration != from || to == 0 ||
      cursorForIcon(icon) == nil)
    return 0;
  activeGeneration = to;
  applyCursorForGeneration(to, icon);
  return 1;
}

int screenwide_cursor_session_release(uint64_t generation) {
  if (!NSThread.isMainThread || activeGeneration != generation)
    return 0;

  NSRunningApplication *restore = previousApplication;
  previousApplication = nil;
  activeGeneration = 0;
  [NSCursor.arrowCursor set];

  // Respect an explicit application switch during the interaction. Only put
  // back the captured application while Screenwide still owns foreground.
  if (NSApp.active && restore != nil && !restore.terminated) {
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
    [restore activateWithOptions:NSApplicationActivateIgnoringOtherApps];
#pragma clang diagnostic pop
  }
  return 1;
}
