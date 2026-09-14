// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ScreenshotOutputSettings } from "../screenshot-output";
import { useAnnotations } from "../use-annotations";

/** Adapts the screenshot layer's output commit to the shared annotation hook. */
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
  return useAnnotations({
    annotations,
    onCommit: (next) => {
      if (!selectedOutput || selectedItemId === null) return;
      onOutputChange?.(
        { ...selectedOutput, annotations: next },
        selectedItemId,
      );
    },
    workspace: "screenshot",
  });
}
