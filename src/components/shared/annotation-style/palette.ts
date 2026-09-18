// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The colours an arrow is offered.
 *
 * An annotation's colour is stored in the document and drawn by the compositor,
 * so it is a value rather than a token: the shader has no theme to resolve
 * against, and an annotation must not change colour when the editor does. The
 * palette is the AppKit system palette measured on macOS 26 - the same source
 * as the semantic colours in `src/index.css`, whose four values are repeated
 * here exactly rather than approximated - plus the two achromatic ends every
 * annotation tool offers.
 */
export type AnnotationSwatch = {
  /** `#rrggbb`, as the document stores it. */
  color: string;
  name: string;
};

/** What a fresh arrow is drawn in before anything has been chosen: yellow reads
 * as an annotation on almost any screenshot, where the accent is sometimes the
 * very colour being pointed at. Rust dresses that first arrow - this is the
 * twin of `NEW_ARROW_COLOR` in
 * `src-tauri/src/editor/screenshot_preview/annotation.rs`, and the palette's
 * own Yellow is the same colour so the swatch reads as chosen. */
const DEFAULT_ANNOTATION_COLOR = "#ffcc00";

export const ANNOTATION_SWATCHES: AnnotationSwatch[] = [
  { color: "#ff383c", name: "Red" },
  { color: "#ff8d28", name: "Orange" },
  { color: DEFAULT_ANNOTATION_COLOR, name: "Yellow" },
  { color: "#34c759", name: "Green" },
  { color: "#0088ff", name: "Blue" },
  { color: "#cb30e0", name: "Purple" },
  { color: "#ff2d55", name: "Pink" },
  { color: "#ffffff", name: "White" },
  { color: "#000000", name: "Black" },
];

/** Whether `color` is one of the offered swatches, ignoring the case the
 * native colour panel happens to report its hex in. */
export const isAnnotationSwatch = (color: string) =>
  ANNOTATION_SWATCHES.some((swatch) =>
    sameAnnotationColor(swatch.color, color),
  );

/** How many colours of your own are kept. Past this the oldest is forgotten:
 * the row is a shortcut back to what you have been using, not an archive. */
const SAVED_COLOR_LIMIT = 12;

/** Two colours are the same colour whatever case their hex arrives in: the
 * palette is written in lower case and AppKit reports upper. */
export const sameAnnotationColor = (first: string, second: string) =>
  first.toLowerCase() === second.toLowerCase();

/**
 * The saved colours with `color` kept among them, newest last.
 *
 * A colour already on offer is not saved twice, and one that is already kept
 * moves to the end rather than being added again, so the row reads as what
 * has been used lately. Kept in lower case, whatever case AppKit reports it
 * in, so the same colour can never be kept twice over.
 */
export const withAnnotationColor = (saved: string[], color: string) => {
  if (!/^#[0-9a-f]{6}([0-9a-f]{2})?$/iu.test(color)) return saved;
  if (isAnnotationSwatch(color)) return saved;
  const rest = saved.filter((kept) => !sameAnnotationColor(kept, color));
  return [...rest, color.toLowerCase()].slice(-SAVED_COLOR_LIMIT);
};

/** The saved colours with `color` forgotten. */
export const withoutAnnotationColor = (saved: string[], color: string) =>
  saved.filter((kept) => !sameAnnotationColor(kept, color));
