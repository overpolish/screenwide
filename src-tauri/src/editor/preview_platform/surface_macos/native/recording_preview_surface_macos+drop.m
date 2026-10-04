// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#import "recording_preview_surface_macos_private.h"
#include "recording_preview_annotation_layers_macos.h"

SCREENWIDE_PREVIEW_PRIVATE void on_main_async(dispatch_block_t block);

/// The formats a picture's own data may be dropped in, the most faithful
/// first: a PNG keeps its transparency, and the format a browser was showing
/// the picture in is the picture itself. The TIFF a browser adds beside it
/// is a conversion, read only where nothing else is offered. All are formats
/// the `image` crate reads.
static NSArray<NSPasteboardType> *sticker_drop_data_types(void) {
  return @[
    NSPasteboardTypePNG, @"com.compuserve.gif", @"org.webmproject.webp", @"public.jpeg",
    NSPasteboardTypeTIFF
  ];
}

/// Only files, and only pictures: a drag of a document or a folder offers
/// nothing a sticker can show.
static NSDictionary<NSPasteboardReadingOptionKey, id> *sticker_drop_file_options(void) {
  return @{
    NSPasteboardURLReadingFileURLsOnlyKey : @YES,
    NSPasteboardURLReadingContentsConformToTypesKey : @[ @"public.image" ]
  };
}

/// Where the files a dragging app writes out for a drop are received, off
/// the main thread.
static NSOperationQueue *sticker_drop_queue(void) {
  static NSOperationQueue *queue;
  static dispatch_once_t once;
  dispatch_once(&once, ^{
    queue = [[NSOperationQueue alloc] init];
    queue.qualityOfService = NSQualityOfServiceUserInitiated;
  });
  return queue;
}

void screenwide_preview_surface_set_sticker_drop_callback(
    void *handle, screenwide_preview_sticker_drop_callback callback, void *context) {
  if (handle == NULL) return;
  ScreenwidePreviewSurface *surface = (__bridge ScreenwidePreviewSurface *)handle;
  on_main_async(^{
    surface.stickerDropCallback = callback;
    surface.stickerDropContext = context;
    if (callback == NULL) {
      [surface.interaction unregisterDraggedTypes];
      return;
    }
    NSMutableArray<NSPasteboardType> *types = [NSMutableArray arrayWithObject:NSPasteboardTypeFileURL];
    [types addObjectsFromArray:sticker_drop_data_types()];
    [types addObjectsFromArray:NSFilePromiseReceiver.readableDraggedTypes];
    [surface.interaction registerForDraggedTypes:types];
  });
}

/// The picture a press at `point` would draw on, by the rule
/// `annotation_drawing_target` keeps, and `point` in its image-normalised
/// space - beyond 0 to 1 for a drop on the frame. `NO` with no picture laid
/// out.
static BOOL sticker_drop_point(ScreenwidePreviewSurface *surface, NSPoint point, int32_t *layer,
                               double *x, double *y) {
  ScreenwidePreviewSelection target;
  if (!annotation_drawing_target(surface, point, &target)) return NO;
  NSRect image = selection_image_frame_for(surface, target);
  if (image.size.width <= 0.0 || image.size.height <= 0.0) return NO;
  *layer = (int32_t)target.layer_id;
  *x = (point.x - NSMinX(image)) / image.size.width;
  *y = (point.y - NSMinY(image)) / image.size.height;
  return YES;
}

@implementation ScreenwidePreviewInteractionView (Drop)

/// Whether the workspace takes drops now, and the drag holds a picture in a
/// form one can be read from.
- (BOOL)stickerDropAccepts:(id<NSDraggingInfo>)info {
  ScreenwidePreviewSurface *surface = self.surface;
  if (surface.stickerDropCallback == NULL || !surface.editorEnabled || surface.editorSuspended)
    return NO;
  NSPasteboard *pasteboard = info.draggingPasteboard;
  return [pasteboard canReadObjectForClasses:@[ NSURL.class ] options:sticker_drop_file_options()] ||
         [pasteboard availableTypeFromArray:sticker_drop_data_types()] != nil ||
         [pasteboard canReadObjectForClasses:@[ NSFilePromiseReceiver.class ] options:nil];
}

- (NSDragOperation)draggingEntered:(id<NSDraggingInfo>)info {
  return [self stickerDropAccepts:info] ? NSDragOperationCopy : NSDragOperationNone;
}

- (NSDragOperation)draggingUpdated:(id<NSDraggingInfo>)info {
  return [self stickerDropAccepts:info] ? NSDragOperationCopy : NSDragOperationNone;
}

- (BOOL)prepareForDragOperation:(id<NSDraggingInfo>)info {
  return [self stickerDropAccepts:info];
}

/// Hands the dropped picture over in the form that keeps the most of it: a
/// file of its own first, then its data, then a file the dragging app writes
/// out for the drop, as browsers do for an image. The picture the drop joins
/// is taken in hand, as a press with the sticker tool there would.
- (BOOL)performDragOperation:(id<NSDraggingInfo>)info {
  if (![self stickerDropAccepts:info]) return NO;
  ScreenwidePreviewSurface *surface = self.surface;
  NSPoint point = [self convertPoint:info.draggingLocation fromView:nil];
  int32_t layer = -1;
  double x = 0.5;
  double y = 0.5;
  sticker_drop_point(surface, point, &layer, &x, &y);
  annotation_take_layer_at_point(surface, point);
  NSPasteboard *pasteboard = info.draggingPasteboard;
  void (^deliver)(ScreenwideStickerDrop, const void *, size_t) =
      ^(ScreenwideStickerDrop kind, const void *bytes, size_t length) {
        surface.stickerDropCallback(kind, layer, x, y, bytes, length, surface.stickerDropContext);
      };
  NSArray<NSURL *> *files =
      [pasteboard readObjectsForClasses:@[ NSURL.class ] options:sticker_drop_file_options()];
  if (files.count > 0) {
    const char *path = files.firstObject.fileSystemRepresentation;
    deliver(ScreenwideStickerDropFile, path, strlen(path));
    return YES;
  }
  NSPasteboardType type = [pasteboard availableTypeFromArray:sticker_drop_data_types()];
  NSData *data = type != nil ? [pasteboard dataForType:type] : nil;
  if (data.length > 0) {
    deliver(ScreenwideStickerDropData, data.bytes, data.length);
    return YES;
  }
  NSFilePromiseReceiver *promise =
      [pasteboard readObjectsForClasses:@[ NSFilePromiseReceiver.class ] options:nil].firstObject;
  if (promise == nil) return NO;
  // A folder of its own, so the one file taken is the only thing in it and
  // the folder goes with it.
  NSURL *folder = [NSURL fileURLWithPath:[NSTemporaryDirectory()
                                             stringByAppendingPathComponent:NSUUID.UUID.UUIDString]
                             isDirectory:YES];
  [NSFileManager.defaultManager createDirectoryAtURL:folder
                         withIntermediateDirectories:YES
                                          attributes:nil
                                               error:nil];
  __weak ScreenwidePreviewSurface *weakSurface = surface;
  __block BOOL taken = NO;
  void (^received)(NSURL *, NSError *) = ^(NSURL *file, NSError *error) {
    if (error != nil) {
      NSLog(@"Screenwide could not receive a dropped picture: %@", error);
      return;
    }
    NSString *path = file.path;
    dispatch_async(dispatch_get_main_queue(), ^{
      // A drag may carry several files; the first that arrives is the one.
      ScreenwidePreviewSurface *live = weakSurface;
      if (taken || live == nil || live.stickerDropCallback == NULL) {
        [NSFileManager.defaultManager removeItemAtPath:path error:nil];
        return;
      }
      taken = YES;
      const char *bytes = path.fileSystemRepresentation;
      live.stickerDropCallback(ScreenwideStickerDropWritten, layer, x, y, (const uint8_t *)bytes,
                               strlen(bytes), live.stickerDropContext);
    });
  };
  [promise receivePromisedFilesAtDestination:folder
                                     options:@{}
                              operationQueue:sticker_drop_queue()
                                      reader:received];
  return YES;
}

@end
