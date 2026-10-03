// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { EditorKind } from "../types";

import { ToolPanelHandlers } from "./tool-panel-handlers";
import { ToolPanelPatch } from "./tool-panel-patch";
import {
  ToolPanelSnapshot,
  useToolPanelRequestStore,
  useToolPanelStore,
} from "./tool-panel-store";

/**
 * The last request each workspace has acted on. Module state rather than a
 * ref: a request outlives any one mount of the editor panel.
 */
const appliedSeq: Record<EditorKind, number> = { recording: 0, screenshot: 0 };

const applyPatch = (values: ToolPanelPatch, on: ToolPanelHandlers) => {
  if (values.annotationAngle !== undefined)
    on.onAnnotationAngleChange?.(values.annotationAngle);
  if (values.annotationAnimated !== undefined)
    on.onAnnotationAnimatedChange?.(values.annotationAnimated);
  if (values.annotationStyle !== undefined)
    on.onAnnotationStyleChange?.(values.annotationStyle);
  if (values.reverseAnnotation) on.onAnnotationReverse?.();
  if (values.shuffleAnnotation) on.onAnnotationShuffle?.();
  if (values.clearDrawings) on.onDrawingsClear?.();
  if (values.deleteAnnotations) on.onAnnotationsDelete?.();
  if (values.saveAnnotationColor !== undefined)
    on.onAnnotationColorSave?.(values.saveAnnotationColor);
  if (values.removeAnnotationColor !== undefined)
    on.onAnnotationColorRemove?.(values.removeAnnotationColor);
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
  if (values.frameRadius !== undefined)
    on.onFrameRadiusChange?.(values.frameRadius);
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
  if (values.scenePreset !== undefined)
    on.onScenePresetChoose?.(values.scenePreset);
  if (values.customizeScene) on.onSceneCustomize?.();
  if (values.sceneFraming !== undefined)
    on.onSceneFramingChange?.(values.sceneFraming);
  if (values.sceneRadius !== undefined)
    on.onSceneRadiusChange?.(values.sceneRadius);
  if (values.sceneVariant) on.onSceneVariantChange?.(values.sceneVariant);
  if (values.chooseSceneTemplate)
    on.onSceneTemplateChoose?.(values.chooseSceneTemplate);
  if (values.saveSceneTemplate) on.onSceneTemplateSave?.();
  if (values.removeSceneTemplate !== undefined)
    on.onSceneTemplateRemove?.(values.removeSceneTemplate);
  if (values.resetSceneFraming) on.onSceneFramingReset?.();
  if (values.autoZoomScenes) on.onSceneAutoZoom?.();
  if (values.deleteScene) on.onSceneDelete?.();
  if (values.swapScenePanes) on.onScenePanesSwap?.();
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
  // What has been applied, and what the panels have been told about. They are
  // deliberately one commit apart: see the acknowledging effect below.
  const [applied, setApplied] = useState(() => ({ ...appliedSeq }));
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

  // The acknowledgement is what makes a panel drop the value it is holding and
  // show the mirror instead, so it must never travel with a snapshot the
  // change has not reached yet.
  //
  // Applying a patch renders the editor at once, but a value the preview
  // owns - the chosen annotation's dress, published through
  // `annotation-channel` - only reaches this snapshot on the render that
  // channel's own publish provokes.
  // Acknowledging a commit later ties the sequence to that render: until then
  // the panel keeps showing what it asked for, rather than pinging back to the
  // previous value for a frame and then settling on the new one. A field the
  // editor itself owns is already published by then, so its acknowledgement
  // costs one more write of the same snapshot.
  useEffect(() => {
    if (applied[workspace] === acknowledged[workspace]) return;
    // The extra render is the point: it is what carries the sequence out on a
    // snapshot the change has reached.
    // eslint-disable-next-line @eslint-react/set-state-in-effect
    setAcknowledged((current) => ({
      ...current,
      [workspace]: applied[workspace],
    }));
  }, [acknowledged, applied, workspace]);

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
      // Batched with the editor's settings update, including no-op requests:
      // a request that changes nothing still has to be acknowledged, or the
      // panel would hold its draft for good.
      setApplied((current) => ({
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
