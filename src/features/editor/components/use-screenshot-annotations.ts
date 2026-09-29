// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { arranged, Arrangement } from "../annotation-order";
import { Annotation } from "../annotations";
import { ScreenshotOutputSettings } from "../screenshot-output";
import { useAnnotations } from "../use-annotations";

import { useScreenshotAnnotationMenu } from "./use-screenshot-annotation-menu";

/** Adapts the screenshot layer's output commit to the shared annotation hook,
 * and moves the layer's annotations through its stacking - from a right press
 * on one, or the bracket keys on the one in hand. */
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
  useScreenshotAnnotationMenu({ annotations, onCommit });
  const selection = useAnnotations({
    annotations,
    onCommit,
    workspace: "screenshot",
  });
  return {
    ...selection,
    /** Moves the annotation in hand, answering whether there was one: a still
     * draws every annotation on its layer together, so each step passes the
     * next one along. */
    arrangeSelected: (arrangement: Arrangement) => {
      const index = annotations.findIndex(
        (annotation) => annotation.id === selection.selectedId,
      );
      if (index < 0) return false;
      const next = arranged(annotations, index, {
        arrangement,
        meets: () => true,
      });
      if (next !== annotations) onCommit(next);
      return true;
    },
  };
}
