// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { sweptTimelineItems } from "../timeline/tracks/timeline-item-selection";
import { EditorKind } from "../types";

import {
  ImagePlayChange,
  usePublishAnnotationSelection,
} from "./annotation-channel";
import {
  rememberAnnotationAngle,
  rememberAnnotationAnimated,
  rememberAnnotationImage,
  rememberAnnotationStyle,
} from "./annotation-defaults";
import {
  annotationDeleteTargets,
  chosenAnnotationIds,
  toggledAnnotationIds,
} from "./annotation-selection";
import { Annotation, AnnotationStyle, ImageArt } from "./annotations";
import { relaidHighlight } from "./highlight-strokes";

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
  workspace,
}: {
  annotations: Annotation[];
  onCommit: (annotations: Annotation[]) => void;
  workspace: EditorKind;
  /** Whether an annotation may be chosen together with others. */
  groupable?: (id: string) => boolean;
}) {
  const [chosen, setChosen] = useState(NOTHING_CHOSEN);
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const annotationIds = annotations.map((annotation) => annotation.id);
  const selectedIds = chosenAnnotationIds(annotationIds, chosen, groupable);
  const selectedId =
    selectedIds.size === 1 ? (selectedIds.values().next().value ?? null) : null;
  const selected =
    annotations.find((annotation) => annotation.id === selectedId) ?? null;

  // Where a counter's tail points is remembered however it was turned - by
  // its grip on the picture or by the panel's own control - so the next one
  // is dropped aiming the same way. An image is turned the same two ways,
  // but a fresh one always stands upright.
  const angle =
    selected?.shape.kind === "counter" || selected?.shape.kind === "image"
      ? selected.shape.angle
      : null;
  const counterAngle =
    selected?.shape.kind === "counter" ? selected.shape.angle : null;
  useEffect(() => {
    if (counterAngle !== null) rememberAnnotationAngle(counterAngle);
  }, [counterAngle]);

  /** Turn the chosen counter's tail or image, in radians clockwise from
   * east. */
  const applyAngle = (next: number) => {
    if (
      !selected ||
      (selected.shape.kind !== "counter" && selected.shape.kind !== "image")
    )
      return;
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
  // An image given another picture keeps its place, size and turn, and
  // takes the new picture's proportions and, where it moves, how it plays;
  // the next image shows it too.
  const image = selected?.shape.kind === "image" ? selected.shape : null;
  const art: ImageArt | null = image
    ? { aspect: image.aspect, asset: image.asset, play: image.play }
    : null;
  const applyImage = (next: ImageArt) => {
    if (!selected || selected.shape.kind !== "image") return;
    rememberAnnotationImage(next);
    const shape = selected.shape;
    if (shape.asset === next.asset && shape.aspect === next.aspect) return;
    // A placed image keeps its size: only the picture and its proportions
    // change, whatever size the new picture is of its own.
    const { aspect, asset, play } = next;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape: { ...shape, aspect, asset, play } }
          : annotation,
      ),
    );
  };
  // A moving image's frame, which a still shows and playback starts on,
  // and whether a recording plays it through once.
  const applyImagePlay = (next: ImagePlayChange) => {
    if (!selected || selected.shape.kind !== "image" || !selected.shape.play)
      return;
    const shape = selected.shape;
    const play = { ...selected.shape.play, ...next };
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape: { ...shape, play } }
          : annotation,
      ),
    );
  };
  const applyStyle = (style: Partial<AnnotationStyle>) => {
    if (!selected) return;
    const dressed = { ...selected.style, ...style };
    rememberAnnotationStyle(dressed, selected.shape.kind);
    const shape = relaidHighlight(selected, dressed) ?? selected.shape;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape, style: dressed }
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
  // mirror of itself rather than a different one. An image is mirrored
  // instead. A counter has no ends to swap; the panel does not offer the row
  // for one.
  const applyReverse = () => {
    if (!selected) return;
    const shape = selected.shape;
    const reversed =
      shape.kind === "arrow"
        ? { ...shape, end: shape.start, start: shape.end }
        : shape.kind === "image"
          ? { ...shape, flip: !shape.flip }
          : null;
    if (reversed === null) return;
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape: reversed }
          : annotation,
      ),
    );
  };

  // Lay a redaction's blocks out again, or draw a hand-drawn highlight's or
  // shape's stroke again. The seed comes from the browser's secure generator,
  // as a fresh redaction's does from the system's; the blocks carry nothing of
  // the picture's layout whatever the seed.
  const applyShuffle = () => {
    if (!selected) return;
    const shape = selected.shape;
    if (
      shape.kind !== "redact" &&
      shape.kind !== "highlight" &&
      shape.kind !== "shape"
    )
      return;
    const [seed] = crypto.getRandomValues(new Uint32Array(1));
    commit(
      annotations.map((annotation) =>
        annotation.id === selected.id
          ? { ...annotation, shape: { ...shape, seed } }
          : annotation,
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

  // Everything the pen made goes in one commit, so one undo brings it all
  // back: its strokes, and what held strokes were taken for. A stroke from
  // before the flag existed is still the pen's. Other annotations stay, and
  // so does the hover unless the pen made it.
  const penMade = (annotation: Annotation) =>
    annotation.pen === true || annotation.shape.kind === "draw";
  const canClearDrawings = annotations.some(penMade);
  const applyClearDrawings = () => {
    if (!canClearDrawings) return;
    remove(new Set(annotations.filter(penMade).map(({ id }) => id)));
  };
  const applyDelete = () => {
    if (selectedIds.size > 0) remove(selectedIds);
  };

  usePublishAnnotationSelection(
    workspace,
    {
      canClearDrawings,
      count: selectedIds.size,
      selection: selected
        ? {
            angle: angle ?? undefined,
            animated: selected.animated,
            id: selected.id,
            image: art ?? undefined,
            kind: selected.shape.kind,
            style: selected.style,
          }
        : null,
    },
    {
      applyAngle,
      applyAnimated,
      applyClearDrawings,
      applyDelete,
      applyImage,
      applyImagePlay,
      applyReverse,
      applyShuffle,
      applyStyle,
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
