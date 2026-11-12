// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { stripIslandControls } from "../../timeline/recording-strip/strip-island-channel";
import { EditorKind } from "../../types";
import { ToolPanelHandlers } from "../tool-panel-handlers";

/** What the strip islands' asks do in `workspace`: they reach the preview,
 * which holds the strip and what is chosen on it. */
export const stripPanelHandlers = (
  workspace: EditorKind,
): Pick<
  ToolPanelHandlers,
  | "onAudioClipChange"
  | "onChosenShortcutsDelete"
  | "onLullPlayChange"
  | "onLullTrimToBreath"
  | "onSoundAdd"
  | "onStripItemCut"
  | "onStripItemDeselect"
> => ({
  onAudioClipChange: (change) => {
    stripIslandControls(workspace)?.changeAudioClip(change);
  },
  onChosenShortcutsDelete: () => {
    stripIslandControls(workspace)?.deleteShortcuts();
  },
  onLullPlayChange: (play) => {
    stripIslandControls(workspace)?.playLull(play);
  },
  onLullTrimToBreath: () => {
    stripIslandControls(workspace)?.trimLullToBreath();
  },
  onSoundAdd: (addition) => {
    stripIslandControls(workspace)?.addSound(addition);
  },
  onStripItemCut: () => {
    stripIslandControls(workspace)?.cut();
  },
  onStripItemDeselect: () => {
    stripIslandControls(workspace)?.deselect();
  },
});
