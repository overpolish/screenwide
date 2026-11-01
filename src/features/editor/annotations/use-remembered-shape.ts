// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect } from "react";

import {
  rememberAnnotationAngle,
  rememberAnnotationZoom,
} from "./annotation-defaults";
import { Annotation } from "./annotations";

/** What of the chosen annotation's own shape the next one of its kind is
 * drawn with: where a counter's tail points and how many times a lens
 * enlarges. Each is remembered however it was set - by a grip on the picture
 * or by the island - so the next counter is dropped aiming the same way and
 * the next lens drawn at the same zoom. Answers both, null on any other
 * kind. */
export function useRememberedShape(selected: Annotation | null) {
  const counterAngle =
    selected?.shape.kind === "counter" ? selected.shape.angle : null;
  useEffect(() => {
    if (counterAngle !== null) rememberAnnotationAngle(counterAngle);
  }, [counterAngle]);
  const lensZoom =
    selected?.shape.kind === "magnify" ? selected.shape.zoom : null;
  useEffect(() => {
    if (lensZoom !== null) rememberAnnotationZoom(lensZoom);
  }, [lensZoom]);
  return { counterAngle, lensZoom };
}
