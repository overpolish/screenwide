// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "target.h"
#import <unistd.h>

// Native titlebar hold + Space navigation lets WindowServer carry the live
// window. Sending it away with the membership API first breaks that drag.
static const int64_t eventTag = 0x5357474c494445;

static void mouseEvent(CGEventSourceRef source, CGEventType type, CGPoint point) {
  CGEventRef event = CGEventCreateMouseEvent(source, type, point, kCGMouseButtonLeft);
  CGEventSetFlags(event, 0);
  CGEventSetIntegerValueField(event, kCGEventSourceUserData, eventTag);
  CGEventSetIntegerValueField(event, kCGMouseEventClickState, 1);
  CGEventPost(kCGHIDEventTap, event);
  CFRelease(event);
}

static void switchSpace(CGEventSourceRef source, bool right) {
  CGKeyCode arrow = right ? 124 : 123;
  for (int down = 1; down >= 0; --down) {
    CGEventRef event = CGEventCreateKeyboardEvent(source, arrow, down);
    // HID system-state events are required for Mission Control on Tahoe.
    // Do not inherit the development shortcut's Shift/Option modifiers.
    CGEventSetFlags(event, kCGEventFlagMaskControl | kCGEventFlagMaskSecondaryFn);
    CGEventSetIntegerValueField(event, kCGEventSourceUserData, eventTag);
    CGEventPost(kCGHIDEventTap, event);
    CFRelease(event);
  }
}

static NSDictionary *displayGroup(NSString *identifier) {
  for (NSDictionary *group in sw_glide_space_displays()) {
    if ([group[@"Display Identifier"] isEqual:identifier]) return group;
  }
  return nil;
}

static uint64_t spaceID(NSDictionary *space) {
  return [space[@"ManagedSpaceID"] ?: space[@"id64"] unsignedLongLongValue];
}

static NSInteger indexOf(NSDictionary *group, uint64_t sid) {
  NSInteger index = 0;
  for (NSDictionary *space in group[@"Spaces"]) {
    if (spaceID(space) == sid) return index;
    ++index;
  }
  return NSNotFound;
}

static bool waitForSwitch(NSString *display, uint64_t previous, SWGlideCancelled cancelled, const void *context) {
  CFAbsoluteTime deadline = CFAbsoluteTimeGetCurrent() + 2.0;
  CFAbsoluteTime settled = 0;
  while (CFAbsoluteTimeGetCurrent() < deadline) {
    if (cancelled(context)) return false;
    NSDictionary *group = displayGroup(display);
    if (!group) return false;
    bool arrived = spaceID(group[@"Current Space"]) != previous && !sw_glide_space_animating(display);
    if (!arrived) settled = 0;
    else if (!settled) settled = CFAbsoluteTimeGetCurrent();
    else if (CFAbsoluteTimeGetCurrent() - settled >= 0.05) return true;
    usleep(10000);
  }
  return false;
}

int sw_glide_space_carry(void *raw, const char *identifier, uint64_t destination,
  double x, double y, SWGlideCancelled cancelled, const void *context) {
  @autoreleasepool {
    SWGlideSpaceTarget *target = (__bridge SWGlideSpaceTarget *)raw;
    NSString *display = [NSString stringWithUTF8String:identifier];
    CGPoint anchor = CGPointMake(x, y);
    if (CGEventSourceButtonState(kCGEventSourceStateHIDSystemState, kCGMouseButtonLeft) ||
        !sw_glide_space_valid_grip(target, anchor)) return 1;
    NSDictionary *initial = displayGroup(display);
    NSInteger from = indexOf(initial, spaceID(initial[@"Current Space"]));
    NSInteger to = indexOf(initial, destination);
    if (!initial || from == NSNotFound || to == NSNotFound || from == to ||
        sw_glide_space_animating(display)) return 1;
    CGEventSourceRef source = CGEventSourceCreate(kCGEventSourceStateHIDSystemState);
    if (!source) return 1;
    bool held = false;
    int result = 2;
    @try {
      // The Glide session already owns cursor hiding and pinning. Using its
      // exact anchor avoids a second warp and competing visibility counts.
      if (cancelled(context)) return 2;
      mouseEvent(source, kCGEventLeftMouseDown, anchor);
      held = true;
      // Allow the application to enter its native window drag loop before
      // Mission Control starts the slide. No initial displacement is needed.
      mouseEvent(source, kCGEventLeftMouseDragged, anchor);
      usleep(50000);
      for (unsigned step = 0; step < 16; ++step) {
        if (cancelled(context)) break;
        NSDictionary *group = displayGroup(display);
        uint64_t current = spaceID(group[@"Current Space"]);
        NSInteger currentIndex = indexOf(group, current), targetIndex = indexOf(group, destination);
        if (!group || currentIndex == NSNotFound || targetIndex == NSNotFound) break;
        if (current == destination) { result = 0; break; }
        // Re-resolve live order. Stop if the user or Mission Control moved us
        // in the opposite direction; never chase a changing desktop layout.
        if ((to > from) != (targetIndex > currentIndex)) break;
        switchSpace(source, targetIndex > currentIndex);
        if (!waitForSwitch(display, current, cancelled, context)) break;
      }
    } @catch (NSException *exception) {
      result = 2;
    } @finally {
      if (held) mouseEvent(source, kCGEventLeftMouseUp, anchor);
      // The mouse-up belongs at the held titlebar. Restore the cursor only
      // after that event has been delivered, or AppKit can see a final jump.
      if (held) usleep(20000);
      CFRelease(source);
    }
    return result;
  }
}
