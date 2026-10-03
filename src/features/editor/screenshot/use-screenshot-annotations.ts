// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useState } from "react";

import { Arrangement } from "../annotations/annotation-order";
import { Annotation } from "../annotations/annotations";
import { useAnnotations } from "../annotations/use-annotations";

import {
  arrangedScreenshotAnnotations,
  ScreenshotSourceWidths,
} from "./screenshot-annotation-layers";
import {
  ScreenshotOutputSettings,
  screenshotWorkspaceItemOutput,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";
import { useScreenshotAnnotationMenu } from "./use-screenshot-annotation-menu";

/** Adapts the screenshot layer's output commit to the shared annotation hook,
 * and moves annotations through the stacking - from a right press on one, on
 * whichever layer it is drawn, or the bracket keys on those in hand. A step
 * past the end of a layer carries them into the next layer, which is then
 * the selected one. */
export function useScreenshotAnnotations({
  onOutputChange,
  onSelectedItemChange,
  onWorkspaceChange,
  selectedItemId,
  selectedOutput,
  sourceWidths,
  workspace,
}: {
  selectedItemId: number | null;
  selectedOutput: ScreenshotOutputSettings | null;
  sourceWidths: ScreenshotSourceWidths;
  workspace: ScreenshotWorkspaceOutputSettings | undefined;
  onOutputChange?: (settings: ScreenshotOutputSettings, itemId: number) => void;
  onSelectedItemChange?: (itemId: number | null) => void;
  onWorkspaceChange?: (settings: ScreenshotWorkspaceOutputSettings) => void;
}) {
  const annotations = selectedOutput?.annotations ?? [];
  const onCommit = (next: Annotation[]) => {
    if (!selectedOutput || selectedItemId === null) return;
    onOutputChange?.({ ...selectedOutput, annotations: next }, selectedItemId);
  };
  const selection = useAnnotations({
    annotations,
    onCommit,
    workspace: "screenshot",
  });
  // One workspace commit, so a move that carries annotations into another
  // layer is one undo step however many layers it touches.
  const arrange = (
    arrangement: Arrangement,
    itemId: number,
    isMember: (annotation: Annotation) => boolean,
  ) => {
    if (!workspace) return;
    const moved = arrangedScreenshotAnnotations({
      arrangement,
      isMember,
      itemId,
      sourceWidths,
      workspace,
    });
    if (!moved) return;
    onWorkspaceChange?.(moved.workspace);
    if (moved.itemId !== selectedItemId) onSelectedItemChange?.(moved.itemId);
  };
  useScreenshotAnnotationMenu({
    arrange,
    selectedIds: selection.selectedIds,
    workspace,
  });
  // The halo can rest on an annotation on any layer, and the hand pointing
  // at it wins over the choice, as it does on the selected layer.
  const [hoveredId, setHoveredId] = useState<string | null>(null);
  const hoveredElsewhere =
    hoveredId === null
      ? undefined
      : workspace?.items.find(
          (item) =>
            item.id !== selectedItemId &&
            item.output.annotations.some(({ id }) => id === hoveredId),
        );
  return {
    ...selection,
    /** Moves the annotations in hand, several together keeping their own
     * stacking, answering whether there were any. */
    arrangeSelected: (arrangement: Arrangement) => {
      if (selection.selectedIds.size === 0 || selectedItemId === null)
        return false;
      arrange(arrangement, selectedItemId, (annotation) =>
        selection.selectedIds.has(annotation.id),
      );
      return true;
    },
    deleteTargeted: () => {
      if (!workspace || !hoveredElsewhere) return selection.deleteTargeted();
      onOutputChange?.(
        {
          ...screenshotWorkspaceItemOutput(workspace, hoveredElsewhere.id),
          annotations: hoveredElsewhere.output.annotations.filter(
            ({ id }) => id !== hoveredId,
          ),
        },
        hoveredElsewhere.id,
      );
      setHoveredId(null);
      return true;
    },
    onHoverChange: (id: string | null) => {
      setHoveredId(id);
      selection.onHoverChange(id);
    },
  };
}
