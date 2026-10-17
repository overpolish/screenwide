// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later
/* eslint-disable @eslint-react/set-state-in-effect -- Export estimation owns this external lifecycle. */

import { useEffect, useRef, useState } from "react";

import { estimateRecordingExport } from "../api";

import { useExportOptionsOpen } from "./options-window/use-export-options-open";
import {
  RecordingExportInputs,
  recordingProcessingOptions,
} from "./recording-export-inputs";
import { mixSignature } from "./recording-export-settings";

const ESTIMATE_DEBOUNCE_MS = 450;

/**
 * The export's size, measured while the export window is open, the only
 * place it is shown. Measuring draws stretches of the export on the GPU as
 * fast as it can, so running it behind the editor stalls the preview.
 */
export function useRecordingExportEstimate(inputs: RecordingExportInputs) {
  const { isExportOpen } = useExportOptionsOpen("recording");
  const {
    artifact,
    audioTrackVolumes,
    bakeCamera,
    camera,
    cameraOverlay,
    collapseAudio,
    compression,
    cursorEffects,
    enabledStreamIndices,
    includeCamera,
    includePrimaryVideo,
    keyboardEffects,
    recordingOutput,
    recordingTimelineEdit,
    resolutionScalePercent,
  } = inputs;
  // The effect below runs on the fields its signature names; the options
  // sent are read from the latest inputs.
  const inputsRef = useRef(inputs);
  inputsRef.current = inputs;
  const cacheRef = useRef(new Map<string, number>());
  const wasOpenRef = useRef(false);
  const [activeJobs, setActiveJobs] = useState(0);
  const [state, setState] = useState<{
    bytes: number | null;
    isEstimating: boolean;
    signature: string;
  } | null>(null);
  const enabledSignature = enabledStreamIndices
    ? mixSignature(enabledStreamIndices)
    : null;
  const hasExportableContent =
    includePrimaryVideo ||
    includeCamera ||
    (enabledSignature !== null && enabledSignature !== "silent");
  const signature =
    artifact?.kind === "recording" &&
    enabledSignature !== null &&
    hasExportableContent
      ? [
          artifact.id,
          includePrimaryVideo ? "primary" : "no-primary",
          includeCamera ? "camera" : "no-camera",
          bakeCamera ? "baked" : "separate",
          compression,
          cursorEffects.bake ? "cursor" : "no-cursor",
          keyboardEffects.bake ? "keyboard" : "no-keyboard",
          resolutionScalePercent,
          camera.compression,
          camera.resolutionScalePercent,
          enabledSignature,
          cameraOverlay.cameraX,
          cameraOverlay.cameraY,
          cameraOverlay.cameraWidth,
          cameraOverlay.frameHeight,
          cameraOverlay.frameWidth,
          cameraOverlay.frameX,
          cameraOverlay.frameY,
          cameraOverlay.radiusPercent,
          JSON.stringify(recordingOutput),
          // The whole edit: annotations, scenes and captions change what
          // the export draws, and so its size, as much as its cuts do.
          JSON.stringify(recordingTimelineEdit),
          collapseAudio ? "mix" : "separate",
          audioTrackVolumes
            .map(
              (volume) =>
                `${volume.streamIndex.toString()}-${volume.decibels.toString()}`,
            )
            .join(","),
        ].join(":")
      : null;

  // Keyed on the id: a cancelled or failed save re-emits the same recording
  // as a new object, and its estimates are still good.
  const artifactId = artifact?.id;
  useEffect(() => {
    cacheRef.current.clear();
    setState(null);
  }, [artifactId]);

  useEffect(() => {
    const justOpened = isExportOpen && !wasOpenRef.current;
    wasOpenRef.current = isExportOpen;
    if (
      artifact?.kind !== "recording" ||
      enabledSignature === null ||
      signature === null
    )
      return;

    const cached = cacheRef.current.get(signature);
    if (cached !== undefined) {
      setState({ bytes: cached, isEstimating: false, signature });
      return;
    }
    if (!isExportOpen) return;

    setState({ bytes: null, isEstimating: true, signature });
    let disposed = false;
    // With nothing yet on screen there is no choice to wait out.
    const delay =
      justOpened ||
      (!bakeCamera && compression === 0 && camera.compression === 0)
        ? 0
        : ESTIMATE_DEBOUNCE_MS;
    const timer = window.setTimeout(() => {
      const streamIndices =
        enabledSignature === "silent"
          ? []
          : enabledSignature.split("-").map(Number);
      setActiveJobs((count) => count + 1);
      estimateRecordingExport({
        ...recordingProcessingOptions(inputsRef.current),
        artifactId: artifact.id,
        enabledStreamIndices: streamIndices,
      })
        .then((bytes) => {
          if (disposed) return;
          cacheRef.current.set(signature, bytes);
          setState({ bytes, isEstimating: false, signature });
        })
        .catch((cause: unknown) => {
          if (disposed) return;
          console.error("Could not estimate the recording size", cause);
          setState({ bytes: null, isEstimating: false, signature });
        })
        .finally(() => {
          setActiveJobs((count) => Math.max(0, count - 1));
        });
    }, delay);

    return () => {
      disposed = true;
      clearTimeout(timer);
    };
  }, [
    artifact,
    audioTrackVolumes,
    bakeCamera,
    camera.compression,
    camera.resolutionScalePercent,
    cameraOverlay,
    collapseAudio,
    compression,
    cursorEffects,
    keyboardEffects,
    enabledSignature,
    includeCamera,
    includePrimaryVideo,
    isExportOpen,
    resolutionScalePercent,
    recordingOutput,
    recordingTimelineEdit,
    signature,
  ]);

  const current =
    signature !== null && state?.signature === signature ? state : null;
  return {
    estimatedSizeBytes: current?.bytes,
    isEstimatingSize:
      artifact?.kind === "recording" &&
      hasExportableContent &&
      (current === null || current.isEstimating),
    isPending: activeJobs > 0,
  };
}
