// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import <AppKit/AppKit.h>
#import <ApplicationServices/ApplicationServices.h>

@interface SWGlideSpaceTarget : NSObject
@property uint32_t windowID;
@property pid_t pid;
@property AXUIElementRef ax;
@property(strong) NSWindow *own;
@end
NSArray *sw_glide_space_displays(void);
bool sw_glide_space_animating(NSString *display);
bool sw_glide_space_valid_grip(SWGlideSpaceTarget *target, CGPoint point);
typedef bool (*SWGlideCancelled)(const void *context);
