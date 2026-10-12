// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { ReactNode } from "react";

import { ToolPanelKind } from "../../popup-panel/store";
import { EditorKind } from "../types";

import { AnnotationPanel } from "./annotation/annotation-panel";
import { CropPanel } from "./crop/crop-panel";
import { CursorPanel } from "./cursor/cursor-panel";
import { FramePanel } from "./frame/frame-panel";
import { KeyboardPanel } from "./keyboard/keyboard-panel";
import { ScenePanel } from "./scene/scene-panel";
import { SelectionPanel } from "./selection/selection-panel";
import { TranscriptPanel } from "./transcript/transcript-panel";
import { usePanelShortcuts } from "./use-panel-shortcuts";

const toolPanels: Record<
  ToolPanelKind,
  (props: { workspace: EditorKind }) => ReactNode
> = {
  annotation: AnnotationPanel,
  crop: CropPanel,
  cursor: CursorPanel,
  frame: FramePanel,
  keyboard: KeyboardPanel,
  scene: ScenePanel,
  selection: SelectionPanel,
  transcript: TranscriptPanel,
};

/** The panel window's surface and inset, which every tool's controls sit
 * in. A story showing a panel's controls without the editor behind them uses
 * it too. */
export function ToolPanelSurface({ children }: { children: ReactNode }) {
  return (
    <section className="window-surface flex w-full flex-col gap-section overflow-hidden rounded-window p-window-inset text-content-fg windows:inset-ring windows:inset-ring-popover-stroke">
      {children}
    </section>
  );
}

/**
 * One editor tool's controls, as they appear in the panel window.
 *
 * The chrome is the panel's own: the window surface and its inset. The panel
 * is never anything but the tool in hand, so it does not name it; every tool
 * is presented the same way, and a panel is only ever asked for its controls.
 */
export function ToolPanel({
  tool,
  workspace,
}: {
  tool: ToolPanelKind;
  workspace: EditorKind;
}) {
  usePanelShortcuts(workspace);
  const Panel = toolPanels[tool];

  return (
    <ToolPanelSurface>
      <Panel workspace={workspace} />
    </ToolPanelSurface>
  );
}
