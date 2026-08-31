// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"
#include <math.h>
#include <stddef.h>

_Static_assert(sizeof(NativeOscResult) == 48, "NativeOscResult ABI size drift");
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
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  NSPoint desktopPoint =
      NSMakePoint(point.x + s.desktopOffset.x, point.y + s.desktopOffset.y);
  if (root.rulerLabelDragActive && (phase == 3 || phase == 4)) {
    NativeOscResult dragResult = {0};
    native_osc_ruler_label_input(s.rustContext, phase == 3 ? 2 : 3,
                                 0, 0, desktopPoint.x, desktopPoint.y,
                                 0, 0, &dragResult);
    if (phase == 4)
      root.rulerLabelDragActive = NO;
    if (dragResult.status != 255) {
      screenwide_region_osc_apply_ruler_result(s, dragResult);
      if (dragResult.cursor && root.inputEnabled)
        applyCursor(dragResult);
    }
    return;
  }
  if ((phase == 1 || phase == 2) && s.rulerVisible &&
      root.rulerGuideKeyCode == 0) {
    ScreenwideRulerLabelHit hit = {0};
    if (screenwide_region_osc_ruler_label_hit(s, point, &hit)) {
      if (phase == 2) {
        NativeOscResult beginResult = {0};
        NSPoint center =
            NSMakePoint(hit.center.x + s.desktopOffset.x,
                        hit.center.y + s.desktopOffset.y);
        native_osc_ruler_label_input(
            s.rustContext, 1, hit.kind, hit.id,
            desktopPoint.x, desktopPoint.y, center.x, center.y,
            &beginResult);
        if (beginResult.status != 255) {
          root.rulerLabelDragActive = YES;
          screenwide_region_osc_apply_ruler_result(s, beginResult);
          if (beginResult.cursor && root.inputEnabled)
            applyCursor(beginResult);
        }
        return;
      }
      NativeOscResult labelResult = {0};
      native_osc_ruler_label_input(s.rustContext, 7, hit.kind, hit.id,
                                   desktopPoint.x, desktopPoint.y,
                                   0, 0, &labelResult);
      if (labelResult.status != 255) {
        screenwide_region_osc_apply_ruler_result(s, labelResult);
        if (labelResult.cursor && root.inputEnabled)
          applyCursor(labelResult);
      }
      return;
    }
  }
  if ((phase == 3 || phase == 4) && !s.gestureActive) {
    // Mouse-dragged events arrive after a rejected Region-editor press. Each
    // event temporarily claims native cursor ownership at the top of this
    // function, so release it again instead of leaving a drawing crosshair.
    screenwide_region_osc_cursor_release(s);
    return;
  }
  if (phase == 2 && s.ocrCancelVisible)
    screenwide_region_osc_ocr_set_cancel_visible(
        (__bridge void *)s.host, 0);
  if (phase == 2 && pointInRectTopLeft(point, s.exclusionRect))
    return;
  NativeOscResult result = {0};
  uint8_t modifiers = 0;
  if ((event.modifierFlags & NSEventModifierFlagShift) != 0)
    modifiers |= 1;
  if ((event.modifierFlags &
       (NSEventModifierFlagCommand | NSEventModifierFlagControl)) != 0)
    modifiers |= 2;
  if (event.clickCount >= 2)
    modifiers |= 4;
  if ((event.modifierFlags & NSEventModifierFlagOption) != 0)
    modifiers |= 8;
  s.input(s.rustContext, phase, desktopPoint.x, desktopPoint.y, modifiers,
          &result);
  if (result.status == 255) {
    // A non-drawing Region editor rejects presses outside its committed
    // region. The temporary native claim above must not leave the crosshair
    // behind when no gesture accepted that press.
    if (phase == 2)
      screenwide_region_osc_cursor_release(s);
    return;
  }
  screenwide_region_osc_apply_ruler_result(s, result);
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
  if ((result.ruler_flags & 1) == 0 &&
      (result.status == 1 || result.status == 2 || result.status == 3)) {
    screenwide_region_osc_apply_region(
        s,
        result.has_region
            ? NSMakeRect(result.x, result.y, result.width, result.height)
            : NSZeroRect,
        screenwide_region_osc_root(s).visible);
  } else if (magnifierChanged)
    screenwide_region_osc_draw(s);
}

static NSPoint eventPoint(ScreenwideRegionOSC *surface, NSEvent *event);

static BOOL processKeyboardCommand(ScreenwideRegionOSC *s, uint32_t phase) {
  if (!s.inputEnabled || !s.input || !s.rustContext)
    return NO;
  NativeOscResult result = {0};
  s.input(s.rustContext, phase, 0, 0, 0, &result);
  if (result.status == 255)
    return NO;
  screenwide_region_osc_apply_ruler_result(s, result);
  if (result.cursor && screenwide_region_osc_root(s).inputEnabled)
    applyCursor(result);
  if ((result.ruler_flags & 1) == 0 &&
      (result.status == 1 || result.status == 2 || result.status == 3)) {
    screenwide_region_osc_apply_region(
        s,
        result.has_region
            ? NSMakeRect(result.x, result.y, result.width, result.height)
            : NSZeroRect,
        screenwide_region_osc_root(s).visible);
  }
  return YES;
}

static BOOL processRulerLabelRightClick(ScreenwideRegionOSC *s,
                                        NSEvent *event) {
  if (!s.inputEnabled || !s.rustContext || !s.rulerVisible ||
      event.window != s.host.window)
    return NO;
  NSPoint point = eventPoint(s, event);
  NSPoint desktopPoint =
      NSMakePoint(point.x + s.desktopOffset.x,
                  point.y + s.desktopOffset.y);
  ScreenwideRulerLabelHit hit = {0};
  BOOL labelHit =
      screenwide_region_osc_ruler_label_hit(s, point, &hit);
  NativeOscResult result = {0};
  native_osc_ruler_label_input(
      s.rustContext, labelHit ? 5 : 6,
      labelHit ? hit.kind : 0, labelHit ? hit.id : 0,
      desktopPoint.x, desktopPoint.y, 0, 0, &result);
  if (result.status == 255)
    return NO;
  screenwide_region_osc_apply_ruler_result(s, result);
  if (result.cursor && screenwide_region_osc_root(s).inputEnabled)
    applyCursor(result);
  return YES;
}

static NSPoint eventPoint(ScreenwideRegionOSC *surface, NSEvent *event) {
  NSPoint point = [surface.host convertPoint:event.locationInWindow
                                    fromView:nil];
  if (!surface.host.isFlipped)
    point.y = NSHeight(surface.host.bounds) - point.y;
  return point;
}

void screenwide_region_osc_ruler_refresh_pointer(void *view_ptr) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root || !root.inputEnabled || !root.input || !root.visible)
    return;
  NSPoint screen = NSEvent.mouseLocation;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    NSWindow *window = surface.host.window;
    if (!window.visible || !NSPointInRect(screen, window.frame))
      continue;
    NSPoint windowPoint = [window convertPointFromScreen:screen];
    NSPoint point = [surface.host convertPoint:windowPoint fromView:nil];
    if (!surface.host.isFlipped)
      point.y = NSHeight(surface.host.bounds) - point.y;
    NativeOscResult result = {0};
    root.input(root.rustContext, 1,
               point.x + surface.desktopOffset.x,
               point.y + surface.desktopOffset.y, 0, &result);
    if (result.status == 255)
      return;
    screenwide_region_osc_apply_ruler_result(surface, result);
    if (result.cursor)
      applyCursor(result);
    return;
  }
}

static BOOL processRulerViewportInput(ScreenwideRegionOSC *surface,
                                      NSEvent *event, uint32_t operation,
                                      NSPoint delta) {
  if (!surface.inputEnabled || !surface.rustContext ||
      event.window != surface.host.window)
    return NO;
  NSPoint anchor = eventPoint(surface, event);
  NativeOscResult result = {0};
  if (!native_osc_ruler_viewport_input(
          surface.rustContext, surface.displayID, operation,
          anchor.x, anchor.y, delta.x, delta.y, &result))
    return NO;
  screenwide_region_osc_cursor_claim(surface);
  screenwide_region_osc_apply_ruler_result(surface, result);
  if (result.cursor)
    applyCursor(result);
  return YES;
}

void screenwide_region_osc_input_install(ScreenwideRegionOSC *s) {
  __weak ScreenwideRegionOSC *weak = s;
  NSEventMask mask = NSEventMaskMouseMoved | NSEventMaskLeftMouseDown |
                     NSEventMaskLeftMouseDragged | NSEventMaskLeftMouseUp |
                     NSEventMaskRightMouseDown |
                     NSEventMaskOtherMouseDown |
                     NSEventMaskOtherMouseDragged |
                     NSEventMaskOtherMouseUp | NSEventMaskScrollWheel |
                     NSEventMaskMagnify | NSEventMaskKeyDown |
                     NSEventMaskKeyUp | NSEventMaskFlagsChanged;
  s.eventMonitor = [NSEvent
      addLocalMonitorForEventsMatchingMask:mask
                                   handler:^NSEvent *(NSEvent *event) {
                                     ScreenwideRegionOSC *strong = weak;
                                     if (!strong)
                                       return event;
                                     if (event.type == NSEventTypeFlagsChanged) {
                                       ScreenwideRegionOSC *root =
                                           screenwide_region_osc_root(strong);
                                       if (strong == root && root.visible &&
                                           root.inputEnabled && root.input &&
                                           root.rustContext) {
                                         NativeOscResult result = {0};
                                         uint8_t modifiers =
                                             (event.modifierFlags &
                                              NSEventModifierFlagOption) != 0
                                                 ? 8
                                                 : 0;
                                         root.input(root.rustContext, 30,
                                                    0, 0, modifiers,
                                                    &result);
                                         if (result.status != 255) {
                                           screenwide_region_osc_apply_ruler_result(
                                               root, result);
                                           if (result.cursor)
                                             applyCursor(result);
                                         }
                                       }
                                       return event;
                                     }
                                     if (event.type == NSEventTypeKeyUp) {
                                       ScreenwideRegionOSC *root =
                                           screenwide_region_osc_root(strong);
                                       if (strong == root &&
                                           root.rulerRangeKeyCode != 0 &&
                                           event.keyCode ==
                                               root.rulerRangeKeyCode) {
                                         root.rulerRangeKeyCode = 0;
                                         if (root.visible && root.inputEnabled)
                                           processKeyboardCommand(strong, 22);
                                         return nil;
                                       }
                                       if (strong == root &&
                                           root.rulerGuideKeyCode != 0 &&
                                           event.keyCode ==
                                               root.rulerGuideKeyCode) {
                                         root.rulerGuideKeyCode = 0;
                                         if (root.visible && root.inputEnabled)
                                           processKeyboardCommand(strong, 28);
                                         return nil;
                                       }
                                       if (strong == root &&
                                           root.rulerRadiusKeyCode != 0 &&
                                           event.keyCode ==
                                               root.rulerRadiusKeyCode) {
                                         root.rulerRadiusKeyCode = 0;
                                         if (root.visible && root.inputEnabled)
                                           processKeyboardCommand(strong, 32);
                                         return nil;
                                       }
                                       return event;
                                     }
                                     if (event.type == NSEventTypeKeyDown) {
                                       ScreenwideRegionOSC *root =
                                           screenwide_region_osc_root(strong);
                                       if (strong == root && root.visible &&
                                           root.inputEnabled) {
                                         BOOL command =
                                             (event.modifierFlags &
                                              (NSEventModifierFlagCommand |
                                               NSEventModifierFlagControl)) != 0;
                                         BOOL shift =
                                             (event.modifierFlags &
                                              NSEventModifierFlagShift) != 0;
                                         if (!command &&
                                             root.rulerRangeKeyCode != 0 &&
                                             event.keyCode ==
                                                 root.rulerRangeKeyCode)
                                           return nil;
                                         if (!command &&
                                             root.rulerGuideKeyCode != 0 &&
                                             event.keyCode ==
                                                 root.rulerGuideKeyCode)
                                           return nil;
                                         if (!command &&
                                             root.rulerRadiusKeyCode != 0 &&
                                             event.keyCode ==
                                                 root.rulerRadiusKeyCode)
                                           return nil;
                                         uint32_t rulerPhase = 0;
                                         if (!command && event.keyCode == 7)
                                           rulerPhase = 13;
                                         else if (!command && event.keyCode == 48)
                                           rulerPhase = 14;
                                         else if (!command &&
                                                  (event.keyCode == 51 ||
                                                   event.keyCode == 117))
                                           rulerPhase = 16;
                                         else if (command && event.keyCode == 8)
                                           rulerPhase = 17;
                                         else if (command && event.keyCode == 6)
                                           rulerPhase = shift ? 19 : 18;
                                         else if (command && event.keyCode == 16)
                                           rulerPhase = 19;
                                         else if (!command && !event.isARepeat &&
                                                  event.keyCode == 17)
                                           rulerPhase = 29;
                                         else if (!command && !event.isARepeat &&
                                                  event.keyCode == 46)
                                           rulerPhase = 33;
                                         else if (!command && !event.isARepeat &&
                                                  root.rulerRangeKeyCode == 0 &&
                                                  root.rulerGuideKeyCode == 0 &&
                                                  root.rulerRadiusKeyCode == 0 &&
                                                  (event.keyCode == 18 ||
                                                   event.keyCode == 19))
                                           rulerPhase = event.keyCode == 18
                                               ? 20
                                               : 21;
                                         else if (!command && !event.isARepeat &&
                                                  root.rulerRangeKeyCode == 0 &&
                                                  root.rulerGuideKeyCode == 0 &&
                                                  root.rulerRadiusKeyCode == 0 &&
                                                  (event.keyCode == 9 ||
                                                   event.keyCode == 4))
                                           rulerPhase = event.keyCode == 9
                                               ? 26
                                               : 27;
                                         else if (!command && !event.isARepeat &&
                                                  root.rulerRangeKeyCode == 0 &&
                                                  root.rulerGuideKeyCode == 0 &&
                                                  root.rulerRadiusKeyCode == 0 &&
                                                  event.keyCode == 15)
                                           rulerPhase = 31;
                                         if (rulerPhase != 0 &&
                                             processKeyboardCommand(
                                                 strong, rulerPhase)) {
                                           if (rulerPhase == 20 ||
                                               rulerPhase == 21)
                                             root.rulerRangeKeyCode =
                                                 event.keyCode;
                                           else if (rulerPhase == 26 ||
                                                    rulerPhase == 27)
                                             root.rulerGuideKeyCode =
                                                 event.keyCode;
                                           else if (rulerPhase == 31)
                                             root.rulerRadiusKeyCode =
                                                 event.keyCode;
                                           return nil;
                                         }
                                       }
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
                                     if (event.type == NSEventTypeRightMouseDown) {
                                       if (processRulerLabelRightClick(strong,
                                                                       event))
                                         return nil;
                                       return event;
                                     }
                                     if (event.type == NSEventTypeScrollWheel) {
                                       BOOL zoom =
                                           (event.modifierFlags &
                                            NSEventModifierFlagControl) != 0;
                                       NSPoint delta = zoom
                                           ? NSMakePoint(
                                                 exp(event.scrollingDeltaY *
                                                     0.01),
                                                 0.0)
                                           : NSMakePoint(event.scrollingDeltaX,
                                                         event.scrollingDeltaY);
                                       if (processRulerViewportInput(
                                               strong, event, zoom ? 1 : 2,
                                               delta))
                                         return nil;
                                       return event;
                                     }
                                     if (event.type == NSEventTypeMagnify) {
                                       if (processRulerViewportInput(
                                               strong, event, 1,
                                               NSMakePoint(
                                                   exp(event.magnification),
                                                   0.0)))
                                         return nil;
                                       return event;
                                     }
                                     if (event.type == NSEventTypeOtherMouseDown &&
                                         event.buttonNumber == 2) {
                                       if (event.window == strong.host.window &&
                                           strong.inputEnabled) {
                                         strong.rulerPanActive = YES;
                                         strong.rulerPanLastPoint =
                                             eventPoint(strong, event);
                                         screenwide_region_osc_cursor_claim(strong);
                                         return nil;
                                       }
                                       return event;
                                     }
                                     if (event.type == NSEventTypeOtherMouseDragged &&
                                         event.buttonNumber == 2 &&
                                         strong.rulerPanActive) {
                                       NSPoint point = eventPoint(strong, event);
                                       NSPoint delta = NSMakePoint(
                                           point.x - strong.rulerPanLastPoint.x,
                                           point.y - strong.rulerPanLastPoint.y);
                                       strong.rulerPanLastPoint = point;
                                       if (processRulerViewportInput(
                                               strong, event, 2, delta))
                                         return nil;
                                       return event;
                                     }
                                     if (event.type == NSEventTypeOtherMouseUp &&
                                         event.buttonNumber == 2 &&
                                         strong.rulerPanActive) {
                                       strong.rulerPanActive = NO;
                                       return nil;
                                     }
                                     if (event.type == NSEventTypeLeftMouseDown &&
                                         event.clickCount >= 2 &&
                                         processRulerViewportInput(
                                             strong, event, 3, NSZeroPoint))
                                       return nil;
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
  s.rulerPanActive = NO;
  ScreenwideRegionOSC *root = screenwide_region_osc_root(s);
  if (root == s && s.rulerLabelDragActive && s.rustContext) {
    NativeOscResult label = {0};
    native_osc_ruler_label_input(s.rustContext, 4, 0, 0,
                                 0, 0, 0, 0, &label);
    s.rulerLabelDragActive = NO;
  }
  if (root == s && s.rulerRangeKeyCode != 0 && s.input &&
      s.rustContext) {
    NativeOscResult range = {0};
    s.input(s.rustContext, 23, 0, 0, 0, &range);
    s.rulerRangeKeyCode = 0;
  }
  if (root == s && s.rulerGuideKeyCode != 0 && s.input &&
      s.rustContext) {
    NativeOscResult guide = {0};
    s.input(s.rustContext, 28, 0, 0, 0, &guide);
    s.rulerGuideKeyCode = 0;
  }
  if (root == s && s.rulerRadiusKeyCode != 0 && s.input &&
      s.rustContext) {
    NativeOscResult radius = {0};
    s.input(s.rustContext, 32, 0, 0, 0, &radius);
    s.rulerRadiusKeyCode = 0;
  }
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
