// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <objc/runtime.h>
#import <dispatch/dispatch.h>

// Select from the live stack, not application activation history: the user
// may have rearranged windows since opening this workspace.
static NSRunningApplication *applicationBehind(NSWindow *window) {
  NSArray *windows = CFBridgingRelease(CGWindowListCopyWindowInfo(
      kCGWindowListOptionOnScreenBelowWindow | kCGWindowListExcludeDesktopElements,
      (CGWindowID)window.windowNumber));
  for (NSDictionary *info in windows) {
    if ([info[(id)kCGWindowLayer] integerValue] != 0 ||
        [info[(id)kCGWindowAlpha] doubleValue] <= 0)
      continue;
    pid_t pid = [info[(id)kCGWindowOwnerPID] intValue];
    // Another Screenwide workspace is already next; normal AppKit selection
    // should keep focus in this app rather than skipping over that workspace.
    if (pid == NSProcessInfo.processInfo.processIdentifier)
      return nil;
    NSRunningApplication *app = [NSRunningApplication runningApplicationWithProcessIdentifier:pid];
    if (app && !app.terminated && app.activationPolicy != NSApplicationActivationPolicyProhibited)
      return app;
  }
  return nil;
}

typedef void (*DismissCompletion)(void *, bool);
static char pendingDismissalKey;

@interface ScreenwidePendingDismissal : NSObject
@property(nonatomic, weak) NSWindow *window;
@property(nonatomic, strong) id observer;
@property(nonatomic) void *context;
@property(nonatomic) DismissCompletion completion;
@property(nonatomic) BOOL finished;
- (void)finish:(BOOL)hide;
@end

@implementation ScreenwidePendingDismissal
- (void)finish:(BOOL)hide {
  if (self.finished) return;
  self.finished = YES;
  if (self.observer)
    [NSNotificationCenter.defaultCenter removeObserver:self.observer];
  self.observer = nil;
  NSWindow *window = self.window;
  objc_setAssociatedObject(window, &pendingDismissalKey, nil, OBJC_ASSOCIATION_RETAIN_NONATOMIC);
  if (hide && window) {
    [window resignKeyWindow];
    [window resignMainWindow];
    [window orderOut:nil];
  }
  self.completion(self.context, hide && window != nil);
}
@end

void screenwide_cancel_pending_dismissal(NSWindow *window) {
  ScreenwidePendingDismissal *pending = objc_getAssociatedObject(window, &pendingDismissalKey);
  [pending finish:NO];
}

void screenwide_dismiss_window(NSWindow *window, void *context, DismissCompletion completion) {
  screenwide_cancel_pending_dismissal(window);
  ScreenwidePendingDismissal *pending = [ScreenwidePendingDismissal new];
  pending.window = window;
  pending.context = context;
  pending.completion = completion;
  objc_setAssociatedObject(window, &pendingDismissalKey, pending, OBJC_ASSOCIATION_RETAIN_NONATOMIC);

  NSRunningApplication *next = nil;
  if (NSApp.active && window.visible && (window.keyWindow || window.mainWindow))
    next = applicationBehind(window);
  if (!next) {
    [pending finish:YES];
    return;
  }

  // Observe before requesting activation. Do not order out until AppKit has
  // actually deactivated; requesting activation/deactivation is not a barrier.
  __weak ScreenwidePendingDismissal *weakPending = pending;
  pending.observer = [NSNotificationCenter.defaultCenter
      addObserverForName:NSApplicationDidResignActiveNotification object:NSApp queue:nil
      usingBlock:^(NSNotification *note) {
    (void)note;
    ScreenwidePendingDismissal *current = weakPending;
    if (current && !NSApp.active) [current finish:YES];
  }];
#pragma clang diagnostic push
#pragma clang diagnostic ignored "-Wdeprecated-declarations"
  BOOL accepted = [next activateWithOptions:NSApplicationActivateIgnoringOtherApps];
#pragma clang diagnostic pop
  if (!accepted) {
    [pending finish:NO];
    return;
  }
  // Bounded cleanup if macOS refuses the handoff. Never hide while still
  // active as a timeout fallback: that would reintroduce the ordering bug.
  dispatch_after(dispatch_time(DISPATCH_TIME_NOW, NSEC_PER_SEC), dispatch_get_main_queue(), ^{
    if (!pending.finished) [pending finish:!NSApp.active];
  });
}
