// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Drawn annotations carried by an editor layer.
 *
 * Points are in the source's own pixel space, the same space the crop is
 * expressed in, so an annotation stays glued to the picture while the frame is
 * moved, resized or re-cropped. The compositor draws them, which is what
 * keeps the editor's preview and the exported PNG the same image. This is the
 * twin of `src-tauri/src/editor/annotations/model.rs`.
 */

import type {
  AnnotationAlign,
  AnnotationHead,
  AnnotationRedaction,
} from "../../../components/shared/annotation-style/types";

/** A point in the layer source's pixel space. */
export type AnnotationPoint = { x: number; y: number };

export type AnnotationStyle = {
  /** How a text box lines up its lines; the other kinds carry the default. */
  align: AnnotationAlign;
  /** Whether a spotlight also blurs what lies outside it; the other kinds
   * carry `false`. */
  blur: boolean;
  /** `#rrggbb` or `#rrggbbaa`, straight alpha. */
  color: string;
  /** Whether a highlight is drawn as a marker stroke by hand rather than as a
   * clean band, or a shape's outline as a pen stroke that misses its own
   * start; the other kinds carry `false`. */
  handDrawn: boolean;
  head: AnnotationHead;
  /** Whether a highlight is laid by hand over the box its drag spans, in
   * strokes `width` tall, rather than fitted to the text under it; the other
   * kinds carry `false`. */
  manual: boolean;
  /** A redaction's, a shape's, a spotlight's, a magnifier's or a sticker's
   * corner radius, as a percentage of its box's shorter side from 0 to 50;
   * the other kinds carry zero. */
  radius: number;
  /** How a redaction covers what is under it; the other kinds carry the
   * default. */
  redaction: AnnotationRedaction;
  /** Whether a magnifier's loupe or a sticker casts a shadow onto the
   * picture; the other kinds carry `false`. */
  shadow: boolean;
  /** How far a spotlight's edge fades from lit to dim, as a percentage of its
   * box's shorter side from 0 to 50; the other kinds carry zero. */
  softness: number;
  /** A blurred redaction's strength, a step from 1 to 5; the other kinds
   * carry zero. */
  strength: number;
  /** Whether a highlight tints what it covers rather than recolouring the
   * page it read; the other kinds carry `false`. */
  tint: boolean;
  /** Stroke width, disc diameter, type size, pixelation block, a shape's pen
   * or a highlight's marker, in points of the capture. */
  width: number;
};

/** A quadratic Bézier from `start` to `end`, bent by `control`. */
export type AnnotationArrow = {
  control: AnnotationPoint;
  end: AnnotationPoint;
  kind: "arrow";
  start: AnnotationPoint;
};

/**
 * A numbered disc with a pin's curved tail. `value` is the annotation's place
 * in the document's counter order, which the editor keeps contiguous, and
 * `angle` is where the tail points, in radians clockwise from east in the
 * source's own pixel space - so a fresh counter's zero points right.
 */
export type AnnotationCounter = {
  angle: number;
  center: AnnotationPoint;
  kind: "counter";
  value: number;
};

/**
 * Lines of type in a solid box. `origin` is the box's top-left corner and
 * `pointer` the pointer drawn out of it, held against the box. The box's size
 * follows from the text and the type size.
 */
export type AnnotationText = {
  kind: "text";
  origin: AnnotationPoint;
  pointer: TextPointer;
  text: string;
};

/**
 * A box that hides what is under it. `start` is its top-left corner and `end`
 * its bottom-right, in source pixels; `seed` generates a pixelated box's
 * blocks.
 */
type AnnotationRedact = {
  end: AnnotationPoint;
  kind: "redact";
  seed: number;
  start: AnnotationPoint;
};

/**
 * An outline round a box, drawn with a round pen. `start` is its top-left
 * corner and `end` its bottom-right, in source pixels; the style's radius
 * rounds its corners, so a square rounded all the way is a circle. `seed` is
 * its hand-drawn stroke's wobble. The twin of `AnnotationShape::Shape` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationOutline = {
  end: AnnotationPoint;
  kind: "shape";
  seed: number;
  start: AnnotationPoint;
};

/**
 * A box left bright while everything around it dims. `start` is its top-left
 * corner and `end` its bottom-right, in source pixels; the style rounds its
 * corners, fades its edge and says whether what is outside it is blurred
 * too. The twin of `AnnotationShape::Spotlight` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationSpotlight = {
  end: AnnotationPoint;
  kind: "spotlight";
  start: AnnotationPoint;
};

/**
 * A line drawn freehand: the points the hand passed through, in source
 * pixels, thinned as it was drawn. `smooth` fits the line loosely enough that
 * a wobbly curve comes out clean. The twin of `AnnotationShape::Draw` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationDraw = {
  kind: "draw";
  points: AnnotationPoint[];
  smooth: boolean;
};

/**
 * A loupe showing a zoom area enlarged. `start` is the zoom area's top-left
 * corner and `end` its bottom-right, in source pixels; `loupe` is the loupe's
 * centre and `size` its longer side, the zoom area's shape scaled up to that.
 * The twin of `AnnotationShape::Magnify` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationMagnify = {
  end: AnnotationPoint;
  kind: "magnify";
  loupe: AnnotationPoint;
  size: number;
  start: AnnotationPoint;
};

/**
 * How a sticker showing a moving picture - a GIF, an animated PNG or WebP -
 * plays: how long one run lasts and how many frames it has, the frame a
 * still shows and playback starts on, and whether a recording plays it
 * through once, its clip then lasting exactly one run, rather than looping.
 * The twin of `StickerPlay` in
 * `src-tauri/src/editor/annotations/sticker/play.rs`.
 */
export type StickerPlay = {
  cycleMs: number;
  frame: number;
  frames: number;
  once: boolean;
};

/**
 * What a sticker shows: its picture by library id, that picture's width
 * over its height, and how it plays where it moves. As the picture the next
 * sticker is made with, it also carries how many pixels long a picture of
 * its own is on its longer side, which a fresh sticker takes as its size; an
 * emoji has none. The twin of `StickerArt` in
 * `src-tauri/src/editor/annotations/sticker/model.rs`.
 */
export type StickerArt = {
  aspect: number;
  asset: string;
  pixels?: number;
  play?: StickerPlay;
};

/**
 * A picture laid over the source. `center` is its middle and `size` its
 * longer side, in source pixels; `aspect` is the picture's width over its
 * height, `angle` how far it is turned clockwise, in radians, and `flip`
 * whether it is mirrored across its upright axis. `asset` names the picture
 * in the sticker library, and `play` how it plays where the picture moves.
 * The twin of `AnnotationShape::Sticker` in
 * `src-tauri/src/editor/annotations/shape.rs`.
 */
type AnnotationSticker = Omit<StickerArt, "pixels"> & {
  angle: number;
  center: AnnotationPoint;
  flip: boolean;
  kind: "sticker";
  size: number;
};

/** One line a highlight covers, in source pixels. */
export type HighlightBand = {
  bottom: number;
  left: number;
  right: number;
  top: number;
};

/**
 * A marker over lines of text. `start` and `end` are where the selection was
 * pressed and let go, in source pixels; `bands` what it covers, one per line in
 * reading order; `tone` how bright the page it was read from is, and its ink,
 * 0 to 1; `seed` its hand-drawn stroke's wobble. The twin of
 * `AnnotationShape::Highlight` in `src-tauri/src/editor/annotations/shape.rs`.
 */
export type AnnotationHighlight = {
  bands: HighlightBand[];
  end: AnnotationPoint;
  kind: "highlight";
  seed: number;
  start: AnnotationPoint;
  tone: { ink: number; surface: number };
};

/**
 * A text box's pointer, held against its box so it keeps its place however
 * the box is moved, retyped or resized. `along` is where the tip sits in each
 * axis as a share of the box's half size from its centre, -1 to 1; `reach` is
 * how far past that edge it goes, in ems of the box's type. One that reaches
 * nowhere is tucked in and not drawn. The twin of `TextPointer` in
 * `src-tauri/src/editor/annotations/text/model.rs`.
 */
export type TextPointer = {
  along: AnnotationPoint;
  reach: AnnotationPoint;
};

export type AnnotationShape =
  | AnnotationArrow
  | AnnotationCounter
  | AnnotationDraw
  | AnnotationHighlight
  | AnnotationMagnify
  | AnnotationOutline
  | AnnotationRedact
  | AnnotationSpotlight
  | AnnotationSticker
  | AnnotationText;

export type Annotation = {
  /**
   * Whether a timed annotation draws itself in at the start of its clip and
   * undraws at the end. Stills have no clip to animate over and draw whole.
   */
  animated: boolean;
  id: string;
  shape: AnnotationShape;
  style: AnnotationStyle;
  /**
   * Set where the draw tool made the annotation: a stroke, or what a held
   * stroke was taken for. Clear all takes these and leaves the rest.
   */
  pen?: true;
};

/**
 * Where a native commit falls in a text box's typing: the typing began, the
 * text changed, or the typing ended. The document groups the commits of one
 * typing into a single edit. The twin of `TextEditPhase` in
 * `src-tauri/src/editor/annotations/text/edit.rs`.
 */
export type AnnotationTextEdit = "begin" | "update" | "end";

export const annotationTextEdit = (
  value: unknown,
): AnnotationTextEdit | null =>
  value === "begin" || value === "update" || value === "end" ? value : null;
