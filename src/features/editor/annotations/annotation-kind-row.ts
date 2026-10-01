// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import type { AnnotationShape } from "./annotations";
import type { AnnotationKind } from "../../../components/shared/annotation-style/types";
import type { ANNOTATION_SIZES } from "../../../components/shared/annotation-style/widths";

/**
 * What one kind of annotation is and can do: how it is read from a document,
 * how long it takes to arrive, what it is called, and which controls it
 * answers to. `ANNOTATION_KINDS` holds one for every kind.
 *
 * One row per kind rather than a condition at each control, so a tool added
 * later states its own answers in one place and the panel, the lane and the
 * document reader all follow. The sizes come from the shared table the live
 * overlay's controls read, so a kind is offered the same sizes wherever it is
 * drawn.
 */
export type AnnotationKindRow<Shape extends AnnotationShape> =
  (typeof ANNOTATION_SIZES)[AnnotationKind] & {
    /** Whether it draws itself in over its clip, which is what the panel's
     * Animate switch offers. */
    animates: boolean;
    drawInMs: number;
    /** Whether it lines up lines of text. */
    hasAlign: boolean;
    /** Whether it is aimed by an angle of its own, rather than by its ends. */
    hasAngle: boolean;
    /** Whether it can blur what lies outside it. */
    hasBlur: boolean;
    /** Whether it has a colour of its own. A spotlight's shade is black. */
    hasColor: boolean;
    /** Whether it fits itself to what is under it, or can be laid by hand
     * over a box instead. */
    hasFit: boolean;
    /** Whether it can be drawn by hand, and drawn again differently. */
    hasHandDrawn: boolean;
    /** Whether it carries heads to choose between. */
    hasHead: boolean;
    /** Whether it can tint what it covers instead of recolouring it. */
    hasInk: boolean;
    /** Whether its box's corners can be rounded. */
    hasRadius: boolean;
    /** Whether it hides what is under it, and so offers the redaction modes. */
    hasRedaction: boolean;
    /** Whether it casts a shadow onto the picture, which is its own to choose. */
    hasShadow: boolean;
    /** Whether its size is its own to choose. A highlight's bands are as tall
     * as the lines it covers, and a box is laid with its one marker. */
    hasSize: boolean;
    /** Whether its edge can fade in from its box, which is its own to choose. */
    hasSoftness: boolean;
    /** What the timeline lane calls one, `index` being its place in the lane. */
    laneLabel: (shape: Shape, index: number) => string;
    /** The shape as stored, or null where the compositor could not place it. */
    parseShape: (value: unknown) => Shape | null;
    /** Whether it can be turned round end for end. */
    reversible: boolean;
    /** Whether a fresh one arrives without drawing itself in, whatever the
     * last one did: a redaction would show what it hides until it had. */
    startsStill: boolean;
  };
