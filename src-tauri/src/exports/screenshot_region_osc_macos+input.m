// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#include <stddef.h>

_Static_assert(sizeof(NativeOscResult) == 40, "NativeOscResult ABI size drift");
_Static_assert(offsetof(NativeOscResult, x) == 8,
               "NativeOscResult ABI offset drift");

static BOOL pointInRectTopLeft(NSPoint p, NSRect r) {
  return !NSIsEmptyRect(r) && NSPointInRect(p, r);
}
static uint32_t edgesForHandle(uint8_t handle) {
  switch (handle) {
  case 2: return 4;      // north / top
  case 3: return 8;      // south / bottom
  case 4: return 2;      // east / right
  case 5: return 1;      // west / left
  case 6: return 2 | 4;  // north-east
  case 7: return 1 | 4;  // north-west
  case 8: return 2 | 8;  // south-east
  case 9: return 1 | 8;  // south-west
  default: return 0;
  }
}
static void applyCursor(NativeOscResult result) {
  NSCursor *value = [NSCursor arrowCursor];
  switch (result.cursor) {
  case 1:
    value = [NSCursor crosshairCursor];
    break;
  case 2:
    value = [NSCursor openHandCursor];
    break;
  case 3:
    value = [NSCursor closedHandCursor];
    break;
  case 4:
  case 5:
  case 6:
    value = screenwide_region_resize_cursor(edgesForHandle(result.handle));
    if (value == nil) value = [NSCursor crosshairCursor];
    break;
  case 7:
    value = [NSCursor arrowCursor];
    break;
  case 8:
    value = [NSCursor IBeamCursor];
    break;
  case 9:
    value = [NSCursor pointingHandCursor];
    break;
  default:
    break;
  }
  screenwide_set_region_expected_cursor(value);
  [value set];
}
static void setCursorHidden(ScreenwideRegionOSC *s, BOOL hidden) {
  if (s.cursorHidden == hidden) return;
  s.cursorHidden = hidden;
  if (hidden)
    [NSCursor hide];
  else
    [NSCursor unhide];
}
static BOOL updateMagnifier(ScreenwideRegionOSC *s, NativeOscResult result,
                            NSPoint point, uint32_t phase) {
  BOOL visible = s == screenwide_region_osc_root(s) && phase == 3 &&
                 result.gesture == 3 &&
                 result.has_region != 0 && result.handle != 0;
  if (!visible) {
    BOOL changed = s.magnifier.active != 0;
    ScreenwideRegionMagnifier magnifier = s.magnifier;
    magnifier.active = 0;
    s.magnifier = magnifier;
    return changed;
  }
  uint32_t edges = edgesForHandle(result.handle);
  NSRect frame = NSMakeRect(result.x, result.y, result.width, result.height);
  NSPoint anchor = screenwide_region_magnifier_anchor(point, frame, edges);
  CGFloat scale = s.host.window.backingScaleFactor ?: 1.0;
  NSString *appearance = [s.host.effectiveAppearance
      bestMatchFromAppearancesWithNames:@[ NSAppearanceNameAqua,
                                           NSAppearanceNameDarkAqua ]];
  uint32_t lightMode =
      [appearance isEqualToString:NSAppearanceNameAqua] ? 1 : 0;
  s.magnifier = screenwide_region_magnifier_make(
      anchor, scale, edges, lightMode, 0, 0, 0,
      anchor.x / MAX(s.host.bounds.size.width, 1.0),
      anchor.y / MAX(s.host.bounds.size.height, 1.0), 0, 0, 1, 1);
  return YES;
}
void screenwide_region_osc_cursor_claim(ScreenwideRegionOSC *s) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  if (s.host.window == nil)
    return;
  ScreenwideRegionOSC *previous = root.cursorOwner;
  if (previous != s && previous.cursorRectsDisabled) {
    NSWindow *window = previous.host.window;
    if (window != nil) {
      [window enableCursorRects];
      [window resetCursorRects];
    }
    previous.cursorRectsDisabled = NO;
  }
  root.cursorOwner = s;
  // Match the mature preview OSC: while native input is active, WebKit cursor
  // rectangles must not race the cursor chosen by the native interaction path.
  if (!s.cursorRectsDisabled) {
    [s.host.window disableCursorRects];
    s.cursorRectsDisabled = YES;
  }
  screenwide_set_region_expected_cursor([NSCursor crosshairCursor]);
  [[NSCursor crosshairCursor] set];
}
void screenwide_region_osc_cursor_release(ScreenwideRegionOSC *s) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  NSWindow *window = s.host.window;
  if (s.cursorRectsDisabled && window != nil) {
    [window enableCursorRects];
    [window resetCursorRects];
  }
  s.cursorRectsDisabled = NO;
  if (root.cursorOwner == s) {
    root.cursorOwner = nil;
    screenwide_set_region_expected_cursor(nil);
    // Clearing the guard does not change the cursor already applied by
    // AppKit. Restore it synchronously so teardown never waits for movement.
    [[NSCursor arrowCursor] set];
  }
}

void screenwide_region_osc_cancel_pointer_claim(ScreenwideRegionOSC *s) {
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  if (root)
    root.cursorClaimGeneration += 1;
}

static void claimPointerSurfaceNow(ScreenwideRegionOSC *root,
                                   uint64_t generation) {
  if (!root)
    return;
  NSPoint pointer = NSEvent.mouseLocation;
  if (root.cursorClaimGeneration != generation || !root.visible ||
      !root.inputEnabled)
    return;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    NSWindow *window = surface.host.window;
    if (surface.inputEnabled && window.visible &&
        NSPointInRect(pointer, window.frame)) {
      screenwide_region_osc_cursor_claim(surface);
      return;
    }
  }
}

void screenwide_region_osc_claim_pointer_surface(void *view_ptr) {
  ScreenwideRegionOSC *s = screenwide_region_osc_for_view(view_ptr);
  if (!s)
    return;
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  uint64_t generation = ++root.cursorClaimGeneration;

  // Keep the shortcut feeling immediate when AppKit has already ordered the
  // panel, then win the final cursor update performed at the end of that
  // presentation turn. The generation guard makes the deferred block inert
  // after hide, input teardown, or a newer presentation request.
  claimPointerSurfaceNow(root, generation);
  dispatch_async(dispatch_get_main_queue(), ^{
    claimPointerSurfaceNow(root, generation);
  });
}
static void processInput(ScreenwideRegionOSC *s, NSEvent *event,
                         uint32_t phase) {
  if (!s.inputEnabled || !s.input || !s.host.window)
    return;
  if (event.window != s.host.window) {
    if (!s.gestureActive)
      screenwide_region_osc_cursor_release(s);
    return;
  }
  screenwide_region_osc_cursor_claim(s);
  NSPoint point = [s.host convertPoint:event.locationInWindow fromView:nil];
  if (!s.host.isFlipped)
    point.y = NSHeight(s.host.bounds) - point.y;
  if (screenwide_region_osc_ocr_control_input(s, point, phase))
    return;
  if ((phase == 3 || phase == 4) && !s.gestureActive)
    return;
  if (phase == 2 && s.ocrCancelVisible)
    screenwide_region_osc_ocr_set_cancel_visible(
        (__bridge void *)s.host, 0);
  if (phase == 2 && pointInRectTopLeft(point, s.exclusionRect))
    return;
  NativeOscResult result = {0};
  NSPoint desktopPoint =
      NSMakePoint(point.x + s.desktopOffset.x, point.y + s.desktopOffset.y);
  uint8_t modifiers = 0;
  if ((event.modifierFlags & NSEventModifierFlagShift) != 0)
    modifiers |= 1;
  if ((event.modifierFlags &
       (NSEventModifierFlagCommand | NSEventModifierFlagControl)) != 0)
    modifiers |= 2;
  if (event.clickCount >= 2)
    modifiers |= 4;
  s.input(s.rustContext, phase, desktopPoint.x, desktopPoint.y, modifiers,
          &result);
  if (result.status == 255)
    return;
  if (phase == 2)
    s.gestureActive = YES;
  if (phase == 4 || phase == 5)
    s.gestureActive = NO;
  if (result.cursor && screenwide_region_osc_root(s).inputEnabled)
    applyCursor(result);
  NativeOscResult localResult = result;
  localResult.x -= s.desktopOffset.x;
  localResult.y -= s.desktopOffset.y;
  BOOL magnifierChanged = updateMagnifier(s, localResult, point, phase);
  setCursorHidden(s, s.magnifier.active != 0);
  if (result.status == 1 || result.status == 2 || result.status == 3) {
    screenwide_region_osc_apply_region(
        s,
        result.has_region
            ? NSMakeRect(result.x, result.y, result.width, result.height)
            : NSZeroRect,
        screenwide_region_osc_root(s).visible);
  } else if (magnifierChanged)
    screenwide_region_osc_draw(s);
}

static void processKeyboardCommand(ScreenwideRegionOSC *s, uint32_t phase) {
  if (!s.inputEnabled || !s.input || !s.rustContext)
    return;
  NativeOscResult result = {0};
  s.input(s.rustContext, phase, 0, 0, 0, &result);
  if (result.cursor && screenwide_region_osc_root(s).inputEnabled)
    applyCursor(result);
  if (result.status == 1 || result.status == 2 || result.status == 3) {
    screenwide_region_osc_apply_region(
        s,
        result.has_region
            ? NSMakeRect(result.x, result.y, result.width, result.height)
            : NSZeroRect,
        screenwide_region_osc_root(s).visible);
  }
}

void screenwide_region_osc_input_install(ScreenwideRegionOSC *s) {
  __weak ScreenwideRegionOSC *weak = s;
  NSEventMask mask = NSEventMaskMouseMoved | NSEventMaskLeftMouseDown |
                     NSEventMaskLeftMouseDragged | NSEventMaskLeftMouseUp |
                     NSEventMaskKeyDown;
  s.eventMonitor = [NSEvent
      addLocalMonitorForEventsMatchingMask:mask
                                   handler:^NSEvent *(NSEvent *event) {
                                     ScreenwideRegionOSC *strong = weak;
                                     if (!strong)
                                       return event;
                                     if (event.type == NSEventTypeKeyDown) {
                                       if (event.window == strong.host.window &&
                                           strong.inputEnabled) {
                                         BOOL command =
                                             (event.modifierFlags &
                                              (NSEventModifierFlagCommand |
                                               NSEventModifierFlagControl)) != 0;
                                         if (strong.ocrPhase == 2 && command &&
                                             (event.keyCode == 0 ||
                                              event.keyCode == 8)) {
                                           processKeyboardCommand(
                                               strong,
                                               event.keyCode == 0 ? 6 : 7);
                                           // Tao force-delivers Command key-up
                                           // directly to the key window. Let
                                           // the passive webview receive the
                                           // matching key-down as well.
                                           return event;
                                         }
                                       }
                                       return event;
                                     }
                                     uint32_t phase =
                                         event.type == NSEventTypeMouseMoved ? 1
                                         : event.type ==
                                                 NSEventTypeLeftMouseDown
                                             ? 2
                                         : event.type ==
                                                 NSEventTypeLeftMouseDragged
                                             ? 3
                                             : 4;
                                     processInput(strong, event, phase);
                                     return event;
                                   }];
}

void screenwide_region_osc_input_teardown(ScreenwideRegionOSC *s) {
  if (s.gestureActive && s.input && s.rustContext) {
    NativeOscResult result = {0};
    s.input(s.rustContext, 5, 0, 0, 0, &result);
    s.gestureActive = NO;
  }
  if (s.eventMonitor) {
    [NSEvent removeMonitor:s.eventMonitor];
    s.eventMonitor = nil;
  }
  setCursorHidden(s, NO);
  screenwide_region_osc_cursor_release(s);
}
