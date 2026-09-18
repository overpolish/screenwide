// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { usePublishAnnotationSelection } from "./annotation-channel";
import {
  rememberAnnotationAngle,
  rememberAnnotationAnimated,
  rememberAnnotationStyle,
} from "./annotation-defaults";
import {
  Annotation,
  AnnotationStyle,
  annotationDeleteTarget,
  renumberedCounters,
} from "./annotations";
import { EditorKind } from "./types";

/** Selection and editing behaviour shared by screenshot and recording
 * annotations. */
export function useAnnotations({
  annotations,
  onCommit,
  workspace,
}: {
  annotations: Annotation[];
  onCommit: (annotations: Annotation[]) => void;
  workspace: EditorKind;
}) {
  const [selectedId, setSelectedId] = useState<string | null>(null);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const selected =
    annotations.find((annotation) => annotation.id === selectedId) ?? null;

  // Counters are numbered by their place in the list, so every list this hook
  // writes is renumbered on the way out: a delete closes the gap it leaves
  // rather than leaving 1 and 3 behind.
  const commit = (next: Annotation[]) => {
    onCommit(renumberedCounters(next));
  };
  // Where a counter's tail points is remembered however it was turned - by
  // its grip on the picture or by the panel's own control - so the next one
  // is dropped aiming the same way.
  const angle =
    selected?.shape.kind === "counter" ? selected.shape.angle : null;
  useEffect(() => {
    if (angle !== null) rememberAnnotationAngle(angle);
  }, [angle]);

  /** Turn the chosen counter's tail, in radians clockwise from east. */
  const applyAngle = (next: number) => {
    if (!selected || selected.shape.kind !== "counter") return;
    const shape = selected.shape;
    if (!Number.isFinite(next) || shape.angle === next) return;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape: { ...shape, angle: next } }
          : annotation,
      ),
    );
  };
  const applyStyle = (style: Partial<AnnotationStyle>) => {
    if (!selected) return;
    const dressed = { ...selected.style, ...style };
    rememberAnnotationStyle(dressed, selected.shape.kind);
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, style: dressed }
          : annotation,
      ),
    );
  };

  // Whether an annotation animates is not part of its dress: it sits on the
  // annotation itself, so it is committed on its own rather than through the
  // style.
  const applyAnimated = (animated: boolean) => {
    if (!selected) return;
    rememberAnnotationAnimated(animated);
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, animated }
          : annotation,
      ),
    );
  };

  // Turn the arrow round: the head rides the end point, so swapping the ends
  // points it the other way. The bend keeps its control, so the curve is the
  // mirror of itself rather than a different one. A counter has no ends to
  // swap; the panel does not offer the row for one.
  const applyReverse = () => {
    if (!selected || selected.shape.kind !== "arrow") return;
    const shape = selected.shape;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? {
              ...annotation,
              shape: { ...shape, end: shape.start, start: shape.end },
            }
          : annotation,
      ),
    );
  };

  usePublishAnnotationSelection(
    workspace,
    selected
      ? {
          angle: angle ?? undefined,
          animated: selected.animated,
          id: selected.id,
          kind: selected.shape.kind,
          style: selected.style,
        }
      : null,
    { applyAngle, applyAnimated, applyReverse, applyStyle },
  );

  return {
    canDelete:
      annotationDeleteTarget(annotations, hoveredId, selectedId) !== null,
    clearSelection: () => {
      setSelectedId(null);
    },
    deleteTargeted: () => {
      const target = annotationDeleteTarget(annotations, hoveredId, selectedId);
      if (target === null) return false;
      commit(annotations.filter((annotation) => annotation.id !== target));
      if (target === selectedId) setSelectedId(null);
      if (target === hoveredId) setHoveredId(null);
      return true;
    },
    hasSelection: selected !== null,
    onHoverChange: setHoveredId,
    onSelectedChange: setSelectedId,
    selectedId,
    /** Which shape the chosen annotation is, for the tool that follows it. */
    selectedKind: selected?.shape.kind ?? null,
  };
}
