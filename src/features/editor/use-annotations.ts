// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { usePublishAnnotationSelection } from "./annotation-channel";
import { rememberAnnotationStyle } from "./annotation-defaults";
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

  usePublishAnnotationSelection(
    workspace,
    selected ? { id: selected.id, style: selected.style } : null,
    applyStyle,
  );

  return {
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
