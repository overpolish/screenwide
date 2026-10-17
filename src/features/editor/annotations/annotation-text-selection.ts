// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationShape, AnnotationText, TextSpan } from "./annotations";

/** What the panel shows of a placed text beyond its dress: its look, what
 * it says and the words it sets apart. Nothing for any other annotation. */
export const textSelection = (shape: AnnotationShape) =>
  shape.kind === "text"
    ? { emphasis: shape.emphasis ?? [], look: shape.look, text: shape.text }
    : {};

/** `shape` setting apart the words `emphasis` and no others, holding no list
 * where none is chosen, as Rust writes it. */
export const withEmphasis = (
  shape: AnnotationText,
  emphasis: TextSpan[],
): AnnotationText => {
  const { emphasis: _, ...rest } = shape;
  return emphasis.length > 0 ? { ...rest, emphasis } : rest;
};
