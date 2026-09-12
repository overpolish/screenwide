// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { SelectionPlacementPatch } from "../selection-placement";
import { CursorEffectSettings, EditorKind } from "../types";

import {
  ToolPanelPatch,
  ToolPanelSnapshot,
  useToolPanelRequestStore,
  useToolPanelStore,
} from "./tool-panel-store";

/**
 * What a tool panel can ask for. Each one is an editor handler that already
 * exists, so a control in a panel and the same control in the editor take the
 * identical path through the workspace's state.
 */
export type ToolPanelHandlers = {
  /** Fill the workspace's canvas with this background. */
  onBackgroundChange?: (background: Background) => void;
  onBackgroundPresetRemove?: (id: string) => void;
  onBackgroundPresetSave?: (preset: BackgroundPreset) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onFrameReset?: () => void;
  /** Size the output canvas to what a field asked for. */
  onFrameSizeChange?: (size: { height?: number; width?: number }) => void;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  onSelectionDropShadowChange?: (dropShadow: boolean) => void;
  /** Place the selection at the size and position a field asked for. */
  onSelectionPlacementChange?: (placement: SelectionPlacementPatch) => void;
  onSelectionReset?: () => void;
};

/**
 * The last request each workspace has acted on. Module state rather than a
 * ref: a request outlives any one mount of the editor panel.
 */
const appliedSeq: Record<EditorKind, number> = { recording: 0, screenshot: 0 };

const applyPatch = (values: ToolPanelPatch, on: ToolPanelHandlers) => {
  if (values.background !== undefined)
    on.onBackgroundChange?.(values.background);
  if (values.savePreset !== undefined) {
    on.onBackgroundPresetSave?.(values.savePreset);
  }
  if (values.removePreset !== undefined) {
    on.onBackgroundPresetRemove?.(values.removePreset);
  }
  if (values.cursorEffects !== undefined) {
    on.onCursorEffectsChange?.(values.cursorEffects);
  }
  if (values.frameSize !== undefined) on.onFrameSizeChange?.(values.frameSize);
  if (values.resetFrame) on.onFrameReset?.();
  if (values.selectionDropShadow !== undefined) {
    on.onSelectionDropShadowChange?.(values.selectionDropShadow);
  }
  if (values.selectionOutput !== undefined) {
    on.onSelectionPlacementChange?.(values.selectionOutput);
  }
  if (values.resetSelection) on.onSelectionReset?.();
};

/**
 * The editor's half of the tool panels: it publishes the settings they show,
 * and applies what they ask for.
 *
 * Only the editor window owns this state, so a panel never writes a setting
 * itself; it sends a request, and reads the result back out of the next
 * published snapshot.
 */
export function useToolPanelBridge(
  workspace: EditorKind,
  snapshot: ToolPanelSnapshot,
  handlers: ToolPanelHandlers,
) {
  const handlersRef = useRef(handlers);
  const [acknowledged, setAcknowledged] = useState(() => ({ ...appliedSeq }));

  useEffect(() => {
    handlersRef.current = handlers;
  });

  useEffect(() => {
    // A no-op when nothing changed, so republishing on every render costs a
    // comparison rather than a `localStorage` write.
    useToolPanelStore.getState().publish(workspace, {
      ...snapshot,
      acknowledgedSeq: acknowledged[workspace],
    });
  }, [acknowledged, snapshot, workspace]);

  useEffect(() => {
    let disposed = false;
    const receive = () => {
      if (disposed) return;
      const { lastRequest } = useToolPanelRequestStore.getState();
      if (
        !lastRequest ||
        lastRequest.workspace !== workspace ||
        lastRequest.seq <= appliedSeq[workspace]
      )
        return;
      appliedSeq[workspace] = lastRequest.seq;
      if (lastRequest.request.type === "shortcut") {
        window.dispatchEvent(
          new KeyboardEvent("keydown", {
            ...lastRequest.request.event,
            cancelable: true,
          }),
        );
      } else {
        applyPatch(lastRequest.request.values, handlersRef.current);
      }
      // Batched with the editor's settings update, including no-op requests.
      // The publishing effect acknowledges only the resulting committed render.
      setAcknowledged((current) => ({
        ...current,
        [workspace]: lastRequest.seq,
      }));
    };
    const unsubscribe = useToolPanelRequestStore.subscribe(receive);
    queueMicrotask(receive);
    return () => {
      disposed = true;
      unsubscribe();
    };
  }, [workspace]);
}
