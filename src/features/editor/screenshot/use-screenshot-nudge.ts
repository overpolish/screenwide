// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useRef } from "react";

import { ScreenshotOutputSettings } from "./screenshot-output";

// A keyboard nudge writes exactly the fields a select-tool drag writes - the
// `move` branch of `selectionGesture` in preview-viewport.tsx. It is not
// wrapped in an edit gesture on purpose: the history hook already groups
// same-key edits that land within its grouping delay, so a held arrow folds
// into one undo step while a pause starts the next. No snapping: the arrows
// are the way to place a layer precisely.

/** Moves the chosen layer a pixel, or ten when `coarse`, each way. */
export function useScreenshotNudge({
  onOutputChange,
  selectedItem,
  selectedOutput,
}: {
  selectedItem: { id: number } | undefined;
  selectedOutput: ScreenshotOutputSettings | null;
  onOutputChange?: (next: ScreenshotOutputSettings, itemId: number) => void;
}) {
  const nudgeRef = useRef<{
    after: ScreenshotOutputSettings;
    beforeX: number;
    beforeY: number;
    itemId: number;
  } | null>(null);
  return (directionX: number, directionY: number, coarse: boolean) => {
    if (!selectedItem || !selectedOutput) return;
    const pixels = coarse ? 10 : 1;
    const deltaX = directionX * pixels;
    const deltaY = directionY * pixels;
    // Key repeat can outrun React. While the committed settings have not come
    // back yet the props still hold the previous press's starting point, so
    // continue from what that press sent instead of repeating it.
    const pending = nudgeRef.current;
    const base =
      pending &&
      pending.itemId === selectedItem.id &&
      pending.beforeX === selectedOutput.cropX &&
      pending.beforeY === selectedOutput.cropY
        ? pending.after
        : selectedOutput;
    const next = {
      ...base,
      cropX: base.cropX + deltaX,
      cropY: base.cropY + deltaY,
      imageX: base.imageX + deltaX,
      imageY: base.imageY + deltaY,
    };
    nudgeRef.current = {
      after: next,
      beforeX: selectedOutput.cropX,
      beforeY: selectedOutput.cropY,
      itemId: selectedItem.id,
    };
    onOutputChange?.(next, selectedItem.id);
  };
}
