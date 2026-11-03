// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useState } from "react";

import { useExportIslandKeys } from "../shortcuts/export-island-keys";

import { CameraOutput } from "./camera-output";
import { applyExportPatch, exportIslandSnapshot } from "./editor-export";
import { putAwayExport } from "./export-file-api";
import {
  resetExportIsland,
  setExportIslandOpen,
  useExportIslandStore,
} from "./export-island-store";
import { useExportLightDismiss } from "./use-export-light-dismiss";
import { useExportProgressStore } from "./use-export-progress";

import type { EditorPanelProps } from "../panel/editor-panel-props";
import type { ToolPanelExportPatch } from "../tools/tool-panel-export";

/** How long the finished Export island stays before it shrinks away. */
const FINISH_LINGER_MS = 8_000;

const noop = () => undefined;

/**
 * The editor's side of the Export island: whether it stands over the editor,
 * what it shows, and what its asks do, through the editor window's own
 * handlers. The title bar's button and Escape bring it out and put it
 * away; while it exports they tuck it away, and once done they put the
 * finish away, as it goes on its own after a while, leaving the editor on
 * the project.
 */
export function useEditorExportIsland(
  props: EditorPanelProps,
  {
    cameraOutput,
    canOpen,
    isAudioOnly,
  }: {
    cameraOutput: CameraOutput | null;
    /** Whether there is anything to export yet. */
    canOpen: boolean;
    isAudioOnly: boolean;
  },
) {
  const { artifact, fileStem, isSaving = false } = props;
  // Subtitles are offered once there is a transcript, which the tool panels
  // learn of: the island reads it a render later.
  const [hasTranscript, setHasTranscript] = useState(false);
  const { exported, failure, isOpen } = useExportIslandStore();
  const progress = useExportProgressStore();
  // Open, exporting (tucked away too) or done, the island stands over the
  // editor: nothing round the picture is edited.
  const isExportOpen = isOpen || isSaving || exported !== null;
  const dismiss = () => {
    resetExportIsland();
    putAwayExport().catch((cause: unknown) => {
      console.error("Could not put the finished export away", cause);
    });
  };
  const toggle = () => {
    if (exported) dismiss();
    else setExportIslandOpen(!isOpen);
  };
  // A press off the island, or the island's own ask, puts it away before
  // the export and lets the finish go after it; while it exports nothing
  // but Cancel export stops it, so a press off it does nothing.
  const putAway = isSaving
    ? null
    : () => {
        if (exported) dismiss();
        else setExportIslandOpen(false);
      };
  useExportIslandKeys(isExportOpen ? toggle : null);
  useExportLightDismiss(isOpen || exported !== null ? putAway : null);
  const artifactId = artifact?.id;
  useEffect(() => {
    resetExportIsland();
  }, [artifactId]);
  useEffect(() => {
    if (!exported) return;
    const timer = window.setTimeout(dismiss, FINISH_LINGER_MS);
    return () => {
      clearTimeout(timer);
    };
  }, [exported]);
  const shown =
    artifact &&
    exportIslandSnapshot({
      artifact,
      cameraOutput,
      canExport: canOpen && fileStem.trim().length > 0 && !isSaving,
      compare: props.exportCompare ?? null,
      enabledAudioTrackCount: props.enabledAudioTrackCount ?? 0,
      exported,
      failure,
      hasTranscript,
      isAudioOnly,
      lookSizeBytes: props.lookSizeBytes ?? [],
      recordingOutput: props.recordingOutput ?? null,
      recordingTimelineEdit: props.recordingTimelineEdit ?? null,
      settings: {
        cameraCompression: props.cameraCompression ?? 0,
        cameraKey: props.cameraKey ?? "green",
        cameraResolutionScalePercent: props.cameraResolutionScalePercent ?? 100,
        collapseAudio: props.collapseAudio ?? false,
        compression: props.compression ?? 0,
        deleteProjectAfterExport: props.deleteProjectAfterExport ?? false,
        directory: props.directory,
        fileStem,
        isCancelingSave: props.isCancelingSave ?? false,
        isSaving,
        resolutionScalePercent: props.resolutionScalePercent ?? 100,
        screenshotFormat: props.screenshotFormat ?? "png",
        subtitles: props.subtitles ?? "none",
      },
    });
  const onPatch = (patch: ToolPanelExportPatch) => {
    if (patch.dismiss) putAway?.();
    applyExportPatch(
      patch,
      {
        onBrowse: props.onBrowse ?? noop,
        onCameraCompressionChange: props.onCameraCompressionChange ?? noop,
        onCameraKeyChange: props.onCameraKeyChange ?? noop,
        onCameraOutputChange: (output) => {
          props.onBakeCameraChange?.(output === "combined");
        },
        onCameraResolutionScaleChange:
          props.onCameraResolutionScaleChange ?? noop,
        onCancelSave: props.onCancelSave ?? noop,
        onCollapseAudioChange: props.onCollapseAudioChange ?? noop,
        onCompressionChange: props.onCompressionChange ?? noop,
        onDeleteProjectAfterExportChange:
          props.onDeleteProjectAfterExportChange ?? noop,
        onExport: props.onSave ?? noop,
        onFileStemChange: props.onFileStemChange ?? noop,
        onResolutionScaleChange: props.onResolutionScaleChange ?? noop,
        onScreenshotFormatChange: props.onScreenshotFormatChange ?? noop,
        onSubtitlesChange: props.onSubtitlesChange ?? noop,
      },
      props.lookQualities ?? [],
    );
  };
  // Presets only: settings that match no look, as an older export may have
  // left them, take Balanced as the island comes out.
  const isUnmatched = isOpen && shown?.quality?.look === null;
  useEffect(() => {
    if (isUnmatched) onPatch({ look: "balanced" });
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- only settings found to match no look are moved; `onPatch` is remade each render.
  }, [isUnmatched]);
  return {
    /** Standing over the editor: open, exporting or done. */
    isExportOpen,
    /** Out over the picture, rather than tucked away or put away. */
    isOpen,
    /** For the tool panels, which publish it to the island's window. */
    island: {
      onPatch,
      snapshot: shown?.progress
        ? {
            ...shown,
            progress: {
              ...shown.progress,
              etaSeconds: progress.etaSeconds,
              percent: progress.saveProgress,
              phase: progress.savePhase,
            },
          }
        : shown,
    },
    /** The running save's whole percent, for the title bar's ring. */
    saveProgress: isSaving ? progress.saveProgress : undefined,
    /** The tool panels learnt whether there is a transcript. */
    setHasTranscript,
    toggle,
  };
}
