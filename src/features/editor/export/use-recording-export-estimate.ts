// SPDX-FileCopyrightText: 2026 overpolish
// SPDX-License-Identifier: GPL-3.0-or-later

import { useEffect, useReducer, useRef } from "react";

import { estimateRecordingExport } from "../api";

import { ExportLookQuality } from "./editor-export";
import { stopExportEstimate } from "./export-file-api";
import {
  RecordingExportInputs,
  recordingProcessingOptions,
} from "./recording-export-inputs";
import { mixSignature } from "./recording-export-settings";

const ESTIMATE_DEBOUNCE_MS = 450;

/**
 * The export's size at each of `looks`, measured while the Export island is
 * out, the only place they are shown. The four are measured together, in
 * one pass over the picture, and each answer is kept for as long as the
 * recording and the edit stay as they are: the export's own quality is left
 * out of what an answer is kept by, so choosing a look never measures again.
 * Measuring draws the export on the GPU as fast as it can, so running it
 * behind the editor stalls the preview, and a new one replaces the one
 * running.
 */
export function useRecordingExportEstimate(
  inputs: RecordingExportInputs,
  { isOpen, looks }: { isOpen: boolean; looks: readonly ExportLookQuality[] },
): (number | null)[] {
  const { artifact, enabledStreamIndices, includeCamera, includePrimaryVideo } =
    inputs;
  // The effect below runs on the signature; the options sent are read from
  // the latest inputs.
  const inputsRef = useRef(inputs);
  inputsRef.current = inputs;
  const answersRef = useRef(new Map<string, (number | null)[]>());
  const wasOpenRef = useRef(false);
  // Answers live in the ref; each one only needs the sizes shown again.
  const [, answered] = useReducer((count: number) => count + 1, 0);
  const enabledSignature = enabledStreamIndices
    ? mixSignature(enabledStreamIndices)
    : null;
  const hasExportableContent =
    includePrimaryVideo ||
    includeCamera ||
    (enabledSignature !== null && enabledSignature !== "silent");
  const sent = looks.map((look) => ({
    cameraCompression: look.camera?.compression ?? look.screen.compression,
    cameraResolutionScalePercent: look.camera?.resolutionScalePercent ?? 100,
    compression: look.screen.compression,
    resolutionScalePercent: look.screen.resolutionScalePercent,
  }));
  const signature =
    artifact?.kind === "recording" &&
    enabledSignature !== null &&
    hasExportableContent &&
    looks.length > 0
      ? JSON.stringify([
          artifact.id,
          includePrimaryVideo,
          includeCamera,
          inputs.bakeCamera,
          inputs.cursorEffects.bake,
          inputs.keyboardEffects.bake,
          enabledSignature,
          inputs.cameraOverlay,
          inputs.recordingOutput,
          // The whole edit: annotations, scenes and captions change what
          // the export draws, and so its size, as much as its cuts do.
          inputs.recordingTimelineEdit,
          inputs.collapseAudio,
          inputs.audioTrackVolumes,
          // What each look sets, which a separate camera changes.
          sent,
        ])
      : null;

  // Keyed on the id: a cancelled or failed save re-emits the same recording
  // as a new object, and its estimates are still good.
  const artifactId = artifact?.id;
  useEffect(() => {
    answersRef.current.clear();
  }, [artifactId]);

  useEffect(() => {
    const justOpened = isOpen && !wasOpenRef.current;
    // The island put away lets go of the estimate still being made, so it
    // does not hold the GPU from the preview.
    if (!isOpen && wasOpenRef.current)
      stopExportEstimate().catch((cause: unknown) => {
        console.error("Could not stop the size estimate", cause);
      });
    wasOpenRef.current = isOpen;
    if (!isOpen || artifactId === undefined || signature === null) return;
    const answers = answersRef.current;
    if (answers.has(signature)) return;
    let disposed = false;
    // With nothing yet on screen there is no choice to wait out.
    const delay = justOpened ? 0 : ESTIMATE_DEBOUNCE_MS;
    const timer = window.setTimeout(() => {
      const latest = inputsRef.current;
      void estimateRecordingExport({
        ...recordingProcessingOptions(latest),
        artifactId,
        enabledStreamIndices: latest.enabledStreamIndices ?? [],
        looks: sent,
      })
        .catch((cause: unknown) => {
          if (!disposed)
            console.error("Could not estimate the recording size", cause);
          return sent.map(() => null);
        })
        .then((bytes) => {
          if (disposed) return;
          answers.set(signature, bytes);
          answered();
        });
    }, delay);
    return () => {
      disposed = true;
      clearTimeout(timer);
    };
    // eslint-disable-next-line @eslint-react/exhaustive-deps -- `sent` is remade each render; the signature holds it.
  }, [artifactId, isOpen, signature]);

  const answer =
    signature === null ? undefined : answersRef.current.get(signature);
  return looks.map((_, index) => answer?.[index] ?? null);
}
