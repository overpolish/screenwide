// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEditorWindowShortcuts } from "../shortcuts/use-editor-window-shortcuts";

import type { screenshotChoices } from "./screenshot-choices";
import type { ScreenshotTool } from "./screenshot-column";
import type { useScreenshotAnnotations } from "./use-screenshot-annotations";
import type { useScreenshotEditingKeys } from "./use-screenshot-editing-keys";
import type { useScreenshotLayerActions } from "./use-screenshot-layer-actions";

/** The screenshot editor's window keys, answered by what is in hand: the
 * tool, the annotations chosen and the layer. */
export function useScreenshotShortcuts({
  annotations,
  choices,
  editing,
  hasPicture,
  isScreenOpen,
  layerActions,
  tool,
}: {
  annotations: ReturnType<typeof useScreenshotAnnotations>;
  choices: ReturnType<typeof screenshotChoices>;
  editing: ReturnType<typeof useScreenshotEditingKeys>;
  hasPicture: boolean;
  isScreenOpen: boolean;
  layerActions: ReturnType<typeof useScreenshotLayerActions>;
  tool: Exclude<ScreenshotTool, null>;
}) {
  const { backToSelect, chooseMode, selectTool, toggle } = choices;
  const isHolding =
    tool !== "select" || annotations.hasSelection || isScreenOpen;
  useEditorWindowShortcuts({
    ...editing.keys,
    // A chosen annotation moves through its layer's stacking; with none in
    // hand the layer itself moves.
    onArrange: (move) => {
      if (!annotations.arrangeSelected(move)) layerActions.moveLayer(move);
    },
    // The crop tool is the one tool you are "in": Enter accepts what is
    // framed and puts it down, the crop itself committed as each handle was
    // released.
    onConfirm: tool === "crop" ? selectTool : undefined,
    // Backspace and Delete take away the annotation the hand is pointing at
    // first, and only the layer when there is no annotation under them.
    onDelete: () => {
      if (!annotations.deleteTargeted()) layerActions.deleteLayer();
    },
    // Escape puts the crop away with its layer chosen, and anything else
    // back to Select with nothing chosen.
    onDeselect: isHolding ? choices.deselect : undefined,
    // ⌘D lets go of everything chosen, the layer as well, with Select in
    // hand, where no annotation is chosen to duplicate.
    onDeselectAll: backToSelect,
    onResizeCanvas: () => {
      toggle("canvas");
    },
    onSelectTool: selectTool,
    onToggleCrop: () => {
      if (hasPicture) toggle("crop");
    },
    onToggleScreenPanel: () => {
      if (hasPicture) chooseMode(isScreenOpen ? null : "screen");
    },
    onTool: toggle,
    ownsEscape: isHolding,
  });
}
