// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useRef, useState } from "react";

import { askPictureTween, TWEENED_PATCH_KEYS } from "../preview/picture-tween";
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
  if (values.annotationZoom !== undefined)
    on.onAnnotationZoomChange?.(values.annotationZoom);
  if (values.annotationImage !== undefined)
    on.onAnnotationImageChange?.(values.annotationImage);
  if (values.annotationLook !== undefined)
    on.onAnnotationLookChange?.(values.annotationLook);
  if (values.annotationEmphasis !== undefined)
    on.onAnnotationEmphasisChange?.(values.annotationEmphasis);
  if (values.annotationImagePlay !== undefined)
    on.onAnnotationImagePlayChange?.(values.annotationImagePlay);
  if (values.annotationImageSway !== undefined)
    on.onAnnotationImageSwayChange?.(values.annotationImageSway);
  if (values.annotationRotation !== undefined)
    on.onAnnotationRotationChange?.(values.annotationRotation);
  if (values.annotationKind !== undefined)
    on.onAnnotationKindChange?.(values.annotationKind);
  if (values.annotationPin !== undefined)
    on.onAnnotationPin?.(values.annotationPin);
  if (values.annotationSceneReset) on.onAnnotationSceneReset?.();
  if (values.reverseAnnotation) on.onAnnotationReverse?.();
  if (values.shuffleAnnotation) on.onAnnotationShuffle?.();
  if (values.deleteAnnotations) on.onAnnotationsDelete?.();
  if (values.saveAnnotationColor !== undefined)
    on.onAnnotationColorSave?.(values.saveAnnotationColor);
  if (values.removeAnnotationColor !== undefined)
    on.onAnnotationColorRemove?.(values.removeAnnotationColor);
  if (values.trackVolume !== undefined)
    on.onTrackVolumeChange?.(
      values.trackVolume.trackId,
      values.trackVolume.decibels,
    );
  if (values.reduceNoise !== undefined)
    on.onReduceNoiseChange?.(values.reduceNoise);
  if (values.vocalCleanup !== undefined)
    on.onVocalCleanupChange?.(values.vocalCleanup);
  if (values.autoVolume !== undefined)
    on.onAutoVolumeChange?.(values.autoVolume);
  if (values.audioClip) on.onAudioClipChange?.(values.audioClip);
  if (values.soundAddition) on.onSoundAdd?.(values.soundAddition);
  if (values.soundKind) on.onSoundKind?.(values.soundKind.kind);
  if (values.studioSound !== undefined)
    on.onStudioSoundChange?.(values.studioSound);
  if (values.downloadStudioSound) on.onStudioSoundDownload?.();
  if (values.studioDucking !== undefined)
    on.onStudioDuckingChange?.(values.studioDucking);
  if (values.downloadStudioDucking) on.onStudioDuckingDownload?.();
  if (values.cameraEffects !== undefined)
    on.onCameraEffectsChange?.(values.cameraEffects);
  if (values.chooseCameraBackground) on.onCameraBackgroundChoose?.();
  if (values.downloadCameraBackground) on.onCameraBackgroundDownload?.();
  if (values.downloadCameraObjects) on.onCameraObjectsDownload?.();
  if (values.pickCameraObjects !== undefined)
    on.onCameraObjectsPick?.(values.pickCameraObjects);
  if (values.removeCameraObject !== undefined)
    on.onCameraObjectRemove?.(values.removeCameraObject);
  if (values.showCameraObject !== undefined)
    on.onCameraObjectShow?.(values.showCameraObject);
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
  if (values.cropAspect !== undefined)
    on.onCropAspectChange?.(values.cropAspect);
  if (values.resetCrop) on.onCropReset?.();
  if (values.selectionDropShadow !== undefined) {
    on.onSelectionDropShadowChange?.(values.selectionDropShadow);
  }
  if (values.selectionFollowFace !== undefined) {
    on.onSelectionFollowFaceChange?.(values.selectionFollowFace);
  }
  if (values.selectionInset !== undefined) {
    on.onSelectionInsetChange?.(values.selectionInset);
  }
  if (values.selectionRadius !== undefined) {
    on.onSelectionRadiusChange?.(values.selectionRadius);
  }
  if (values.selectionRotation !== undefined) {
    on.onSelectionRotationChange?.(values.selectionRotation);
  }
  if (values.recenterSelection) on.onSelectionRecenter?.();
  if (values.resetPlacement) on.onSelectionPlacementReset?.();
  if (values.keyboardEffects !== undefined) {
    on.onKeyboardEffectsChange?.(values.keyboardEffects);
  }
  if (values.restoreShortcuts) on.onKeyboardShortcutsRestore?.();
  if (values.deleteChosenShortcuts) on.onChosenShortcutsDelete?.();
  if (values.scenePreset !== undefined)
    on.onScenePresetChoose?.(values.scenePreset);
  if (values.sceneFraming !== undefined)
    on.onSceneFramingChange?.(values.sceneFraming, values.sceneFramingPane);
  if (values.sceneCursor !== undefined)
    on.onSceneCursorChange?.(values.sceneCursor);
  if (values.sceneFollowFace !== undefined)
    on.onSceneFollowFaceChange?.(values.sceneFollowFace);
  if (values.sceneInset !== undefined)
    on.onSceneInsetChange?.(values.sceneInset);
  if (values.sceneRadius !== undefined)
    on.onSceneRadiusChange?.(values.sceneRadius);
  if (values.sceneRecenter) on.onSceneRecenter?.();
  if (values.scenePlacementReset)
    on.onScenePlacementReset?.(values.scenePlacementReset);
  if (values.sceneRotation !== undefined)
    on.onSceneRotationChange?.(values.sceneRotation);
  if (values.sceneVariant) on.onSceneVariantChange?.(values.sceneVariant);
  if (values.chooseSceneTemplate)
    on.onSceneTemplateChoose?.(values.chooseSceneTemplate);
  if (values.saveSceneTemplate) on.onSceneTemplateSave?.();
  if (values.removeSceneTemplate !== undefined)
    on.onSceneTemplateRemove?.(values.removeSceneTemplate);
  if (values.autoZoomScenes) on.onSceneAutoZoom?.();
  if (values.clearAutoZooms) on.onSceneAutoZoomsClear?.();
  if (values.deleteScene) on.onSceneDelete?.();
  if (values.swapScenePanes) on.onScenePanesSwap?.();
  if (values.transcriptRequest)
    on.onTranscriptRequest?.(values.transcriptRequest);
  if (values.captions) on.onCaptionsChange?.(values.captions);
  if (values.captionDefaults)
    on.onCaptionDefaultsChange?.(values.captionDefaults);
  if (values.cutStripItem) on.onStripItemCut?.();
  if (values.deselectStripItem) on.onStripItemDeselect?.();
  if (values.lullPlay !== undefined) on.onLullPlayChange?.(values.lullPlay);
  if (values.trimLullToBreath) on.onLullTrimToBreath?.();
  if (values.trackEnabled !== undefined)
    on.onTrackEnabledChange?.(
      values.trackEnabled.trackId,
      values.trackEnabled.isEnabled,
    );
  if (values.exportPatch) on.onExportPatch?.(values.exportPatch);
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
        // A setting chosen on an island is drawn as the picture moving to
        // it (`picture-tween.ts`).
        const values = lastRequest.request.values;
        if (TWEENED_PATCH_KEYS.some((key) => values[key] !== undefined))
          askPictureTween(workspace);
        applyPatch(values, handlersRef.current);
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
