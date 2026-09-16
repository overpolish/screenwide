// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { usePublishAnnotationSelection } from "./annotation-channel";
import {
  rememberAnnotationAnimated,
  rememberAnnotationStyle,
} from "./annotation-defaults";
import {
  Annotation,
  AnnotationStyle,
  annotationDeleteTarget,
} from "./annotations";
import { EditorKind } from "./types";

/** Selection and editing behaviour shared by screenshot and recording marks. */
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

  const applyStyle = (style: Partial<AnnotationStyle>) => {
    if (!selected) return;
    const dressed = { ...selected.style, ...style };
    rememberAnnotationStyle(dressed);
    onCommit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, style: dressed }
          : annotation,
      ),
    );
  };

  // Whether a mark animates is not part of its dress: it sits on the mark
  // itself, so it is committed on its own rather than through the style.
  const applyAnimated = (animated: boolean) => {
    if (!selected) return;
    rememberAnnotationAnimated(animated);
    onCommit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, animated }
          : annotation,
      ),
    );
  };

  // Turn the arrow round: the head rides the end point, so swapping the ends
  // points it the other way. The bend keeps its control, so the curve is the
  // mirror of itself rather than a different one.
  const applyReverse = () => {
    if (!selected) return;
    onCommit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? {
              ...annotation,
              shape: {
                ...annotation.shape,
                end: annotation.shape.start,
                start: annotation.shape.end,
              },
            }
          : annotation,
      ),
    );
  };

  usePublishAnnotationSelection(
    workspace,
    selected
      ? { animated: selected.animated, id: selected.id, style: selected.style }
      : null,
    { applyAnimated, applyReverse, applyStyle },
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
      onCommit(annotations.filter((annotation) => annotation.id !== target));
      if (target === selectedId) setSelectedId(null);
      if (target === hoveredId) setHoveredId(null);
      return true;
    },
    hasSelection: selected !== null,
    onHoverChange: setHoveredId,
    onSelectedChange: setSelectedId,
    selectedId,
  };
}
