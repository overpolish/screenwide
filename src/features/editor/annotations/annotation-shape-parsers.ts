// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * Each kind's stored shape, read back from a document: the shape as it can be
 * drawn, or null where the compositor could not place it. `ANNOTATION_KINDS`
 * gives each kind its reader from here.
 */

import {
  annotationBox,
  annotationPoint,
  annotationSeed,
  highlightBand,
  highlightTone,
  textControls,
  textEmphasis,
  textLook,
} from "./annotation-shape-parts";

import type {
  AnnotationArrow,
  AnnotationCounter,
  AnnotationHighlight,
  AnnotationPoint,
  AnnotationShape,
  AnnotationText,
  ImagePlay,
  ImageTilt,
} from "./annotations";

type Shape<Kind extends AnnotationShape["kind"]> = Extract<
  AnnotationShape,
  { kind: Kind }
>;

export const arrowShape = (value: unknown): Shape<"arrow"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationArrow>;
  const control = annotationPoint(shape.control);
  const end = annotationPoint(shape.end);
  const start = annotationPoint(shape.start);
  return control && end && start
    ? { control, end, kind: "arrow", start }
    : null;
};

export const counterShape = (value: unknown): Shape<"counter"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationCounter>;
  const center = annotationPoint(shape.center);
  const number = shape.value;
  return center &&
    typeof shape.angle === "number" &&
    Number.isFinite(shape.angle) &&
    typeof number === "number" &&
    Number.isInteger(number) &&
    number >= 1
    ? { angle: shape.angle, center, kind: "counter", value: number }
    : null;
};

/** The most points a stroke keeps. The twin of `MAX_DRAW_POINTS` in
 * `src-tauri/src/editor/annotations/freehand/model.rs`. */
const MAX_DRAW_POINTS = 4096;

export const drawShape = (value: unknown): Shape<"draw"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  if (!Array.isArray(shape.points)) return null;
  const points = shape.points.map(annotationPoint);
  return points.length > 0 &&
    points.length <= MAX_DRAW_POINTS &&
    points.every((point) => point !== null)
    ? { kind: "draw", points, smooth: shape.smooth === true }
    : null;
};

export const highlightShape = (value: unknown): Shape<"highlight"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationHighlight>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  const seed = annotationSeed(shape.seed);
  const bands = Array.isArray(shape.bands)
    ? shape.bands.map(highlightBand)
    : [];
  return start && end && seed !== null && bands.every((band) => band !== null)
    ? {
        bands,
        end,
        kind: "highlight",
        seed,
        start,
        tone: highlightTone(shape.tone),
      }
    : null;
};

/** The zooms a lens comes in. The twin of `MAGNIFY_ZOOMS` in
 * `src-tauri/crates/annotations/src/magnify/model.rs`, which a Rust test holds
 * to this line. */
export const LENS_ZOOMS = [1.5, 2, 3, 4] as const;

/** The zoom a fresh lens comes in. The twin of `NEW_MAGNIFY_ZOOM`. */
export const NEW_LENS_ZOOM = 2;

/** The look nearest `zoom`, measured as a ratio, since a zoom is a factor:
 * 2.5× is nearer 3× than 2×. The twin of `nearest_zoom`. */
export const nearestLensZoom = (zoom: number): number =>
  Number.isFinite(zoom) && zoom > 0
    ? LENS_ZOOMS.reduce((best, look) =>
        Math.abs(Math.log(zoom / look)) < Math.abs(Math.log(zoom / best))
          ? look
          : best,
      )
    : NEW_LENS_ZOOM;

/** The lens a magnifier from before the lens becomes: over its zoom area's
 * centre, as big as its loupe was - the zoom area's shape `size` along its
 * longer side - at the look nearest the zoom that size gave it. The twin of
 * `lens_from_loupe`. */
const lensFromLoupe = (
  start: AnnotationPoint,
  end: AnnotationPoint,
  size: number,
): Shape<"magnify"> => {
  const longest = Math.max(
    Math.abs(end.x - start.x),
    Math.abs(end.y - start.y),
  );
  const scale =
    longest > 0 && Number.isFinite(size) && size > 0 ? size / longest : 1;
  const centre = { x: (start.x + end.x) / 2, y: (start.y + end.y) / 2 };
  const half = {
    x: (Math.abs(end.x - start.x) / 2) * scale,
    y: (Math.abs(end.y - start.y) / 2) * scale,
  };
  return {
    end: { x: centre.x + half.x, y: centre.y + half.y },
    kind: "magnify",
    start: { x: centre.x - half.x, y: centre.y - half.y },
    zoom: nearestLensZoom(scale),
  };
};

export const magnifyShape = (value: unknown): Shape<"magnify"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  if (!start || !end) return null;
  const { size, zoom } = shape;
  if (typeof zoom === "number") {
    return Number.isFinite(zoom) && zoom > 0
      ? { end, kind: "magnify", start, zoom }
      : null;
  }
  return typeof size === "number"
    ? lensFromLoupe(start, end, size)
    : { end, kind: "magnify", start, zoom: NEW_LENS_ZOOM };
};

export const redactShape = (value: unknown): Shape<"redact"> | null => {
  const box = annotationBox(value);
  return box && { ...box, kind: "redact" };
};

export const outlineShape = (value: unknown): Shape<"shape"> | null => {
  const box = annotationBox(value);
  return box && { ...box, kind: "shape" };
};

export const spotlightShape = (value: unknown): Shape<"spotlight"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  return start && end ? { end, kind: "spotlight", start } : null;
};

export const focusShape = (value: unknown): Shape<"focus"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  const start = annotationPoint(shape.start);
  const end = annotationPoint(shape.end);
  return start && end ? { end, kind: "focus", start } : null;
};

/** A finite number above zero, or null. */
const positive = (value: unknown) =>
  typeof value === "number" && Number.isFinite(value) && value > 0
    ? value
    : null;

/** How a moving image plays, or undefined for one that does not move:
 * fewer than two frames, or no time for them, is a still. */
const imagePlay = (value: unknown): ImagePlay | undefined => {
  const play = (value ?? {}) as Record<string, unknown>;
  const cycleMs = positive(play.cycleMs);
  const frames = positive(play.frames);
  if (cycleMs === null || frames === null || frames < 2) return undefined;
  const count = Math.floor(frames);
  const frame = typeof play.frame === "number" ? Math.floor(play.frame) : 0;
  return {
    cycleMs,
    frame: Math.min(Math.max(frame, 0), count - 1),
    frames: count,
    once: play.once === true,
  };
};

/** An image's turn in depth, or undefined where it lies flat or the tilt
 * cannot be read. */
const imageTilt = (value: unknown): ImageTilt | undefined => {
  const tilt = (value ?? {}) as Record<string, unknown>;
  const { x, y } = tilt;
  return typeof x === "number" &&
    typeof y === "number" &&
    Number.isFinite(x) &&
    Number.isFinite(y) &&
    (x !== 0 || y !== 0)
    ? { x, y }
    : undefined;
};

export const imageShape = (value: unknown): Shape<"image"> | null => {
  const shape = (value ?? {}) as Record<string, unknown>;
  const center = annotationPoint(shape.center);
  const size = positive(shape.size);
  const aspect = positive(shape.aspect);
  const angle = shape.angle;
  const tilt = imageTilt(shape.tilt);
  const play = imagePlay(shape.play);
  // A seed that cannot be read leaves the image standing still.
  const sway = annotationSeed(shape.sway);
  return center &&
    size !== null &&
    aspect !== null &&
    typeof angle === "number" &&
    Number.isFinite(angle) &&
    typeof shape.asset === "string"
    ? {
        angle,
        aspect,
        asset: shape.asset,
        center,
        flip: shape.flip === true,
        kind: "image",
        ...(play ? { play } : {}),
        size,
        ...(sway === null ? {} : { sway }),
        ...(tilt ? { tilt } : {}),
      }
    : null;
};

export const textShape = (value: unknown): Shape<"text"> | null => {
  const shape = (value ?? {}) as Partial<AnnotationText>;
  const origin = annotationPoint(shape.origin);
  return origin && typeof shape.text === "string"
    ? {
        kind: "text",
        origin,
        ...textLook(shape.look),
        ...textControls(shape.controls),
        ...textEmphasis(shape.emphasis),
        text: shape.text,
      }
    : null;
};
