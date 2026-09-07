// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "target.h"

bool sw_glide_space_valid_grip(SWGlideSpaceTarget *target, CGPoint point) {
  // The shared titlebar resolver already accepted this exact anchor. This
  // final check excludes interactive controls before synthesizing a hold.
  if (target.own) {
    __block bool valid = false;
    dispatch_sync(dispatch_get_main_queue(), ^{
      NSWindow *window = target.own;
      CGPoint cocoa = CGPointMake(point.x, CGDisplayBounds(CGMainDisplayID()).size.height - point.y);
      NSRect layout = [window convertRectToScreen:window.contentLayoutRect];
      valid = window.visible && window.movable && NSPointInRect(cocoa, window.frame) && cocoa.y > NSMaxY(layout);
    });
    return valid;
  }
  if (!target.ax) return false;
  // Query the captured application, so Glide's noninteractive preview above
  // the anchor cannot become the hit target halfway through this gesture.
  AXUIElementRef app = AXUIElementCreateApplication(target.pid), hit = NULL;
  AXUIElementSetMessagingTimeout(app, 0.1);
  AXError error = AXUIElementCopyElementAtPosition(app, point.x, point.y, &hit);
  CFRelease(app);
  if (error != kAXErrorSuccess || !hit) return false;
  NSSet *controls = [NSSet setWithArray:@[@"AXButton", @"AXTextField", @"AXTextArea", @"AXTab",
    @"AXLink", @"AXCheckBox", @"AXRadioButton", @"AXSlider", @"AXComboBox", @"AXMenuButton", @"AXPopUpButton"]];
  bool valid = false;
  for (unsigned hop = 0; hit && hop < 8; ++hop) {
    if (CFEqual(hit, target.ax)) { valid = true; break; }
    CFTypeRef role = NULL, parent = NULL;
    AXUIElementCopyAttributeValue(hit, kAXRoleAttribute, &role);
    bool control = role && [controls containsObject:(__bridge id)role];
    if (role) CFRelease(role);
    if (control) break;
    AXUIElementCopyAttributeValue(hit, kAXParentAttribute, &parent);
    CFRelease(hit);
    hit = (AXUIElementRef)parent;
  }
  if (hit) CFRelease(hit);
  return valid;
}
