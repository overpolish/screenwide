// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { arrangedGroup, Arrangement } from "../annotation-order";
import { Annotation } from "../annotations";
import { ScreenshotOutputSettings } from "../screenshot-output";
import { useAnnotations } from "../use-annotations";

import { useScreenshotAnnotationMenu } from "./use-screenshot-annotation-menu";

/** Adapts the screenshot layer's output commit to the shared annotation hook,
 * and moves the layer's annotations through its stacking - from a right press
 * on one, or the bracket keys on those in hand. */
export function useScreenshotAnnotations({
  onOutputChange,
  selectedItemId,
  selectedOutput,
}: {
  selectedItemId: number | null;
  selectedOutput: ScreenshotOutputSettings | null;
  onOutputChange?: (settings: ScreenshotOutputSettings, itemId: number) => void;
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
  useScreenshotAnnotationMenu({
    annotations,
    onCommit,
    selectedIds: selection.selectedIds,
  });
  return {
    ...selection,
    /** Moves the annotations in hand, several together keeping their own
     * stacking, answering whether there were any: a still draws every
     * annotation on its layer together, so each step passes the next one
     * along. */
    arrangeSelected: (arrangement: Arrangement) => {
      if (selection.selectedIds.size === 0) return false;
      const next = arrangedGroup(
        annotations,
        (annotation) => selection.selectedIds.has(annotation.id),
        { arrangement, meets: () => true },
      );
      if (next !== annotations) onCommit(next);
      return true;
    },
  };
}
