// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

#ifndef SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_TEXT_MACOS_H
#define SCREENWIDE_RECORDING_PREVIEW_ANNOTATION_TEXT_MACOS_H

/// Typing into a text box: what it keeps on the interaction view and the
/// surface, beside the rest of their state in
/// `recording_preview_surface_macos_private.h`, the text view it types in,
/// and the functions the typing's files share.

/// The typing of a box whose look sets chosen words apart, which Rust lays
/// out as the compositor draws it: `words` is the handle Rust opened the
/// box with, `font` the type size and `align` the record's alignment, and
/// every point is from the top-left of the text's block. Offsets are bytes
/// of the text's UTF-8. See `surface_macos/text_words.rs`.
void screenwide_text_words_follow(void *words, const uint8_t *text, uint32_t length);
int screenwide_text_words_sets_apart(void *words);
uint32_t screenwide_text_words_offset_at(void *words, double font, uint32_t align, double x,
                                         double y);
uint32_t screenwide_text_words_line(void *words, double font, uint32_t align, uint32_t offset,
                                    int down, double *goal);
void screenwide_text_words_free(void *words);

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
/// Where a press in the box the text view does not take began its
/// selection, and in what steps a drag from it selects: characters, words or
/// paragraphs.
@property(nonatomic) NSUInteger annotationTextSelectingFrom;
@property(nonatomic) NSSelectionGranularity annotationTextSelectingBy;
/// The box's words as Rust lays them out, where its look sets chosen words
/// apart; NULL where the text view's own layout matches what is drawn.
@property(nonatomic) void *annotationTextWords;
/// Whether what is typed into is the caption showing rather than a box, and
/// the caption layer a press opened it on, whose pane it shows on.
@property(nonatomic) BOOL annotationTextCaption;
@property(nonatomic) ScreenwidePreviewSelection annotationTextCaptionTarget;
@end

/// The view the box being typed into is typed in, its own type clear, laid
/// over the box's text; see `+annotation_text_view.m`.
@interface ScreenwideAnnotationTextView : NSTextView
@property(nonatomic, weak) ScreenwidePreviewSurface *surface;
@property(nonatomic) BOOL finishing;
@property(nonatomic) CGFloat fontPoints;
@property(nonatomic) uint32_t alignmentValue;
/// The face the box's look sets its type in; no family is the app's Inter.
@property(nonatomic, copy) NSString *faceFamily;
@property(nonatomic) uint32_t faceWeight;
@property(nonatomic) BOOL faceItalic;
/// A run of line moves through words set apart: whether one is going on,
/// where across the block it keeps to, and the end of the selection that
/// stays put as Shift extends it. Any other change of selection ends it.
@property(nonatomic) BOOL lineRun;
@property(nonatomic) BOOL settingLine;
@property(nonatomic) double lineGoal;
@property(nonatomic) NSUInteger lineAnchor;
@end

/// Opens the text view for typing `value`, set in the face `face`, `weight`
/// and `italic` name, into the box at `index` in the published grips, or the
/// caption showing where `caption`; `words` is Rust's layout of its words
/// where it lays them out, and `caret_utf8` the byte the caret starts at, or
/// -1 for where the opening press landed, or the end.
void annotation_text_open(ScreenwidePreviewSurface *surface, NSInteger index, NSString *value,
                          NSString *face, uint32_t weight, uint32_t italic, void *words,
                          BOOL caption, int64_t caret_utf8);
/// Where the caption being typed into sets its words, in the interaction
/// view's points, its type size in points and its alignment. NO where the
/// scene has not laid it out.
BOOL caption_text_place(ScreenwidePreviewSurface *surface, NSRect *block, CGFloat *points,
                        uint32_t *align);
/// Lays `view` over `block` with its type `points` large, its lines lined
/// up by `align`.
void annotation_text_lay_view(ScreenwideAnnotationTextView *view, NSRect block, CGFloat points,
                              uint32_t align);
/// Lays `view` over the caption being typed into, wherever the scene last
/// laid it out.
void caption_text_layout(ScreenwidePreviewSurface *surface, ScreenwideAnnotationTextView *view);
/// The caption showing under `point`, as the captions layer a press on it
/// reports, wherever no drawing tool is in hand. NO off it.
BOOL caption_target_at(ScreenwidePreviewSurface *surface, NSPoint point,
                       ScreenwidePreviewSelection *target);
/// A double-click on the caption showing, with the select tool or the
/// Captions tool in hand, which chooses the captions and asks Rust to open
/// the caption for typing where the press landed. YES when the press was that.
BOOL caption_text_press(ScreenwidePreviewInteractionView *view, NSEvent *event, NSPoint point);
/// The byte of the caption being typed into's text a press at `point` lands
/// at, or -1 where no caption is typed into or the press is off it.
int64_t caption_text_offset_at(ScreenwidePreviewSurface *surface, NSPoint point);
/// The word of the caption showing under `point`, on the caption layer
/// `target`, counted across its lines; -1 over none.
int32_t caption_word_at(ScreenwidePreviewSurface *surface, ScreenwidePreviewSelection target,
                        NSPoint point);
/// Whether a text box is being typed into.
BOOL annotation_text_editing(ScreenwidePreviewSurface *surface);
/// Ends the typing, telling Rust the text it ended with. Answers whether that
/// text has anything to read, which is what Rust keeps the box for.
BOOL annotation_text_finish(ScreenwidePreviewSurface *surface);
/// Tells Rust the text the box being typed into holds now, at `phase` of the
/// typing: 1 a change, 2 its end.
void annotation_text_report(ScreenwidePreviewSurface *surface, uint32_t phase);
/// Lays the text view over its box again, at the box's current place and
/// type size, or ends the typing when there is no box left to type into.
void annotation_text_layout(ScreenwidePreviewSurface *surface);
/// A press typing owns: one that ends the typing and does nothing else, and
/// a double-click on a text box, which asks Rust to open it. A press outside
/// the box being typed into ends the typing and, answering NO, carries on as
/// the press it is. YES when the press was typing's.
BOOL annotation_text_press(ScreenwidePreviewInteractionView *view, NSEvent *event);
/// The I-beam over the text being typed into, or nil anywhere else.
NSCursor *annotation_text_cursor(ScreenwidePreviewSurface *surface, NSPoint point);
/// The layer the box being typed into belongs to and its place in that
/// layer's own list: the identity a gesture reports, and the scene draws by.
void annotation_text_identity(ScreenwidePreviewSurface *surface, uint32_t *layer,
                              uint32_t *index);
/// Whether the text view does not take the presses in the box being typed
/// into, because its own layout is not what is drawn: the box lies on a
/// turned layer, where the view stands upright away from it, or sets
/// chosen words apart in their own faces.
BOOL annotation_text_routed(ScreenwidePreviewSurface *surface);
/// Whether the box being typed into sets chosen words apart, so Rust lays
/// out where its words are.
BOOL annotation_text_sets_words_apart(ScreenwidePreviewSurface *surface);
/// The block the box being typed into sets its text in, in the interaction
/// view's points, its type size in points and its alignment. NO where it
/// cannot be placed now.
BOOL annotation_text_words_place(ScreenwidePreviewSurface *surface, NSRect *block,
                                 CGFloat *points, uint32_t *align);
/// The UTF-16 index of byte `utf8` of `text`'s UTF-8, and back.
NSUInteger annotation_text_utf16_index(NSString *text, NSUInteger utf8);
uint32_t annotation_text_utf8_offset(NSString *text, NSUInteger utf16);
/// Tells the scene where the caret and selection now are, and redraws if
/// that changes them. `restart` shows the caret for a whole blink again, as
/// after every edit and move.
void annotation_text_marks_changed(ScreenwidePreviewSurface *surface, BOOL restart);
/// Stops the blink and takes the caret and selection out of the scene.
void annotation_text_marks_end(ScreenwidePreviewSurface *surface);
/// Extends the selection a press in the box the text view does not take
/// began to `point`, on the layer standing upright.
void annotation_text_drag(ScreenwidePreviewSurface *surface, NSPoint point);
/// The character a press at `point`, on the layer standing upright, lands
/// nearest in the text being typed into.
NSUInteger annotation_text_index_at(ScreenwidePreviewSurface *surface, NSPoint point);

#endif
