// SPDX-License-Identifier: GPL-3.0-or-later

#import "screenshot_region_osc_macos_private.h"

static uint32_t display_id(NSScreen *screen) {
  return [screen.deviceDescription[@"NSScreenNumber"] unsignedIntValue];
}

static NSRect desktop_frame(NSArray<NSScreen *> *screens) {
  NSRect desktop = screens.firstObject.frame;
  for (NSScreen *screen in screens)
    desktop = NSUnionRect(desktop, screen.frame);
  return desktop;
}

static NSPoint local_origin(NSRect desktop, NSRect screen) {
  return NSMakePoint(NSMinX(screen) - NSMinX(desktop),
                     NSMaxY(desktop) - NSMaxY(screen));
}

static NSScreen *screen_for_id(NSArray<NSScreen *> *screens,
                               uint32_t identifier) {
  for (NSScreen *screen in screens)
    if (display_id(screen) == identifier)
      return screen;
  return nil;
}

static NSScreen *nearest_screen(NSArray<NSScreen *> *screens, NSRect frame) {
  NSScreen *best = nil;
  CGFloat bestOverlap = -1.0;
  CGFloat bestDistance = CGFLOAT_MAX;
  NSPoint center = NSMakePoint(NSMidX(frame), NSMidY(frame));
  for (NSScreen *screen in screens) {
    NSRect intersection = NSIntersectionRect(frame, screen.frame);
    CGFloat overlap = NSWidth(intersection) * NSHeight(intersection);
    CGFloat dx = center.x - NSMidX(screen.frame);
    CGFloat dy = center.y - NSMidY(screen.frame);
    CGFloat distance = dx * dx + dy * dy;
    if (overlap > bestOverlap ||
        (overlap == bestOverlap && distance < bestDistance)) {
      best = screen;
      bestOverlap = overlap;
      bestDistance = distance;
    }
  }
  return best ?: screens.firstObject;
}

static BOOL layout_matches(ScreenwideRegionOSC *root,
                           NSArray<NSScreen *> *screens,
                           uint32_t anchor_id) {
  if (root.displayID != anchor_id ||
      root.desktopPeers.count + 1 != screens.count)
    return NO;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    NSScreen *screen = screen_for_id(screens, surface.displayID);
    if (!screen || !NSEqualRects(surface.host.window.frame, screen.frame))
      return NO;
  }
  return YES;
}

static void discard_peers(ScreenwideRegionOSC *root) {
  for (ScreenwideRegionOSC *peer in root.desktopPeers) {
    screenwide_region_osc_cursor_release(peer);
    peer.input = NULL;
    peer.rustContext = NULL;
  }
  for (NSWindow *window in root.desktopWindows) {
    [window orderOut:nil];
    [window close];
  }
  root.desktopPeers = nil;
  root.desktopWindows = nil;
}

static NSPanel *make_panel(NSWindow *parent, NSScreen *screen) {
  // `screen:` establishes the panel's coordinate context. Passing the
  // display's global origin as the content rect can apply that origin a second
  // time on an external display, so create it screen-local and assign the
  // authoritative global frame exactly once afterwards.
  NSRect localFrame =
      NSMakeRect(0, 0, NSWidth(screen.frame), NSHeight(screen.frame));
  NSPanel *panel =
      [[NSPanel alloc] initWithContentRect:localFrame
                                styleMask:NSWindowStyleMaskBorderless |
                                          NSWindowStyleMaskNonactivatingPanel
                                  backing:NSBackingStoreBuffered
                                    defer:NO
                                   screen:screen];
  [panel setFrame:screen.frame display:NO];
  panel.animationBehavior = NSWindowAnimationBehaviorNone;
  panel.backgroundColor = NSColor.clearColor;
  panel.collectionBehavior =
      NSWindowCollectionBehaviorCanJoinAllSpaces |
      NSWindowCollectionBehaviorFullScreenAuxiliary |
      NSWindowCollectionBehaviorStationary;
  panel.hasShadow = NO;
  panel.hidesOnDeactivate = NO;
  panel.level = parent.level;
  panel.opaque = NO;
  panel.releasedWhenClosed = NO;
  panel.sharingType = NSWindowSharingNone;
  panel.acceptsMouseMovedEvents = YES;
  NSView *host =
      [[NSView alloc] initWithFrame:NSMakeRect(0, 0, NSWidth(screen.frame),
                                              NSHeight(screen.frame))];
  panel.contentView = host;
  return panel;
}

static BOOL rebuild_surfaces(ScreenwideRegionOSC *root,
                             NSArray<NSScreen *> *screens, NSRect desktop,
                             uint32_t anchor_id) {
  NSScreen *anchor = screen_for_id(screens, anchor_id);
  if (!anchor)
    return NO;
  for (ScreenwideRegionOSC *surface in screenwide_region_osc_surfaces(root)) {
    surface.gestureActive = NO;
    ScreenwideRegionMagnifier magnifier = surface.magnifier;
    magnifier.active = 0;
    surface.magnifier = magnifier;
    screenwide_region_osc_cursor_release(surface);
  }
  discard_peers(root);

  NSWindow *parent = root.host.window;
  [parent setFrame:anchor.frame display:NO];
  [parent.contentView layoutSubtreeIfNeeded];
  root.displayID = anchor_id;
  root.desktopOffset = local_origin(desktop, anchor.frame);
  root.desktopSize = desktop.size;
  root.desktopPeers = [NSMutableArray array];
  root.desktopWindows = [NSMutableArray array];

  for (NSScreen *screen in screens) {
    if (display_id(screen) == anchor_id)
      continue;
    NSPanel *panel = make_panel(parent, screen);
    if (!screenwide_region_osc_attach((__bridge void *)panel.contentView,
                                      root.rustContext, NULL, root.input,
                                      root.layoutChanged)) {
      [panel close];
      discard_peers(root);
      return NO;
    }
    ScreenwideRegionOSC *peer =
        screenwide_region_osc_for_view((__bridge void *)panel.contentView);
    peer.desktopRoot = root;
    peer.desktopOffset = local_origin(desktop, screen.frame);
    peer.desktopSize = desktop.size;
    peer.displayID = display_id(screen);
    peer.showFrame = root.showFrame;
    peer.showHandles = root.showHandles;
    peer.inputEnabled = root.inputEnabled;
    peer.exclusionRect = NSZeroRect;
    peer.rulerVisible = root.rulerVisible;
    peer.rulerCrosshair = root.rulerCrosshair;
    peer.rulerCopied = root.rulerCopied;
    peer.rulerPoint = NSMakePoint(root.rulerPoint.x - peer.desktopOffset.x,
                                 root.rulerPoint.y - peer.desktopOffset.y);
    peer.rulerColor = root.rulerColor;
    peer.rulerToleranceMode = root.rulerToleranceMode;
    peer.rulerToleranceVisible = root.rulerToleranceVisible;
    peer.rulerToleranceAnimationFrom =
        root.rulerToleranceAnimationFrom;
    peer.rulerToleranceAnimationStarted =
        root.rulerToleranceAnimationStarted;
    peer.rulerToleranceAnimationTarget =
        root.rulerToleranceAnimationTarget;
    peer.rulerMeasurements = root.rulerMeasurements;
    peer.rulerHoveredArtifactKey = root.rulerHoveredArtifactKey;
    peer.rulerHoverPulseStarted = root.rulerHoverPulseStarted;
    panel.ignoresMouseEvents = !root.inputEnabled;
    [root.desktopPeers addObject:peer];
    [root.desktopWindows addObject:panel];
    if (parent.visible) {
      panel.alphaValue = parent.alphaValue;
      [panel orderFrontRegardless];
    }
  }
  screenwide_region_osc_apply_region(root, root.desktopRegion, root.visible);
  return YES;
}

size_t screenwide_region_osc_configure_desktop(
    void *view_ptr, uint32_t anchor_id,
    ScreenwideRegionDesktopDisplay *displays, size_t capacity,
    double *desktop_width, double *desktop_height,
    uint32_t *resolved_anchor_id, int *layout_changed) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  NSArray<NSScreen *> *screens = NSScreen.screens;
  if (!root || !root.host.window || screens.count == 0 || !displays ||
      capacity == 0)
    return 0;
  NSRect desktop = desktop_frame(screens);
  NSScreen *anchor = screen_for_id(screens, anchor_id);
  if (!anchor) {
    anchor = nearest_screen(screens, root.host.window.frame);
    anchor_id = display_id(anchor);
  }
  if (!root.screenObserver && root.layoutChanged) {
    __weak ScreenwideRegionOSC *weakRoot = root;
    root.screenObserver = [NSNotificationCenter.defaultCenter
        addObserverForName:NSApplicationDidChangeScreenParametersNotification
                    object:nil
                     queue:NSOperationQueue.mainQueue
                usingBlock:^(__unused NSNotification *notification) {
                  ScreenwideRegionOSC *strongRoot = weakRoot;
                  if (strongRoot && strongRoot.layoutChanged &&
                      strongRoot.rustContext)
                    strongRoot.layoutChanged(strongRoot.rustContext);
                }];
  }
  BOOL changed = !layout_matches(root, screens, anchor_id);
  if (changed && !rebuild_surfaces(root, screens, desktop, anchor_id))
      return 0;

  root.desktopSize = desktop.size;
  if (resolved_anchor_id)
    *resolved_anchor_id = anchor_id;
  if (layout_changed)
    *layout_changed = changed;
  if (desktop_width)
    *desktop_width = NSWidth(desktop);
  if (desktop_height)
    *desktop_height = NSHeight(desktop);
  size_t count = MIN(screens.count, capacity);
  for (size_t index = 0; index < count; index++) {
    NSScreen *screen = screens[index];
    NSPoint origin = local_origin(desktop, screen.frame);
    displays[index] = (ScreenwideRegionDesktopDisplay){
        .id = display_id(screen),
        .x = origin.x,
        .y = origin.y,
        .width = NSWidth(screen.frame),
        .height = NSHeight(screen.frame),
        .scale = screen.backingScaleFactor ?: 1.0,
    };
  }
  return count;
}

void screenwide_region_osc_set_desktop_presented(void *view_ptr,
                                                  int presented) {
  ScreenwideRegionOSC *root =
      screenwide_region_osc_root(screenwide_region_osc_for_view(view_ptr));
  if (!root)
    return;
  NSWindow *parent = root.host.window;
  if (!presented) {
    screenwide_region_osc_cancel_pointer_claim(root);
    for (ScreenwideRegionOSC *surface in
         screenwide_region_osc_surfaces(root))
      screenwide_region_osc_cursor_release(surface);
  }
  for (NSWindow *window in root.desktopWindows) {
    if (presented) {
      window.level = parent.level;
      window.alphaValue = parent.alphaValue;
      [window orderFrontRegardless];
    } else {
      [window orderOut:nil];
    }
  }
  for (ScreenwideRegionOSC *surface in root.desktopPeers)
    screenwide_region_osc_draw(surface);
}
