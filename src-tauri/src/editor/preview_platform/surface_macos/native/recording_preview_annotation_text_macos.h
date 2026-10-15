// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_TEXT_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_TEXT_MACOS_H

/// What typing into a text box keeps on the interaction view and the surface,
/// beside the rest of their state in `recording_preview_surface_macos_private.h`.

@interface ScreenwidePreviewInteractionView ()
/// A press typing took - one that only ended the typing, or opened a box -
/// whose drag and release belong to nothing else.
@property(nonatomic) BOOL annotationPressIgnored;
@end

@interface ScreenwidePreviewSurface ()
/// The native text view laid over the box being typed into, its place in the
/// published grips, how many changes it has reported, and where they go. No
/// view means nothing is being typed.
@property(nonatomic, strong) NSTextView *annotationTextView;
@property(nonatomic) NSInteger annotationTextIndex;
@property(nonatomic) uint64_t annotationTextRevision;
@property(nonatomic) screenwide_preview_annotation_text_callback annotationTextCallback;
@property(nonatomic) void *annotationTextContext;
/// Where the double-click that opened the box landed, for the caret to start
/// there rather than at the end.
@property(nonatomic) NSPoint annotationTextOpenPoint;
@property(nonatomic) BOOL annotationTextOpensAtPoint;
/// The caret the compositor draws into the box, which blinks on this timer,
/// and whether it shows now. The text view's own caret and selection are
/// clear, so they never sit flat over a turned layer.
@property(nonatomic, strong) NSTimer *annotationTextBlink;
@property(nonatomic) BOOL annotationTextCaretOn;
/// Where a press in the box on a turned layer began its selection, and in
/// what steps a drag from it selects: characters, words or paragraphs.
@property(nonatomic) NSUInteger annotationTextSelectingFrom;
@property(nonatomic) NSSelectionGranularity annotationTextSelectingBy;
@end

#endif
