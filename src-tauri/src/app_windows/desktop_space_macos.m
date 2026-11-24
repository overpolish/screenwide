// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <dlfcn.h>

// macOS moves a process's own windows between Spaces through these private
// SkyLight calls and refuses the same move for another process's windows.
// Each is optional: an OS update that drops one disables the move rather than
// stopping Screenwide from launching.
static int (*connection)(void);
static CFArrayRef (*displaySpaces)(int);
static CFArrayRef (*windowSpaces)(int, uint64_t, CFArrayRef);
static void (*moveWindows)(int, CFArrayRef, uint64_t);

static bool loaded(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    void *sky = dlopen("/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight", RTLD_LAZY);
    if (!sky) return;
    connection = dlsym(sky, "CGSMainConnectionID");
    displaySpaces = dlsym(sky, "CGSCopyManagedDisplaySpaces");
    windowSpaces = dlsym(sky, "CGSCopySpacesForWindows");
    moveWindows = dlsym(sky, "CGSMoveWindowsToManagedSpace");
  });
  return connection && displaySpaces && windowSpaces && moveWindows;
}

static uint64_t spaceID(NSDictionary *space) {
  return [space[@"ManagedSpaceID"] ?: space[@"id64"] unsignedLongLongValue];
}

static bool isDesktop(NSDictionary *space) {
  return [space[@"type"] intValue] == 0;
}

static NSArray<NSNumber *> *spacesOf(uint32_t windowID) {
  // 7: every Space the window is on, current or not.
  return CFBridgingRelease(windowSpaces(connection(), 7, (__bridge CFArrayRef)@[@(windowID)]));
}

// The desktop for a window on a full-screen Space: the nearest one before
// that Space on its display, where macOS returns when the app leaves full
// screen, or else the nearest after it. 0 for a window on a desktop already
// or on several Spaces.
uint64_t screenwide_desktop_beside_full_screen(uint32_t windowID) {
  @autoreleasepool {
    if (!loaded()) return 0;
    NSArray<NSNumber *> *membership = spacesOf(windowID);
    if (membership.count != 1) return 0;
    uint64_t current = membership.firstObject.unsignedLongLongValue;
    for (NSDictionary *display in CFBridgingRelease(displaySpaces(connection()))) {
      NSArray<NSDictionary *> *spaces = display[@"Spaces"];
      NSUInteger index = [spaces indexOfObjectPassingTest:^BOOL(NSDictionary *space, NSUInteger i, BOOL *stop) {
        (void)i;
        (void)stop;
        return spaceID(space) == current;
      }];
      if (index == NSNotFound) continue;
      if (isDesktop(spaces[index])) return 0;
      for (NSUInteger before = index; before > 0; --before) {
        if (isDesktop(spaces[before - 1])) return spaceID(spaces[before - 1]);
      }
      for (NSUInteger after = index + 1; after < spaces.count; ++after) {
        if (isDesktop(spaces[after])) return spaceID(spaces[after]);
      }
      return 0;
    }
    return 0;
  }
}

// Moves one of this process's windows to `space`, answering whether it is
// there now.
bool screenwide_move_own_window_to_space(uint32_t windowID, uint64_t space) {
  @autoreleasepool {
    if (!loaded()) return false;
    moveWindows(connection(), (__bridge CFArrayRef)@[@(windowID)], space);
    NSArray<NSNumber *> *membership = spacesOf(windowID);
    return membership.count == 1 && membership.firstObject.unsignedLongLongValue == space;
  }
}
