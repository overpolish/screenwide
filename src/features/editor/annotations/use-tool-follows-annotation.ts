// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef } from "react";

import { drawingToolKind } from "../tools/tool-registry";

import type { AnnotationKind } from "../../annotations/types";

/**
 * Choosing an annotation takes up its pen.
 *
 * Every drawing tool hit-tests every annotation, so the arrow pen can pick up
 * a counter and the text pen an arrow. What it must not leave behind is a
 * mismatch: the island dressing a counter while the next press on empty
 * picture draws an arrow. So choosing an annotation, with a pen or with
 * Select, takes up its own pen, which the island then shows, and the pen in
 * hand is always the shape the next press makes. Several chosen at once are
 * of no one kind, and take up nothing.
 */
export function useToolFollowsAnnotation(
  kind: AnnotationKind | null,
  tool: string | null,
  setTool: (tool: AnnotationKind) => void,
) {
  // The workspaces rebuild their setter every render; only the chosen
  // annotation and the tool in hand are worth reacting to.
  const setToolRef = useRef(setTool);
  setToolRef.current = setTool;
  useEffect(() => {
    if (kind === null) return;
    if (tool !== "select" && drawingToolKind(tool) === null) return;
    if (tool === kind) return;
    setToolRef.current(kind);
  }, [kind, tool]);
}

/**
 * Whether taking up `next` should let the annotation in hand go.
 *
 * Reaching for the tool that draws the other shape is a decision about what
 * comes next, so the annotation goes with it: otherwise the tool follows the
 * annotation straight back and the switch never lands. Taking up the tool an
 * annotation was drawn with keeps it, which is how choosing one adopts its
 * tool, the pen and a stroke included.
 */
export function toolDisagreesWithAnnotation(
  next: string | null,
  kind: AnnotationKind | null,
) {
  return drawingToolKind(next) !== null && kind !== null && kind !== next;
}
