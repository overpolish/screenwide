// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { SceneRotation } from "../layer-rotation";
import { sweptTimelineItems } from "../timeline/tracks/timeline-item-selection";
import { EditorKind } from "../types";

import { usePublishAnnotationSelection } from "./annotation-channel";
import {
  rememberAnnotationAnimated,
  rememberAnnotationStyle,
  rememberAnnotationTextLook,
} from "./annotation-defaults";
import { imageEdits } from "./annotation-image-edits";
import {
  annotationDeleteTargets,
  chosenAnnotationIds,
  toggledAnnotationIds,
} from "./annotation-selection";
import {
  dressedShape,
  freshSeed,
  relookedFocus,
  reversedShape,
  shuffledShape,
} from "./annotation-shape-edits";
import { withEmphasis } from "./annotation-text-selection";
import {
  Annotation,
  AnnotationShape,
  AnnotationStyle,
  TextSpan,
} from "./annotations";
import { imageRotation, withImageRotation } from "./image-rotation";
import {
  type AnnotationPins,
  type AnnotationScenes,
  publishedAnnotation,
} from "./published-annotation";
import { useRememberedShape } from "./use-remembered-shape";

import type { TextStyle } from "../../../bindings/TextStyle";

const NOTHING_CHOSEN: ReadonlySet<string> = new Set();
const anyGroupable = () => true;

/** Selection and editing behaviour shared by screenshot and recording
 * annotations. One annotation or several may be chosen; only one on its own
 * is dressed by the panel and gripped on the picture. Counters are numbered by
 * the document that owns them, across all of its lists: a screenshot across
 * its layers, a recording by clip time. One list on its own cannot know where
 * its counters fall in that run, so this hook leaves their numbers alone. */
export function useAnnotations({
  annotations,
  groupable = anyGroupable,
  onCommit: commit,
  pins,
  scenes,
  workspace,
}: {
  annotations: Annotation[];
  onCommit: (annotations: Annotation[]) => void;
  workspace: EditorKind;
  /** Whether an annotation may be chosen together with others. */
  groupable?: (id: string) => boolean;
  pins?: AnnotationPins;
  scenes?: AnnotationScenes;
}) {
  const [chosen, setChosen] = useState(NOTHING_CHOSEN);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const annotationIds = annotations.map((annotation) => annotation.id);
  const selectedIds = chosenAnnotationIds(annotationIds, chosen, groupable);
  const selectedId =
    selectedIds.size === 1 ? (selectedIds.values().next().value ?? null) : null;
  const selected =
    annotations.find((annotation) => annotation.id === selectedId) ?? null;

  const { counterAngle, lensZoom } = useRememberedShape(selected);

  // Give the chosen annotation `shape` in one edit; null leaves it as it is.
  const reshape = (shape: AnnotationShape | null) => {
    if (!selected || shape === null) return;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id ? { ...annotation, shape } : annotation,
      ),
    );
  };
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
  /** Set how many times the chosen lens enlarges. */
  const applyZoom = (zoom: number) => {
    if (selected?.shape.kind === "magnify" && zoom > 0 && zoom !== lensZoom)
      reshape({ ...selected.shape, zoom });
  };
  // An image is turned flat by its angle and in depth by its tilt, which the
  // Rotation row's dials show as one turn about three axes.
  const rotation =
    selected?.shape.kind === "image"
      ? imageRotation(selected.shape)
      : undefined;
  /** Turn the chosen image, a field at a time, in degrees about each axis. */
  const applyRotation = (next: Partial<SceneRotation>) => {
    if (!selected || selected.shape.kind !== "image" || !rotation) return;
    const shape = withImageRotation(selected.shape, { ...rotation, ...next });
    if (shape === selected.shape) return;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id ? { ...annotation, shape } : annotation,
      ),
    );
  };
  const { applyImage, applyImagePlay, applySway, art } = imageEdits(
    selected,
    reshape,
  );
  const applyStyle = (style: Partial<AnnotationStyle>) => {
    if (!selected) return;
    const dressed = { ...selected.style, ...style };
    rememberAnnotationStyle(dressed, selected.shape.kind);
    const shape = dressedShape(selected, dressed);
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape, style: dressed }
          : annotation,
      ),
    );
  };

  // A text's look is its own, as an image's picture is, and the next text
  // is dressed in it too.
  const applyLook = (look: TextStyle) => {
    if (selected?.shape.kind !== "text") return;
    rememberAnnotationTextLook(look);
    reshape({ ...selected.shape, look });
  };
  // The words a text sets apart are its own, and never carried to the next.
  const applyEmphasis = (emphasis: TextSpan[]) => {
    if (selected?.shape.kind === "text")
      reshape(withEmphasis(selected.shape, emphasis));
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

  const applyReverse = () => {
    if (selected) reshape(reversedShape(selected.shape));
  };
  const applyShuffle = () => {
    if (selected) reshape(shuffledShape(selected.shape, freshSeed()));
  };
  // A focus and a spotlight are one pen with two looks (`relookedFocus`).
  const applyKind = (kind: "focus" | "spotlight") => {
    const relooked = selected && relookedFocus(selected, kind);
    if (relooked)
      commit(
        annotations.map((annotation) =>
          annotation.id === relooked.id ? relooked : annotation,
        ),
      );
  };

  // A deleted annotation stays in the choice while it is gone: the choice
  // only ever shows annotations that are there, so it reads as let go, and an
  // undo that brings it back brings it back chosen. The hover goes with it.
  const remove = (targets: ReadonlySet<string>) => {
    commit(annotations.filter((annotation) => !targets.has(annotation.id)));
    if (hoveredId !== null && targets.has(hoveredId)) setHoveredId(null);
  };

  const applyDelete = () => {
    if (selectedIds.size > 0) remove(selectedIds);
  };

  usePublishAnnotationSelection(
    workspace,
    {
      count: selectedIds.size,
      selection:
        selected &&
        publishedAnnotation(selected, {
          angle: counterAngle,
          art,
          pin: pins?.viewOf(selected.id),
          placedInScene: scenes?.isPlaced(selected.id),
          rotation,
          zoom: lensZoom,
        }),
    },
    {
      applyAngle,
      applyAnimated,
      applyDelete,
      applyEmphasis,
      applyImage,
      applyImagePlay,
      applyKind,
      applyLook,
      applyPin: (action) => selected && pins?.apply(selected.id, action),
      applyReverse,
      applyRotation,
      applyShuffle,
      applyStyle,
      applySway,
      applyZoom,
      resetScene: () => selected && scenes?.reset(selected.id),
    },
  );

  const deleteTargets = annotationDeleteTargets(
    annotationIds,
    hoveredId,
    selectedIds,
  );
  return {
    canDelete: deleteTargets.size > 0,
    clearSelection: () => {
      setChosen(NOTHING_CHOSEN);
    },
    deleteTargeted: () => {
      if (deleteTargets.size === 0) return false;
      remove(deleteTargets);
      return true;
    },
    hasSelection: selectedIds.size > 0,
    onHoverChange: setHoveredId,
    /** Choose exactly `ids`: what a press on the picture reports. */
    onSelectedIdsChange: (ids: readonly string[]) => {
      setChosen(ids.length === 0 ? NOTHING_CHOSEN : new Set(ids));
    },
    /** Choose `id` alone, or with `toggle`, add it to the choice or take it
     * away. */
    selectAnnotation: (id: string, toggle: boolean) => {
      setChosen(toggledAnnotationIds(selectedIds, id, { groupable, toggle }));
    },
    /** Choose what a band swept over, alone or added to the choice. */
    selectAnnotations: (ids: readonly string[], additive: boolean) => {
      setChosen(sweptTimelineItems(selectedIds, ids, additive));
    },
    /** The one annotation in hand, or null with none or several chosen. */
    selectedId,
    selectedIds,
    /** Which shape the one annotation in hand is, for the tool that follows
     * it. A group has no one shape, so no tool follows it. */
    selectedKind: selected?.shape.kind ?? null,
  };
}
