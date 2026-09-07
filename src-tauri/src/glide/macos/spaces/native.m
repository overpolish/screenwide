// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>
#import <dlfcn.h>
#import <objc/message.h>

// All private entry points are optional: an OS update must disable this
// capability rather than prevent Screenwide from launching.
static int (*connection)(void);
static CFArrayRef (*displaySpaces)(int);
static CFArrayRef (*windowSpaces)(int, uint64_t, CFArrayRef);
static bool (*displayIsAnimating)(int, CFStringRef);
static int (*axWindowID)(AXUIElementRef, uint32_t *);
static void loadAPI(void) {
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    void *sky = dlopen("/System/Library/PrivateFrameworks/SkyLight.framework/SkyLight", RTLD_LAZY);
    if (!sky) return;
    connection = dlsym(sky, "CGSMainConnectionID");
    displaySpaces = dlsym(sky, "CGSCopyManagedDisplaySpaces");
    windowSpaces = dlsym(sky, "CGSCopySpacesForWindows");
    displayIsAnimating = dlsym(sky, "SLSManagedDisplayIsAnimating");
    axWindowID = dlsym(RTLD_DEFAULT, "_AXUIElementGetWindow");
  });
}

#import "target.h"
@implementation SWGlideSpaceTarget
- (void)dealloc { if (_ax) CFRelease(_ax); }
@end

// The gesture has already resolved its exact window at the titlebar anchor.
// Never replace it by whichever application happens to be focused later.
void *sw_glide_space_target(void *ax, void *own) {
  @autoreleasepool {
    loadAPI();
    if (!connection || !displaySpaces || !windowSpaces || !axWindowID || !displayIsAnimating) return NULL;
    SWGlideSpaceTarget *target = [SWGlideSpaceTarget new];
    if (ax) {
      target.ax = (AXUIElementRef)CFRetain(ax);
      AXUIElementSetMessagingTimeout(target.ax, 0.1);
      pid_t pid = 0; uint32_t wid = 0;
      AXUIElementGetPid(target.ax, &pid);
      target.pid = pid;
      if (axWindowID(target.ax, &wid) == 0) target.windowID = wid;
    } else if (own) {
      dispatch_sync(dispatch_get_main_queue(), ^{
        target.own = (__bridge NSWindow *)own;
        target.windowID = (uint32_t)target.own.windowNumber;
        target.pid = getpid();
      });
    }
    return target.windowID ? (__bridge_retained void *)target : NULL;
  }
}

void sw_glide_space_release(void *raw) { CFBridgingRelease(raw); }

char *sw_glide_space_snapshot(void *raw) {
  @autoreleasepool {
    SWGlideSpaceTarget *target = (__bridge SWGlideSpaceTarget *)raw;
    NSArray *displays = CFBridgingRelease(displaySpaces(connection()));
    NSArray *membership = CFBridgingRelease(windowSpaces(connection(), 7,
      (__bridge CFArrayRef)@[@(target.windowID)]));
    if (!displays || !membership) return NULL;
    NSMutableArray *groups = [NSMutableArray array];
    for (NSDictionary *display in displays) {
      NSMutableArray *desktops = [NSMutableArray array];
      for (NSDictionary *space in display[@"Spaces"]) {
        NSNumber *sid = space[@"ManagedSpaceID"] ?: space[@"id64"];
        if (!sid || !space[@"type"]) return NULL;
        [desktops addObject:@{@"id":sid.stringValue, @"regular":([space[@"type"] intValue] == 0 ? @YES : @NO)}];
      }
      NSDictionary *current = display[@"Current Space"];
      NSNumber *sid = current[@"ManagedSpaceID"] ?: current[@"id64"];
      NSString *identifier = display[@"Display Identifier"];
      if (!sid || !identifier) return NULL;
      [groups addObject:@{@"id":identifier, @"current":sid.stringValue, @"desktops":desktops,
        @"transitioning":(displayIsAnimating(connection(), (__bridge CFStringRef)identifier) ? @YES : @NO)}];
    }
    NSMutableArray *ids = [NSMutableArray array];
    for (NSNumber *sid in membership) [ids addObject:sid.stringValue];
    NSData *json = [NSJSONSerialization dataWithJSONObject:@{@"groups":groups, @"membership":ids}
      options:0 error:nil];
    return json ? strndup(json.bytes, json.length) : NULL;
  }
}

// Worker-thread reads used by the native carry loop; no UI mutation here.
NSArray *sw_glide_space_displays(void) {
  return CFBridgingRelease(displaySpaces(connection()));
}
bool sw_glide_space_animating(NSString *display) {
  return displayIsAnimating(connection(), (__bridge CFStringRef)display);
}

// The same panel pool serves every display group. Preload only as many panels
// as the largest group needs, without resolving or focusing a target window.
size_t sw_glide_space_preview_count(void) {
  @autoreleasepool {
    loadAPI();
    if (!connection || !displaySpaces) return 0;
    size_t maximum = 0;
    for (NSDictionary *display in sw_glide_space_displays()) {
      size_t count = 0;
      for (NSDictionary *space in display[@"Spaces"]) {
        if ([space[@"type"] intValue] == 0) ++count;
      }
      if (count > maximum) maximum = count;
    }
    return maximum;
  }
}
