// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_GROUP_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_GROUP_MACOS_H

/// What several annotations chosen together, and the marquee that chooses
/// them, keep on the interaction view and the surface, beside the rest of
/// their state in `recording_preview_surface_macos_private.h`.

@interface ScreenwidePreviewInteractionView ()
/// What the press under way does if it never travels, and what it becomes if
/// it does: an ordinary press, a toggle that only adds or removes an
/// annotation on release, a carry of every annotation chosen, or a marquee.
@property(nonatomic) uint32_t annotationPressKind;
/// The layer a group carry's samples, or a marquee band's corners, are
/// measured on.
@property(nonatomic) int32_t annotationDragLayer;
@end

@interface ScreenwidePreviewSurface ()
/// The boxes drawn round the annotations chosen together, as
/// `ScreenwideAnnotationGroupBox` records; empty with fewer than two chosen.
@property(nonatomic, strong) NSData *annotationGroupBoxes;
/// The marquee band being drawn, in this view's coordinates, or the zero
/// rect while there is none.
@property(nonatomic) NSRect annotationMarquee;
@end

#endif
