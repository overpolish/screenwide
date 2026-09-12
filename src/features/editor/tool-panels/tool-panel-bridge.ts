// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import {
  Background,
  BackgroundPreset,
} from "../../../components/shared/background-picker/background";
import { SelectionPlacementPatch } from "../selection-placement";
import {
  CursorEffectSettings,
  EditorKind,
  KeyboardEffectSettings,
} from "../types";

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
  /** Play the selected audio track this much louder or quieter than it was
   * recorded, in decibels. */
  onAudioVolumeChange?: (decibels: number) => void;
  /** Fill the workspace's canvas with this background. */
  onBackgroundChange?: (background: Background) => void;
  onBackgroundPresetRemove?: (id: string) => void;
  onBackgroundPresetSave?: (preset: BackgroundPreset) => void;
  /** Draw the camera into the screen's picture, or carry it as a track of its
   * own. The placement of each is carried across the change. */
  onBakeCameraChange?: (bake: boolean) => void;
  /** Show the whole source again, the committed crop taken away. */
  onCropReset?: () => void;
  /** Cut a crop of the size a field asked for, in source pixels. */
  onCropSizeChange?: (size: { height?: number; width?: number }) => void;
  onCursorEffectsChange?: (settings: CursorEffectSettings) => void;
  onFrameReset?: () => void;
  /** Size the output canvas to what a field asked for. */
  onFrameSizeChange?: (size: { height?: number; width?: number }) => void;
  /** Draw every shortcut with these settings. */
  onKeyboardEffectsChange?: (settings: Partial<KeyboardEffectSettings>) => void;
  /** Put the shortcuts back where the recording draws them by default. */
  onKeyboardPositionReset?: () => void;
  /** Put every shortcut's own placement away, the global one left as it is. */
  onKeyboardShortcutsResetAll?: () => void;
  /** Bring back every shortcut deleted from the timeline. */
  onKeyboardShortcutsRestore?: () => void;
  /** Cast the selected layer's shadow onto the canvas, or take it away. */
  onSelectionDropShadowChange?: (dropShadow: boolean) => void;
  /** Pad the selected layer by this many output pixels on each side. */
  onSelectionInsetChange?: (inset: number) => void;
  /** Place the selection at the size and position a field asked for. */
  onSelectionPlacementChange?: (placement: SelectionPlacementPatch) => void;
  /** Round the selected layer's corners by this share of its shorter side. */
  onSelectionRadiusChange?: (radius: number) => void;
  /** Put the layer's content in the middle of its padded frame. */
  onSelectionRecenter?: () => void;
  onSelectionReset?: () => void;
  /** Give every shortcut the selected one's size and position. */
  onShortcutApplyToAll?: () => void;
  /** Draw the selected shortcut at this size and centre, all in percent. */
  onShortcutPlacementChange?: (
    placement: NonNullable<ToolPanelPatch["shortcutPlacement"]>,
  ) => void;
  /** Put the selected shortcut back where the recording drew it. */
  onShortcutReset?: () => void;
};

/**
 * The last request each workspace has acted on. Module state rather than a
 * ref: a request outlives any one mount of the editor panel.
 */
const appliedSeq: Record<EditorKind, number> = { recording: 0, screenshot: 0 };

const applyPatch = (values: ToolPanelPatch, on: ToolPanelHandlers) => {
  if (values.audioVolume !== undefined)
    on.onAudioVolumeChange?.(values.audioVolume);
  if (values.bakeCamera !== undefined)
    on.onBakeCameraChange?.(values.bakeCamera);
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
  if (values.cropSize !== undefined) on.onCropSizeChange?.(values.cropSize);
  if (values.resetCrop) on.onCropReset?.();
  if (values.selectionDropShadow !== undefined) {
    on.onSelectionDropShadowChange?.(values.selectionDropShadow);
  }
  if (values.selectionInset !== undefined) {
    on.onSelectionInsetChange?.(values.selectionInset);
  }
  if (values.selectionOutput !== undefined) {
    on.onSelectionPlacementChange?.(values.selectionOutput);
  }
  if (values.selectionRadius !== undefined) {
    on.onSelectionRadiusChange?.(values.selectionRadius);
  }
  if (values.recenterSelection) on.onSelectionRecenter?.();
  if (values.resetSelection) on.onSelectionReset?.();
  if (values.keyboardEffects !== undefined) {
    on.onKeyboardEffectsChange?.(values.keyboardEffects);
  }
  if (values.restoreShortcuts) on.onKeyboardShortcutsRestore?.();
  if (values.resetAllShortcuts) on.onKeyboardShortcutsResetAll?.();
  if (values.resetKeyboardPosition) on.onKeyboardPositionReset?.();
  if (values.shortcutPlacement !== undefined) {
    on.onShortcutPlacementChange?.(values.shortcutPlacement);
  }
  if (values.resetShortcut) on.onShortcutReset?.();
  if (values.applyShortcutToAll) on.onShortcutApplyToAll?.();
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
