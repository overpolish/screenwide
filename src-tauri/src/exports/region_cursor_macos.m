// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "osc_gpu_macos.h"

NSCursor *screenwide_region_resize_cursor(uint32_t edges) {
  if (@available(macOS 15.0, *)) {
    NSCursorFrameResizePosition position = 0;
    if (edges & 1) position |= NSCursorFrameResizePositionLeft;
    if (edges & 2) position |= NSCursorFrameResizePositionRight;
    if (edges & 4) position |= NSCursorFrameResizePositionTop;
    if (edges & 8) position |= NSCursorFrameResizePositionBottom;
    if (position != 0)
      return [NSCursor frameResizeCursorFromPosition:position
                                        inDirections:NSCursorFrameResizeDirectionsAll];
  }
  static NSCursor *nwse = nil;
  static NSCursor *nesw = nil;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    NSImage *descending = [[NSImage alloc] initWithSize:NSMakeSize(16, 16)];
    [descending lockFocus];
    [[NSColor whiteColor] setStroke];
    NSBezierPath *outline = [NSBezierPath bezierPath];
    [outline setLineWidth:3.0];
    [outline moveToPoint:NSMakePoint(2, 14)];
    [outline lineToPoint:NSMakePoint(14, 2)];
    [outline moveToPoint:NSMakePoint(2, 14)];
    [outline lineToPoint:NSMakePoint(2, 9)];
    [outline moveToPoint:NSMakePoint(2, 14)];
    [outline lineToPoint:NSMakePoint(7, 14)];
    [outline moveToPoint:NSMakePoint(14, 2)];
    [outline lineToPoint:NSMakePoint(9, 2)];
    [outline moveToPoint:NSMakePoint(14, 2)];
    [outline lineToPoint:NSMakePoint(14, 7)];
    [outline stroke];
    [[NSColor blackColor] setStroke];
    NSBezierPath *line = [NSBezierPath bezierPath];
    [line setLineWidth:1.0];
    [line moveToPoint:NSMakePoint(2, 14)];
    [line lineToPoint:NSMakePoint(14, 2)];
    [line moveToPoint:NSMakePoint(2, 14)];
    [line lineToPoint:NSMakePoint(2, 9)];
    [line moveToPoint:NSMakePoint(2, 14)];
    [line lineToPoint:NSMakePoint(7, 14)];
    [line moveToPoint:NSMakePoint(14, 2)];
    [line lineToPoint:NSMakePoint(9, 2)];
    [line moveToPoint:NSMakePoint(14, 2)];
    [line lineToPoint:NSMakePoint(14, 7)];
    [line stroke];
    [descending unlockFocus];
    nwse = [[NSCursor alloc] initWithImage:descending
                                  hotSpot:NSMakePoint(8, 8)];

    NSImage *ascending = [[NSImage alloc] initWithSize:NSMakeSize(16, 16)];
    [ascending lockFocus];
    [[NSColor whiteColor] setStroke];
    NSBezierPath *outline2 = [NSBezierPath bezierPath];
    [outline2 setLineWidth:3.0];
    [outline2 moveToPoint:NSMakePoint(2, 2)];
    [outline2 lineToPoint:NSMakePoint(14, 14)];
    [outline2 moveToPoint:NSMakePoint(2, 2)];
    [outline2 lineToPoint:NSMakePoint(2, 7)];
    [outline2 moveToPoint:NSMakePoint(2, 2)];
    [outline2 lineToPoint:NSMakePoint(7, 2)];
    [outline2 moveToPoint:NSMakePoint(14, 14)];
    [outline2 lineToPoint:NSMakePoint(9, 14)];
    [outline2 moveToPoint:NSMakePoint(14, 14)];
    [outline2 lineToPoint:NSMakePoint(14, 9)];
    [outline2 stroke];
    [[NSColor blackColor] setStroke];
    NSBezierPath *line2 = [NSBezierPath bezierPath];
    [line2 setLineWidth:1.0];
    [line2 moveToPoint:NSMakePoint(2, 2)];
    [line2 lineToPoint:NSMakePoint(14, 14)];
    [line2 moveToPoint:NSMakePoint(2, 2)];
    [line2 lineToPoint:NSMakePoint(2, 7)];
    [line2 moveToPoint:NSMakePoint(2, 2)];
    [line2 lineToPoint:NSMakePoint(7, 2)];
    [line2 moveToPoint:NSMakePoint(14, 14)];
    [line2 lineToPoint:NSMakePoint(9, 14)];
    [line2 moveToPoint:NSMakePoint(14, 14)];
    [line2 lineToPoint:NSMakePoint(14, 9)];
    [line2 stroke];
    [ascending unlockFocus];
    nesw = [[NSCursor alloc] initWithImage:ascending
                                  hotSpot:NSMakePoint(8, 8)];
  });
  if (edges == (1 | 4) || edges == (2 | 8)) return nwse;
  if (edges == (2 | 4) || edges == (1 | 8)) return nesw;
  if (edges == 1 || edges == 2) return [NSCursor resizeLeftRightCursor];
  if (edges == 4 || edges == 8) return [NSCursor resizeUpDownCursor];
  return nil;
}
