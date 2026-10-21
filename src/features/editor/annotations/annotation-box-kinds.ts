// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

/**
 * The rows of the kinds held by a box and its eight grips - a redaction, a
 * shape, a spotlight and a focus - which `ANNOTATION_KINDS` lays beside the
 * rest. The twin of the kinds `box_gesture` moves in
 * `src-tauri/crates/annotations`.
 */

import { t } from "../../../i18n/i18n";
import { ANNOTATION_SIZES } from "../../annotations/widths";

import {
  ANNOTATION_COUNTER_DRAW_IN_MS,
  ANNOTATION_DRAW_IN_MS,
  ANNOTATION_SPOTLIGHT_DRAW_IN_MS,
} from "./annotation-draw-in";
import {
  focusShape,
  outlineShape,
  redactShape,
  spotlightShape,
} from "./annotation-shape-parsers";

import type { AnnotationKindRow } from "./annotation-kind-row";
import type { AnnotationShape } from "./annotations";

type BoxKind = "focus" | "redact" | "shape" | "spotlight";

export const BOX_KIND_ROWS: {
  [Kind in BoxKind]: AnnotationKindRow<
    Extract<AnnotationShape, { kind: Kind }>
  >;
} = {
  focus: {
    ...ANNOTATION_SIZES.focus,
    // The blur deepens round it the way a spotlight's shade does, so it takes
    // the same time to arrive and leave.
    animates: true,
    drawInMs: ANNOTATION_SPOTLIGHT_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    // Nothing is drawn for it: the blur is the effect, so there is no colour.
    hasColor: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: false,
    hasSize: false,
    hasSoftness: false,
    laneLabel: (_, index) => t("editor-lane-focus", { number: index + 1 }),
    parseShape: focusShape,
    reversible: false,
    startsStill: false,
  },
  redact: {
    ...ANNOTATION_SIZES.redact,
    // A redaction ramps in on a counter's timing and never leaves. It starts
    // without the ramp, since a box arriving shows what it hides until it
    // has.
    animates: true,
    drawInMs: ANNOTATION_COUNTER_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: true,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (_, index) => t("editor-lane-redaction", { number: index + 1 }),
    parseShape: redactShape,
    reversible: false,
    startsStill: true,
  },
  shape: {
    ...ANNOTATION_SIZES.shape,
    // Drawn round its outline the way an arrow is along its path.
    animates: true,
    drawInMs: ANNOTATION_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: false,
    hasColor: true,
    hasFit: false,
    hasHandDrawn: true,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: false,
    hasSize: true,
    hasSoftness: false,
    laneLabel: (_, index) => t("editor-lane-shape", { number: index + 1 }),
    parseShape: outlineShape,
    reversible: false,
    startsStill: false,
  },
  spotlight: {
    ...ANNOTATION_SIZES.spotlight,
    // The shade deepens round it as it arrives and lifts as it leaves. It is
    // no colour and no stroke: only its box, its corners, its fade and
    // whether what is outside it blurs.
    animates: true,
    drawInMs: ANNOTATION_SPOTLIGHT_DRAW_IN_MS,
    hasAlign: false,
    hasAngle: false,
    hasBlur: true,
    hasColor: false,
    hasFit: false,
    hasHandDrawn: false,
    hasHead: false,
    hasInk: false,
    hasRadius: true,
    hasRedaction: false,
    hasShadow: false,
    hasSize: false,
    hasSoftness: true,
    laneLabel: (_, index) => t("editor-lane-spotlight", { number: index + 1 }),
    parseShape: spotlightShape,
    reversible: false,
    startsStill: false,
  },
};
