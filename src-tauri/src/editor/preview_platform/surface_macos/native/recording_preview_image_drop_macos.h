// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_IMAGE_DROP_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_IMAGE_DROP_MACOS_H

/// What a picture dropped on the workspace keeps on the surface, beside the
/// rest of its state in `recording_preview_surface_macos_private.h`.

/// What a drop hands over: an image file's path, the path of a file the
/// dragging app wrote out for the drop, or the picture's own file data. The
/// twin of the `DROP_` kinds in `surface_macos/image_drop.rs`.
typedef NS_ENUM(uint32_t, ScreenwideImageDrop) {
  ScreenwideImageDropFile = 0,
  ScreenwideImageDropWritten = 1,
  ScreenwideImageDropData = 2,
};

/// A picture dropped on the workspace: what it is, as `kind` says, in `data`,
/// and the layer and image-normalised point it landed on, the nearest
/// picture's for a drop on the frame; `layer` -1 with no picture laid out.
typedef void (*screenwide_preview_image_drop_callback)(uint32_t kind, int32_t layer, double x,
                                                          double y, const uint8_t *data,
                                                          size_t length, void *context);

@interface ScreenwidePreviewSurface ()
@property(nonatomic) screenwide_preview_image_drop_callback imageDropCallback;
@property(nonatomic) void *imageDropContext;
@end

#endif
