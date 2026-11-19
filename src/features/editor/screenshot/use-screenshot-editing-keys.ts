// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { AnnotationChoice } from "../annotations/annotation-clipboard";
import { CaptureSource } from "../layer-measure";

import { screenshotAnnotationClipboard } from "./screenshot-annotation-clipboard";
import {
  ScreenshotOutputSettings,
  ScreenshotWorkspaceOutputSettings,
} from "./screenshot-output";
import { useScreenshotNudge } from "./use-screenshot-nudge";

/**
 * The screenshot editor's editing keys: ⌘C and ⌘X copy and cut the
 * annotations chosen (with none, ⌘C is the title bar's copy of the
 * screenshot), ⌘D duplicates them, ⌘A chooses every annotation on the layer,
 * ⌘V pastes copied annotations, and the arrows move the annotations chosen,
 * or with none the layer in hand where `canNudgeLayer`.
 */
export function useScreenshotEditingKeys({
  annotations,
  canNudgeLayer,
  items,
  onOutputChange,
  onSelectedItemChange,
  selectedItemId,
  selectedOutput,
  selectedPicture,
  targetItemId,
  workspace,
}: {
  annotations: AnnotationChoice;
  canNudgeLayer: boolean;
  items: readonly (CaptureSource & { id: number })[];
  selectedItemId: number | null;
  selectedOutput: ScreenshotOutputSettings | null;
  selectedPicture: { id: number } | undefined;
  targetItemId: number | null;
  workspace: ScreenshotWorkspaceOutputSettings | undefined;
  onOutputChange?: (settings: ScreenshotOutputSettings, itemId: number) => void;
  onSelectedItemChange?: (itemId: number | null) => void;
}) {
  const nudgeLayer = useScreenshotNudge({
    onOutputChange,
    selectedItem: selectedPicture,
    selectedOutput,
  });
  const clipboard = screenshotAnnotationClipboard({
    annotations,
    items,
    onOutputChange,
    onSelectedItemChange,
    selectedItemId,
    targetItemId,
    workspace,
  });
  return {
    keys: {
      onCopy: clipboard.copy,
      onCut: clipboard.cut,
      onDuplicate: clipboard.duplicate,
      onNudge: clipboard.hasChosen
        ? clipboard.nudge
        : canNudgeLayer
          ? nudgeLayer
          : undefined,
      onSelectAll: clipboard.selectAll,
    },
    pasteAnnotations: clipboard.paste,
  };
}
